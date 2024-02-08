//! The wgpu backend for the Piet 2D graphics abstraction.

use context::{WgpuImage, WgpuRenderContext};
use text::{WgpuText, WgpuTextLayout, WgpuTextLayoutBuilder};

mod composer;
mod context;
pub mod renderer;
mod text;

/// The `RenderContext` for the CoreGraphics backend, which is selected.
pub type Piet<'a> = WgpuRenderContext<'a>;

/// The associated brush type for this backend.
///
/// This type matches `RenderContext::Brush`
pub type Brush = context::Brush;

/// The associated text factory for this backend.
///
/// This type matches `RenderContext::Text`
pub type PietText = WgpuText;

/// The associated text layout type for this backend.
///
/// This type matches `RenderContext::Text::TextLayout`
pub type PietTextLayout = WgpuTextLayout;

/// The associated text layout builder for this backend.
///
/// This type matches `RenderContext::Text::TextLayoutBuilder`
pub type PietTextLayoutBuilder = WgpuTextLayoutBuilder;

/// The associated image type for this backend.
///
/// This type matches `RenderContext::Image`
pub type PietImage = WgpuImage;
