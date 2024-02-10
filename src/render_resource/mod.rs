use piet_wgpu_derive::{Deref, DerefMut};
use std::sync::Arc;

mod batched_uniform_buffer;
mod bind_group;
mod bind_group_entries;
mod bind_group_layout;
mod buffer;
mod buffer_vec;
mod device;
mod gpu_array_buffer;
mod pipeline;
mod pipeline_cache;
mod pipeline_specializer;
pub mod resource_macros;
mod shader;
mod storage_buffer;
mod texture;
mod uniform_buffer;

pub use bind_group::*;
pub use bind_group_entries::*;
pub use bind_group_layout::*;
pub use buffer::*;
pub use buffer_vec::*;
pub use device::*;
pub use gpu_array_buffer::*;

pub use pipeline::*;
pub use pipeline_cache::*;
pub use pipeline_specializer::*;
pub use shader::*;
pub use storage_buffer::*;
pub use texture::*;
pub use uniform_buffer::*;
use wgpu::Queue;

/// This queue is used to enqueue tasks for the GPU to execute asynchronously.
#[derive(Clone, Deref, DerefMut)]
pub struct RenderQueue(pub Arc<Queue>);
