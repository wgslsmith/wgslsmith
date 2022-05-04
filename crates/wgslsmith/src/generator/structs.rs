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
}

impl<'a> super::Generator<'a> {
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
        };

        let mut members = (0..member_count)
            .map(|i| {
                StructMember::new(
                    vec![],
                    FIELD_NAMES[i as usize].to_owned(),
                    self.cx.types.get_mut().select_with_filter(self.rng, filter),
                )
            })
            .collect::<Vec<_>>();

        if kind == StructKind::HostShareable {
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
