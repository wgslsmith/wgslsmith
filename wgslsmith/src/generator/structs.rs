use ast::{StructDecl, StructMember};
use rand::Rng;

use crate::Options;

use super::cx::Context;

const FIELD_NAMES: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

pub fn gen_struct_decl(
    rng: &mut impl Rng,
    cx: &mut Context,
    options: &Options,
    name: String,
) -> StructDecl {
    let member_count = rng.gen_range(options.min_struct_members..=options.max_struct_members);

    let members = (0..member_count)
        .map(|i| StructMember {
            name: FIELD_NAMES[i as usize].to_owned(),
            data_type: cx.types.get_mut().select(rng),
        })
        .collect();

    StructDecl { name, members }
}
