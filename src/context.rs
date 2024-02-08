use crate::renderer::WgpuRenderer;
use crate::text::{WgpuText, WgpuTextLayout};
use piet::{
    kurbo::{Affine, Point, Rect, Shape, Size},
    Color, Error, FixedGradient, Image, ImageFormat, InterpolationMode, IntoBrush, RenderContext,
    StrokeStyle,
};
use std::borrow::Cow;
use tracing::info;

#[doc(hidden)]
#[derive(Clone)]
pub struct WgpuImage;

#[derive(Clone)]
#[doc(hidden)]
pub enum Brush {
    Solid(Color),
}

impl<'a> IntoBrush<WgpuRenderContext<'a>> for Brush {
    fn make_brush<'b>(
        &'b self,
        _piet: &mut WgpuRenderContext,
        _bbox: impl FnOnce() -> Rect,
    ) -> Cow<'b, Brush> {
        Cow::Borrowed(self)
    }
}

impl Image for WgpuImage {
    fn size(&self) -> Size {
        todo!()
    }
}

pub struct WgpuRenderContext<'a> {
    pub(crate) renderer: &'a mut WgpuRenderer,

    /// The context state stack. There is always at least one, until finishing.
    ctx_stack: Vec<CtxState>,
}

#[derive(Default)]
struct CtxState {
    transform: Affine,
}

impl<'a> WgpuRenderContext<'a> {
    pub fn new(renderer: &'a mut WgpuRenderer) -> Self {
        let mut context = Self {
            renderer,
            ctx_stack: vec![CtxState::default()],
        };
        context
    }

    fn pop_state(&mut self) {
        // This is an unwrap because we protect the invariant.
        let old_state = self.ctx_stack.pop().unwrap();
    }
}

impl<'a> RenderContext for WgpuRenderContext<'a> {
    type Brush = Brush;
    type Image = WgpuImage;
    type Text = WgpuText;
    type TextLayout = WgpuTextLayout;

    fn status(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn solid_brush(&mut self, color: Color) -> Self::Brush {
        Brush::Solid(color)
    }

    fn gradient(&mut self, _gradient: impl Into<FixedGradient>) -> Result<Self::Brush, Error> {
        todo!()
    }

    fn clear(&mut self, _: impl Into<Option<Rect>>, _color: Color) {}

    fn stroke(&mut self, _shape: impl Shape, _brush: &impl IntoBrush<Self>, _width: f64) {}

    fn stroke_styled(
        &mut self,
        _shape: impl Shape,
        _brush: &impl IntoBrush<Self>,
        _width: f64,
        _style: &StrokeStyle,
    ) {
    }

    fn fill(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {
        if let Some(rect) = shape.as_rect() {
            let brush = brush.make_brush(self, || shape.bounding_box()).into_owned();
            let Brush::Solid(color) = brush;

            info!("fill rect {:?}", color);
        }
    }

    fn fill_even_odd(&mut self, _shape: impl Shape, _brush: &impl IntoBrush<Self>) {}

    fn clip(&mut self, _shape: impl Shape) {}

    fn text(&mut self) -> &mut Self::Text {
        // &mut self.0
        todo!()
    }

    fn draw_text(&mut self, _layout: &Self::TextLayout, _pos: impl Into<Point>) {}

    fn save(&mut self) -> Result<(), Error> {
        let new_state = CtxState {
            transform: self.current_transform(),
        };
        self.ctx_stack.push(new_state);
        Ok(())
    }

    fn restore(&mut self) -> Result<(), Error> {
        if self.ctx_stack.len() <= 1 {
            return Err(Error::StackUnbalance);
        }
        self.pop_state();
        Ok(())
    }

    // Discussion question: should this subsume EndDraw, with BeginDraw on
    // D2DRenderContext creation? I'm thinking not, as the shell might want
    // to do other stuff, possibly related to incremental paint.
    fn finish(&mut self) -> Result<(), Error> {
        if self.ctx_stack.len() != 1 {
            return Err(Error::StackUnbalance);
        }
        self.pop_state();
        Ok(())
        // std::mem::replace(&mut self.err, Ok(()))
    }

    fn transform(&mut self, transform: Affine) {
        self.ctx_stack.last_mut().unwrap().transform *= transform;
    }

    fn current_transform(&self) -> Affine {
        // This is an unwrap because we protect the invariant.
        self.ctx_stack.last().unwrap().transform
    }

    fn capture_image_area(&mut self, _src_rect: impl Into<Rect>) -> Result<Self::Image, Error> {
        Ok(WgpuImage)
    }

    #[allow(clippy::identity_op)]
    fn make_image_with_stride(
        &mut self,
        _width: usize,
        _height: usize,
        _stride: usize,
        _buf: &[u8],
        _format: ImageFormat,
    ) -> Result<Self::Image, Error> {
        Ok(WgpuImage)
    }

    fn draw_image(
        &mut self,
        _image: &Self::Image,
        _dst_rect: impl Into<Rect>,
        _interp: InterpolationMode,
    ) {
    }
    fn draw_image_area(
        &mut self,
        _image: &Self::Image,
        _src_rect: impl Into<Rect>,
        _dst_rect: impl Into<Rect>,
        _interp: InterpolationMode,
    ) {
    }

    fn blurred_rect(&mut self, _rect: Rect, _blur_radius: f64, _brush: &impl IntoBrush<Self>) {}
}
