use std::rc::Rc;

use ast::{StructDecl, StructMember};
use rand::Rng;

const FIELD_NAMES: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

impl<'a> super::Generator<'a> {
    pub fn gen_struct(&mut self, name: String) -> Rc<StructDecl> {
        let member_count = self
            .rng
            .gen_range(self.options.min_struct_members..=self.options.max_struct_members);

        let members = (0..member_count)
            .map(|i| {
                StructMember::new(
                    FIELD_NAMES[i as usize].to_owned(),
                    self.cx.types.get_mut().select(self.rng),
                )
            })
            .collect();

        StructDecl::new(name, members)
    }
}
