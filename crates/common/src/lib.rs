use serde::Deserialize;

#[derive(Debug)]
pub enum ScalarType {
    I32,
    U32,
    F32,
    AU32,
    AI32,
}

#[derive(Debug)]
pub enum VectorSize {
    N2,
    N3,
    N4,
}

#[derive(Debug)]
pub struct StructMember {
    pub name: String,
    pub type_desc: Type,
}

#[derive(Debug)]
pub enum Type {
    Scalar {
        scalar_type: ScalarType,
    },
    Vector {
        size: VectorSize,
        scalar_type: ScalarType,
    },
    Array {
        size: u32,
        element_type: Box<Type>,
    },
    Struct {
        members: Vec<StructMember>,
    },
}

#[derive(Debug, Deserialize)]
pub struct BufferInit {
  pub data: Vec<u8>,
  pub size: Option<u32>,
}

fn aligned(size: u32, alignment: u32) -> u32 {
    ((size + (alignment - 1)) / alignment) * alignment
}

impl Type {
    pub fn buffer_size(&self) -> u32 {
        aligned(self.size(), self.alignment())
    }

    pub fn size(&self) -> u32 {
        match self {
            Type::Scalar { .. } => 4,
            Type::Vector { size, .. } => match size {
                VectorSize::N2 => 8,
                VectorSize::N3 => 12,
                VectorSize::N4 => 16,
            },
            Type::Array { size, element_type } => {
                size * aligned(element_type.size(), element_type.alignment())
            }
            Type::Struct { members } => {
                let mut size = 0;
                let mut alignment = 0;

                for member in members {
                    let member_alignment = member.type_desc.alignment();
                    let member_size = member.type_desc.size();
                    alignment = u32::max(alignment, member.type_desc.alignment());
                    size = aligned(size, member_alignment) + member_size;
                }

                aligned(size, alignment)
            }
        }
    }

    pub fn alignment(&self) -> u32 {
        match self {
            Type::Scalar { .. } => 4,
            Type::Vector { size, .. } => match size {
                VectorSize::N2 => 8,
                VectorSize::N3 => 16,
                VectorSize::N4 => 16,
            },
            Type::Array { element_type, .. } => element_type.alignment(),
            Type::Struct { members } => members
                .iter()
                .map(|it| it.type_desc.alignment())
                .max()
                .expect("struct must have at least one member"),
        }
    }

    pub fn ranges(&self) -> Vec<(usize, usize)> {
        let mut ranges = vec![];

        fn collect_ranges(acc: &mut Vec<(usize, usize)>, mut offset: u32, type_desc: &Type) {
            match type_desc {
                Type::Scalar { .. } => acc.push((offset as _, type_desc.size() as _)),
                Type::Vector { .. } => acc.push((offset as _, type_desc.size() as _)),
                Type::Array { size, element_type } => {
                    let element_size = element_type.size();
                    let alignment = element_type.alignment();
                    for _ in 0..*size {
                        collect_ranges(acc, offset, element_type);
                        offset = aligned(offset + element_size, alignment);
                    }
                }
                Type::Struct { members } => {
                    for member in members {
                        let alignment = member.type_desc.alignment();
                        offset = aligned(offset, alignment);
                        collect_ranges(acc, offset, &member.type_desc);
                        let size = member.type_desc.size();
                        offset += size;
                    }
                }
            }
        }

        collect_ranges(&mut ranges, 0, self);

        ranges
    }
}

impl TryFrom<&ast::ScalarType> for ScalarType {
    type Error = &'static str;

    fn try_from(value: &ast::ScalarType) -> Result<Self, Self::Error> {
        match value {
            ast::ScalarType::Bool => Err("bool is not allowed"),
            ast::ScalarType::I32 => Ok(ScalarType::I32),
            ast::ScalarType::U32 => Ok(ScalarType::U32),
            ast::ScalarType::F32 => Ok(ScalarType::F32),
            ast::ScalarType::AU32 => Ok(ScalarType::AU32),
            ast::ScalarType::AI32 => Ok(ScalarType::AI32),
        }
    }
}

impl TryFrom<&ast::DataType> for Type {
    type Error = &'static str;

    fn try_from(value: &ast::DataType) -> Result<Self, Self::Error> {
        match value {
            ast::DataType::Scalar(scalar) => Ok(Type::Scalar {
                scalar_type: scalar.try_into()?,
            }),
            ast::DataType::Vector(n, scalar) => Ok(Type::Vector {
                size: match n {
                    2 => VectorSize::N2,
                    3 => VectorSize::N3,
                    4 => VectorSize::N4,
                    _ => return Err("invalid vector size"),
                },
                scalar_type: scalar.try_into()?,
            }),
            ast::DataType::Array(inner, size) => Ok(Type::Array {
                size: size.ok_or("struct member runtime sized arrays are not supported")?,
                element_type: Box::new(inner.as_ref().try_into()?),
            }),
            ast::DataType::Struct(decl) => {
                let mut members = vec![];

                for member in &decl.members {
                    let type_desc = Type::try_from(&member.data_type)?;

                    members.push(StructMember {
                        name: member.name.clone(),
                        type_desc,
                    });
                }

                Ok(Type::Struct { members })
            }
            ast::DataType::Ptr(_) => Err("pointers are not storable"),
            ast::DataType::Ref(_) => Err("references are not storable"),
        }
    }
}
