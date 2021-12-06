use rand::prelude::IteratorRandom;
use rand::Rng;
use rpds::Vector;

use crate::types::{DataType, TypeConstraints};

#[derive(Clone, Debug)]
pub struct Scope {
    next_name: u32,
    consts: Vector<(String, DataType)>,
    const_types: TypeConstraints,
    vars: Vector<(String, DataType)>,
    var_types: TypeConstraints,
}

impl Scope {
    pub fn empty() -> Scope {
        Scope {
            next_name: 0,
            consts: Vector::new(),
            const_types: TypeConstraints::empty(),
            vars: Vector::new(),
            var_types: TypeConstraints::empty(),
        }
    }

    pub fn has_vars(&self) -> bool {
        !self.vars.is_empty()
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
        self.consts.push_back_mut((name, data_type));
        self.const_types.insert(data_type);
    }

    pub fn insert_var(&mut self, name: String, data_type: DataType) {
        self.vars.push_back_mut((name, data_type));
        self.var_types.insert(data_type);
    }

    pub fn next_name(&mut self) -> String {
        let next = self.next_name;
        self.next_name += 1;
        format!("var_{}", next)
    }

    pub fn intersects(&self, constraints: &TypeConstraints) -> bool {
        constraints.intersects(&self.const_types.union(&self.var_types))
    }
}
