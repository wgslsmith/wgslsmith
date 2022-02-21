use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{StructDecl, StructMember};
use rand::prelude::{SliceRandom, StdRng};
use rand::Rng;

use crate::Options;

const FIELD_NAMES: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

pub struct StructGenerator<'a> {
    rng: &'a mut StdRng,
    #[allow(unused)]
    options: Rc<Options>,
}

impl<'a> StructGenerator<'a> {
    pub fn new(rng: &'a mut StdRng, options: Rc<Options>) -> StructGenerator<'a> {
        StructGenerator { rng, options }
    }

    pub fn gen_decl(&mut self, name: String) -> StructDecl {
        let member_count = self.rng.gen_range(1..=10);
        let members = (0..member_count)
            .map(|i| StructMember {
                name: FIELD_NAMES[i].to_owned(),
                data_type: self.gen_ty(),
            })
            .collect();

        StructDecl { name, members }
    }

    fn gen_ty(&mut self) -> DataType {
        let scalar_ty = [ScalarType::I32, ScalarType::U32, ScalarType::Bool]
            .choose(&mut self.rng)
            .copied()
            .unwrap();

        match self.rng.gen_range(0..2) {
            0 => DataType::Scalar(scalar_ty),
            1 => DataType::Vector(self.rng.gen_range(2..=4), scalar_ty),
            _ => unreachable!(),
        }
    }
}
