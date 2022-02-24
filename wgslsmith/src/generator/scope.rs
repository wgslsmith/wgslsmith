use std::iter;

use ast::types::DataType;
use rand::prelude::IteratorRandom;
use rand::Rng;
use rpds::{HashTrieMap, Vector};

#[derive(Clone, Debug)]
pub struct Scope {
    next_name: u32,
    symbols: HashTrieMap<DataType, Vec<(String, DataType)>>,
    mutables: Vector<(String, DataType)>,
}

impl Scope {
    pub fn empty() -> Scope {
        Scope {
            next_name: 0,
            symbols: HashTrieMap::new(),
            mutables: Vector::new(),
        }
    }

    pub fn has_mutables(&self) -> bool {
        !self.mutables.is_empty()
    }

    pub fn of_type(&self, ty: &DataType) -> &[(String, DataType)] {
        self.symbols.get(ty).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn choose_mutable(&self, rng: &mut impl Rng) -> (&String, &DataType) {
        self.mutables
            .iter()
            .choose(rng)
            .map(|(n, t)| (n, t))
            .unwrap()
    }

    pub fn insert_let(&mut self, name: String, data_type: DataType) {
        self.insert_symbol(&name, &data_type);
    }

    pub fn insert_var(&mut self, name: String, data_type: DataType) {
        self.insert_symbol(&name, &data_type);
        self.mutables.push_back_mut((name, data_type));
    }

    fn insert_symbol(&mut self, name: &str, ty: &DataType) {
        for key in iter::once(ty.clone()).chain(accessible_types_of(ty)) {
            let symbols = if let Some(symbols) = self.symbols.get_mut(&key) {
                symbols
            } else {
                self.symbols.insert_mut(key.clone(), Vec::new());
                self.symbols.get_mut(&key).unwrap()
            };

            symbols.push((name.to_owned(), ty.clone()));
        }
    }

    pub fn next_name(&mut self) -> String {
        let next = self.next_name;
        self.next_name += 1;
        format!("var_{}", next)
    }
}

/// Computes the types which are accessible through this type via member access, etc.
fn accessible_types_of(ty: &DataType) -> Vec<DataType> {
    match ty {
        DataType::Scalar(_) => vec![],
        DataType::Vector(n, ty) => {
            let mut derived = vec![DataType::Scalar(*ty)];
            // Add all smaller vectors accessible by swizzling
            for i in 2..*n {
                derived.push(DataType::Vector(i, *ty));
            }
            derived
        }
        DataType::Array(ty) => vec![(**ty).clone()],
        DataType::User(decl) => decl.accessible_types().cloned().collect(),
    }
}
