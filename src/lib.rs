//! The wgpu backend for the Piet 2D graphics abstraction.

use context::{WgpuImage, WgpuRenderContext};
use text::{WgpuText, WgpuTextLayout, WgpuTextLayoutBuilder};

mod composer;
mod context;
mod text;

/// The `RenderContext` for the CoreGraphics backend, which is selected.
pub type Piet = WgpuRenderContext;

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

/// Catch-all error type.
pub type Error = Box<dyn std::error::Error>;
/// Specialization of `Result` for our catch-all error type.
pub type Result<T> = std::result::Result<T, Error>;
