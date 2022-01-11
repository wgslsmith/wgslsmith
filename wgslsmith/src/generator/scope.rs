use ast::types::DataType;
use rand::prelude::IteratorRandom;
use rand::Rng;
use rpds::Vector;

#[derive(Clone, Debug)]
pub struct Scope {
    next_name: u32,
    consts: Vector<(String, DataType)>,
    vars: Vector<(String, DataType)>,
    functions: Vector<(String, Vec<DataType>, Option<DataType>)>,
}

impl Scope {
    pub fn empty() -> Scope {
        Scope {
            next_name: 0,
            consts: Vector::new(),
            vars: Vector::new(),
            functions: Vector::new(),
        }
    }

    pub fn has_vars(&self) -> bool {
        !self.vars.is_empty()
    }

    pub fn iter_vars(&self) -> impl Iterator<Item = (&String, &DataType)> {
        self.consts
            .iter()
            .chain(self.vars.iter())
            .map(|(n, t)| (n, t))
    }

    pub fn iter_fns(&self) -> impl Iterator<Item = (&String, &[DataType], Option<&DataType>)> {
        self.functions
            .iter()
            .map(|(n, a, t)| (n, a.as_slice(), t.as_ref()))
    }

    pub fn choose_var(&self, rng: &mut impl Rng) -> (&String, &DataType) {
        self.vars.iter().choose(rng).map(|(n, t)| (n, t)).unwrap()
    }

    pub fn insert_let(&mut self, name: String, data_type: DataType) {
        self.consts.push_back_mut((name, data_type));
    }

    pub fn insert_var(&mut self, name: String, data_type: DataType) {
        self.vars.push_back_mut((name, data_type));
    }

    pub fn insert_fn(&mut self, name: String, args: Vec<DataType>, return_type: Option<DataType>) {
        self.functions.push_back_mut((name, args, return_type));
    }

    pub fn next_var(&mut self) -> String {
        let next = self.next_name;
        self.next_name += 1;
        format!("var_{}", next)
    }
}
