use ast::{Module, StorageClass, VarQualifier};
pub use types::{PipelineDescription, PipelineResource, ResourceData, ResourceKind};

pub fn reflect(
    module: &Module,
    mut init: impl FnMut(ResourceData<'_>) -> Option<Vec<u8>>,
) -> PipelineDescription {
    let mut resources = vec![];

    for var in &module.vars {
        if let Some(VarQualifier { storage_class, .. }) = &var.qualifier {
            let kind = match storage_class {
                StorageClass::Uniform => ResourceKind::UniformBuffer,
                StorageClass::Storage => ResourceKind::StorageBuffer,
                _ => continue,
            };

            let type_desc =
                common::Type::try_from(&var.data_type).expect("invalid type for pipeline resource");

            let group = var
                .group_index()
                .expect("resource variable must have group attribute");

            let binding = var
                .binding_index()
                .expect("resource variable must have binding attribute");

            let init = init(ResourceData {
                name: &var.name,
                group,
                binding,
            })
            .map(|mut init| {
                init.resize(type_desc.buffer_size() as usize, 0);
                init
            });

            resources.push(PipelineResource {
                name: var.name.clone(),
                kind,
                group,
                binding,
                init,
                type_desc,
            })
        }
    }

    PipelineDescription { resources }
}
