use std::iter;

use ast::types::{DataType, MemoryViewType};
use rand::prelude::IteratorRandom;
use rand::Rng;
use rpds::{HashTrieMap, Vector};

use super::utils;

#[derive(Clone, Debug)]
pub struct Scope {
    next_name: u32,
    symbols: HashTrieMap<DataType, Vec<(String, DataType)>>,
    mutables: Vector<(String, DataType)>,
    references: Vector<(String, MemoryViewType)>,
}

impl Scope {
    pub fn empty() -> Scope {
        Scope {
            next_name: 0,
            symbols: HashTrieMap::new(),
            mutables: Vector::new(),
            references: Vector::new(),
        }
    }

    pub fn has_mutables(&self) -> bool {
        !self.mutables.is_empty()
    }

    pub fn has_references(&self) -> bool {
        !self.references.is_empty()
    }

    pub fn of_type(&self, ty: &DataType) -> &[(String, DataType)] {
        self.symbols.get(ty).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn choose_mutable(&self, rng: &mut impl Rng) -> (&String, &DataType) {
        #[allow(clippy::map_identity)]
        self.mutables
            .iter()
            .choose(rng)
            .map(|(n, t)| (n, t))
            .unwrap()
    }

    pub fn choose_reference(&self, rng: &mut impl Rng) -> (&String, &MemoryViewType) {
        #[allow(clippy::map_identity)]
        self.references
            .iter()
            .choose(rng)
            .map(|(n, t)| (n, t))
            .unwrap()
    }

    pub fn insert_readonly(&mut self, name: String, data_type: DataType) {
        self.insert_symbol(&name, &data_type);
    }

    pub fn insert_mutable(&mut self, name: String, data_type: DataType) {
        self.insert_symbol(&name, &data_type);
        if let DataType::Ref(mem_view) = &data_type {
            self.references
                .push_back_mut((name.clone(), mem_view.clone()));
        }
        self.mutables.push_back_mut((name, data_type));
    }

    fn insert_symbol(&mut self, name: &str, ty: &DataType) {
        for key in iter::once(ty.clone()).chain(utils::accessible_types_of(ty)) {
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
