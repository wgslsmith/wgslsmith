use common::Type;
use reflection_types::{PipelineDescription, ResourceKind};

pub fn compare<'a>(
    mut buffers: impl Iterator<Item = &'a Vec<Vec<u8>>>,
    pipeline_desc: &PipelineDescription,
    type_descs: &[Type],
) -> bool {
    if let Some(mut prev) = buffers.next() {
        for execution in buffers {
            for (i, (j, _)) in pipeline_desc
                .resources
                .iter()
                .enumerate()
                .filter(|(_, it)| it.kind == ResourceKind::StorageBuffer)
                .enumerate()
            {
                for (offset, size) in type_descs[j].ranges() {
                    let range = offset..(offset + size);
                    if execution[i][range.clone()] != prev[i][range] {
                        return false;
                    }
                }
            }

            prev = execution;
        }
    }

    true
}
