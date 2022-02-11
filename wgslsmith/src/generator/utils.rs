use ast::types::DataType;
use rand::prelude::SliceRandom;
use rand::Rng;

pub fn gen_vector_accessor(
    rng: &mut impl Rng,
    vector_type: &DataType,
    target_type: &DataType,
) -> String {
    // Find m (size of src vector) and n (size of target vector).
    let (m, n) = match vector_type {
        DataType::Vector(m, _) => match target_type {
            DataType::Scalar(_) => return "x".to_owned(),
            DataType::Vector(n, _) => (*m, *n),
            DataType::Array(_) => todo!(),
            DataType::User(_) => todo!(),
        },
        _ => unreachable!(),
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