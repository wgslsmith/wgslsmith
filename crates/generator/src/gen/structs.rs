use std::rc::Rc;

use ast::types::DataType;
use ast::{StructDecl, StructMember, StructMemberAttr};
use rand::Rng;

use super::cx::SelectionFilter;

const FIELD_NAMES: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StructKind {
    Default,
    HostShareable,
    UniformBuffer,
}

impl super::Generator<'_> {
    pub fn gen_struct(&mut self, name: String) -> Rc<StructDecl> {
        self.gen_struct_with(name, StructKind::Default)
    }

    pub fn gen_struct_with(&mut self, name: String, kind: StructKind) -> Rc<StructDecl> {
        let member_count = self
            .rng
            .gen_range(self.options.min_struct_members..=self.options.max_struct_members);

        let filter = match kind {
            StructKind::Default => SelectionFilter::Any,
            StructKind::HostShareable => SelectionFilter::HostShareable,
            StructKind::UniformBuffer => SelectionFilter::Uniform,
        };

        let mut members = (0..member_count)
            .map(|i| {
                StructMember::new(
                    vec![],
                    FIELD_NAMES[i as usize].to_owned(),
                    self.cx.types.select_with_filter(self.rng, filter),
                )
            })
            .collect::<Vec<_>>();

        if matches!(kind, StructKind::HostShareable | StructKind::UniformBuffer) {
            for member in &mut members {
                if let DataType::Struct(_) = member.data_type {
                    Rc::get_mut(member)
                        .unwrap()
                        .attrs
                        .push(StructMemberAttr::Align(16));
                }
            }
        }

        StructDecl::new(name, members)
    }
}
