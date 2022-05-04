use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::rc::Rc;

use crate::types::DataType;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum StructMemberAttr {
    Align(u8),
}

impl Display for StructMemberAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StructMemberAttr::Align(n) => write!(f, "align({n})"),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StructMember {
    pub attrs: Vec<StructMemberAttr>,
    pub name: String,
    pub data_type: DataType,
}

impl StructMember {
    pub fn new(
        attrs: Vec<StructMemberAttr>,
        name: impl Into<String>,
        ty: DataType,
    ) -> Rc<StructMember> {
        Rc::new(StructMember {
            attrs,
            name: name.into(),
            data_type: ty,
        })
    }
}

#[derive(Clone, Debug)]
pub struct StructDecl {
    pub name: String,
    pub members: Vec<Rc<StructMember>>,
    accessors: HashMap<DataType, Vec<Rc<StructMember>>>,
}

impl Hash for StructDecl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for StructDecl {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for StructDecl {}

impl StructDecl {
    pub fn new(name: impl Into<String>, members: Vec<Rc<StructMember>>) -> Rc<StructDecl> {
        let name = name.into();
        let accessors = collect_struct_accessors(&members);
        Rc::new(StructDecl {
            name,
            members,
            accessors,
        })
    }

    /// Returns a list of members through which the specified type may be accessed (directly or
    /// indirectly).
    pub fn accessors_of(&self, ty: &DataType) -> &[Rc<StructMember>] {
        let x = self.accessors.get(ty).map(Vec::as_slice).unwrap_or(&[]);
        if x.is_empty() {
            println!("{:?} -> {} {:?}", ty, self.name, self.accessors);
        }
        x
    }

    /// Returns all types that can be accessed (directly or indirectly) through this struct type.
    pub fn accessible_types(&self) -> impl Iterator<Item = &DataType> {
        self.accessors.keys()
    }
}

/// For a list of struct members, this function will build a mapping from data types to lists of
/// members through which those types can be accessed.
fn collect_struct_accessors(
    members: &[Rc<StructMember>],
) -> HashMap<DataType, Vec<Rc<StructMember>>> {
    let mut accessors = HashMap::new();

    fn insert(
        map: &mut HashMap<DataType, HashSet<Rc<StructMember>>>,
        ty: &DataType,
        member: &Rc<StructMember>,
    ) {
        map.entry(ty.clone()).or_default().insert(member.clone());
    }

    for member in members {
        insert(&mut accessors, &member.data_type, member);

        match &member.data_type {
            DataType::Scalar(_) => {}
            DataType::Vector(n, ty) => {
                // Access to component type
                insert(&mut accessors, &DataType::Scalar(*ty), member);

                // Access to subvectors via swizzling
                for i in 2..*n {
                    insert(&mut accessors, &DataType::Vector(i, *ty), member);
                }
            }
            DataType::Array(_, _) => {
                // TODO
            }
            DataType::Struct(decl) => {
                for ty in decl.accessible_types() {
                    insert(&mut accessors, ty, member);
                }
            }
        }
    }

    // Convert the sets into vectors
    // We use vectors for more efficient random selection later on
    accessors
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect()
}
