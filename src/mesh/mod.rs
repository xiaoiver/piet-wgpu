use thiserror::Error;
use wgpu::{VertexAttribute, VertexFormat};

use crate::{render_resource::VertexBufferLayout, Hashed};

#[derive(Debug, Clone)]
pub struct MeshVertexAttribute {
    /// The friendly name of the vertex attribute
    pub name: &'static str,

    /// The _unique_ id of the vertex attribute. This will also determine sort ordering
    /// when generating vertex buffers. Built-in / standard attributes will use "close to zero"
    /// indices. When in doubt, use a random / very large usize to avoid conflicts.
    pub id: MeshVertexAttributeId,

    /// The format of the vertex attribute.
    pub format: VertexFormat,
}

impl MeshVertexAttribute {
    pub const fn new(name: &'static str, id: usize, format: VertexFormat) -> Self {
        Self {
            name,
            id: MeshVertexAttributeId(id),
            format,
        }
    }

    pub const fn at_shader_location(&self, shader_location: u32) -> VertexAttributeDescriptor {
        VertexAttributeDescriptor::new(shader_location, self.id, self.name)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MeshVertexAttributeId(usize);

impl From<MeshVertexAttribute> for MeshVertexAttributeId {
    fn from(attribute: MeshVertexAttribute) -> Self {
        attribute.id
    }
}

pub type MeshVertexBufferLayout = Hashed<InnerMeshVertexBufferLayout>;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct InnerMeshVertexBufferLayout {
    attribute_ids: Vec<MeshVertexAttributeId>,
    layout: VertexBufferLayout,
}

impl InnerMeshVertexBufferLayout {
    #[inline]
    pub fn contains(&self, attribute_id: impl Into<MeshVertexAttributeId>) -> bool {
        self.attribute_ids.contains(&attribute_id.into())
    }

    #[inline]
    pub fn attribute_ids(&self) -> &[MeshVertexAttributeId] {
        &self.attribute_ids
    }

    #[inline]
    pub fn layout(&self) -> &VertexBufferLayout {
        &self.layout
    }

    pub fn get_layout(
        &self,
        attribute_descriptors: &[VertexAttributeDescriptor],
    ) -> Result<VertexBufferLayout, MissingVertexAttributeError> {
        let mut attributes = Vec::with_capacity(attribute_descriptors.len());
        for attribute_descriptor in attribute_descriptors {
            if let Some(index) = self
                .attribute_ids
                .iter()
                .position(|id| *id == attribute_descriptor.id)
            {
                let layout_attribute = &self.layout.attributes[index];
                attributes.push(VertexAttribute {
                    format: layout_attribute.format,
                    offset: layout_attribute.offset,
                    shader_location: attribute_descriptor.shader_location,
                });
            } else {
                return Err(MissingVertexAttributeError {
                    id: attribute_descriptor.id,
                    name: attribute_descriptor.name,
                    pipeline_type: None,
                });
            }
        }

        Ok(VertexBufferLayout {
            array_stride: self.layout.array_stride,
            step_mode: self.layout.step_mode,
            attributes,
        })
    }
}

#[derive(Error, Debug)]
#[error("Mesh is missing requested attribute: {name} ({id:?}, pipeline type: {pipeline_type:?})")]
pub struct MissingVertexAttributeError {
    pub(crate) pipeline_type: Option<&'static str>,
    id: MeshVertexAttributeId,
    name: &'static str,
}

pub struct VertexAttributeDescriptor {
    pub shader_location: u32,
    pub id: MeshVertexAttributeId,
    name: &'static str,
}

impl VertexAttributeDescriptor {
    pub const fn new(shader_location: u32, id: MeshVertexAttributeId, name: &'static str) -> Self {
        Self {
            shader_location,
            id,
            name,
        }
    }
}
