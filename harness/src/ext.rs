use ast::types::{DataType, ScalarType};

pub trait DataTypeExt {
    fn size(&self) -> usize;
}

impl DataTypeExt for DataType {
    fn size(&self) -> usize {
        match self {
            DataType::Scalar(ScalarType::I32 | ScalarType::U32) => 4,
            DataType::Vector(n, ScalarType::I32 | ScalarType::U32) => 4 * *n as usize,
            DataType::Struct(decl) => {
                let mut total = 0;
                for member in &decl.members {
                    total += member.data_type.size();
                }
                total
            }
            ty => panic!("cannot compute size for unsupported type `{}`", ty),
        }
    }
}
