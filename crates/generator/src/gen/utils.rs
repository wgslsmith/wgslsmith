use ast::types::DataType;
use ast::Statement;
use rand::prelude::SliceRandom;
use rand::Rng;

pub fn gen_vector_accessor(rng: &mut impl Rng, size: u8, target_type: &DataType) -> String {
    // Find m (size of src vector) and n (size of target vector).
    let (m, n) = match target_type {
        DataType::Scalar(_) => return "x".to_owned(),
        DataType::Vector(n, _) => (size, *n),
        _ => panic!("vector component type must be a scalar"),
    };

    assert!((2..=4).contains(&m));
    assert!((2..=4).contains(&n));

    let mut accessor = String::new();

    // Possible accessors we can use depending on the size of the src vector.
    let possible_accessors: &[&str] = match m {
        2 => &["x", "y"],
        3 => &["x", "y", "z"],
        4 => &["x", "y", "z", "w"],
        _ => unreachable!(),
    };

    // Generate a sequence of accessors depending on the size of the target vector.
    for _ in 0..n {
        accessor += possible_accessors.choose(rng).copied().unwrap();
    }

    accessor
}

/// Computes the types which are accessible through this type via member access, etc.
pub fn accessible_types_of(ty: &DataType) -> Vec<DataType> {
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
        DataType::Array(ty, _) => vec![(**ty).clone()],
        DataType::Struct(decl) => decl.accessible_types().cloned().collect(),
        DataType::Ptr(view) | DataType::Ref(view) => accessible_types_of(&view.inner),
    }
}

pub fn is_terminal_stmt<'a>(last_statement: impl Into<Option<&'a Statement>>) -> bool {
    matches!(
        last_statement.into(),
        Some(
            Statement::Return(_) | Statement::Break | Statement::Continue | Statement::Fallthrough
        )
    )
}
