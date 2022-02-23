use ast::types::DataType;
use rand::prelude::IteratorRandom;
use rand::Rng;
use rpds::{HashTrieMap, Vector};

#[derive(Clone, Debug)]
pub struct Scope {
    next_name: u32,
    symbols: HashTrieMap<DataType, Vec<(String, DataType)>>,
    consts: Vector<(String, DataType)>,
    vars: Vector<(String, DataType)>,
}

impl Scope {
    pub fn empty() -> Scope {
        Scope {
            next_name: 0,
            symbols: HashTrieMap::new(),
            consts: Vector::new(),
            vars: Vector::new(),
        }
    }

    pub fn has_vars(&self) -> bool {
        !self.vars.is_empty()
    }

    pub fn of_type(&self, ty: &DataType) -> &[(String, DataType)] {
        self.symbols.get(ty).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn choose_var(&self, rng: &mut impl Rng) -> (&String, &DataType) {
        self.vars.iter().choose(rng).map(|(n, t)| (n, t)).unwrap()
    }

    pub fn insert_let(&mut self, name: String, data_type: DataType) {
        self.insert_symbol(&data_type, &name, &data_type);
        self.consts.push_back_mut((name, data_type));
    }

    pub fn insert_var(&mut self, name: String, data_type: DataType) {
        self.insert_symbol(&data_type, &name, &data_type);
        self.vars.push_back_mut((name, data_type));
    }

    fn insert_symbol(&mut self, key: &DataType, name: &str, ty: &DataType) {
        let symbols = if let Some(symbols) = self.symbols.get_mut(key) {
            symbols
        } else {
            self.symbols.insert_mut(key.clone(), Vec::new());
            self.symbols.get_mut(key).unwrap()
        };

        symbols.push((name.to_owned(), ty.clone()));

        if let DataType::Vector(n, t) = key {
            if *n == 2 {
                self.insert_symbol(&DataType::Scalar(*t), name, ty);
            } else {
                self.insert_symbol(&DataType::Vector(n - 1, *t), name, ty);
            }
        }
    }

    pub fn next_var(&mut self) -> String {
        let next = self.next_name;
        self.next_name += 1;
        format!("var_{}", next)
    }
}
