use ast::types::{DataType, ScalarType};
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
            functions: gen_builtin_fns(),
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

fn vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    (2..=4).map(move |n| DataType::Vector(n, ty))
}

fn scalar_and_vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    std::iter::once(DataType::Scalar(ty)).chain(vectors_of(ty))
}

fn gen_builtin_fns() -> Vector<(String, Vec<DataType>, Option<DataType>)> {
    let mut fns = Vector::new();

    for ty in vectors_of(ScalarType::Bool) {
        fns.push_back_mut((
            "all".to_owned(),
            vec![ty.clone()],
            Some(DataType::Scalar(ScalarType::Bool)),
        ));

        fns.push_back_mut((
            "any".to_owned(),
            vec![ty.clone()],
            Some(DataType::Scalar(ScalarType::Bool)),
        ));
    }

    for s_ty in [ScalarType::Bool, ScalarType::I32, ScalarType::U32] {
        for ty in scalar_and_vectors_of(s_ty) {
            fns.push_back_mut((
                "select".to_owned(),
                vec![ty.clone(), ty.clone(), DataType::Scalar(ScalarType::Bool)],
                Some(ty),
            ));
        }

        for n in 2..=4 {
            fns.push_back_mut((
                "select".to_owned(),
                vec![
                    DataType::Vector(n, s_ty),
                    DataType::Vector(n, s_ty),
                    DataType::Vector(n, ScalarType::Bool),
                ],
                Some(DataType::Vector(n, s_ty)),
            ));
        }
    }

    for s_ty in [ScalarType::I32, ScalarType::U32] {
        for ty in scalar_and_vectors_of(s_ty) {
            fns.push_back_mut((
                "clamp".to_owned(),
                vec![ty.clone(), ty.clone(), ty.clone()],
                Some(ty.clone()),
            ));

            // TODO: Uncomment functions below once they've been implemented in naga and tint

            for ident in [
                "abs",
                // "countLeadingZeros",
                // "countOneBits",
                // "countTrailingZeros",
                // "firstBitHigh",
                // "firstBitLow",
                // "reverseBits",
            ] {
                fns.push_back_mut((ident.to_owned(), vec![ty.clone()], Some(ty.clone())));
            }

            // fns.push_back_mut((
            //     "extractBits".to_owned(),
            //     vec![
            //         ty.clone(),
            //         DataType::Scalar(ScalarType::U32),
            //         DataType::Scalar(ScalarType::U32),
            //     ],
            //     Some(ty.clone()),
            // ));

            // fns.push_back_mut((
            //     "insertBits".to_owned(),
            //     vec![
            //         ty.clone(),
            //         ty.clone(),
            //         DataType::Scalar(ScalarType::U32),
            //         DataType::Scalar(ScalarType::U32),
            //     ],
            //     Some(ty.clone()),
            // ));

            for ident in ["max", "min"] {
                fns.push_back_mut((
                    ident.to_owned(),
                    vec![ty.clone(), ty.clone()],
                    Some(ty.clone()),
                ));
            }
        }

        // dot product on integers not implemented in naga:
        //   https://github.com/gfx-rs/naga/issues/1667
        // for ty in vectors_of(s_ty) {
        //     fns.push_back_mut((
        //         "dot".to_owned(),
        //         vec![ty.clone(), ty.clone()],
        //         Some(DataType::Scalar(s_ty)),
        //     ));
        // }
    }

    fns
}
