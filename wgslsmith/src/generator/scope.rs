use ast::types::DataType;
use rand::prelude::IteratorRandom;
use rand::Rng;
use rpds::{HashTrieSet, HashTrieSetSync, Vector};

type TypeSet = HashTrieSetSync<DataType, crate::BuildFxHasher>;

#[derive(Clone, Debug)]
pub struct Scope {
    next_name: u32,
    consts: Vector<(String, DataType)>,
    const_types: TypeSet,
    vars: Vector<(String, DataType)>,
    var_types: TypeSet,
}

impl Scope {
    pub fn empty() -> Scope {
        Scope {
            next_name: 0,
            consts: Vector::new(),
            const_types: HashTrieSet::new_with_hasher_with_ptr_kind(crate::BuildFxHasher),
            vars: Vector::new(),
            var_types: HashTrieSet::new_with_hasher_with_ptr_kind(crate::BuildFxHasher),
        }
    }

    pub fn has_vars(&self) -> bool {
        !self.vars.is_empty()
    }

    pub fn contains_ty(&self, ty: &DataType) -> bool {
        self.const_types.contains(ty) || self.var_types.contains(ty)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &DataType)> {
        self.consts
            .iter()
            .chain(self.vars.iter())
            .map(|(n, t)| (n, t))
    }

    pub fn choose_var(&self, rng: &mut impl Rng) -> (&String, &DataType) {
        self.vars.iter().choose(rng).map(|(n, t)| (n, t)).unwrap()
    }

    pub fn insert_let(&mut self, name: String, data_type: DataType) {
        self.consts.push_back_mut((name, data_type.clone()));
        self.const_types.insert_mut(data_type);
    }

    pub fn insert_var(&mut self, name: String, data_type: DataType) {
        self.vars.push_back_mut((name, data_type.clone()));
        self.var_types.insert_mut(data_type);
    }

    pub fn next_name(&mut self) -> String {
        let next = self.next_name;
        self.next_name += 1;
        format!("var_{}", next)
    }
}
