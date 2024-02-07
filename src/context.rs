use crate::text::{WgpuText, WgpuTextLayout};
use piet::{
    kurbo::{Affine, Point, Rect, Shape, Size},
    Color, Error, FixedGradient, Image, ImageFormat, InterpolationMode, IntoBrush, RenderContext,
    StrokeStyle,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::borrow::Cow;
use tracing::info;
use wgpu::{
    Adapter, Device, Instance, Limits, Queue, Surface, SurfaceConfiguration, TextureFormat,
};

#[doc(hidden)]
#[derive(Clone)]
pub struct WgpuImage;

#[derive(Clone)]
#[doc(hidden)]
pub enum Brush {
    Solid(Color),
}

impl IntoBrush<WgpuRenderContext> for Brush {
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

pub struct DeviceHandle {
    adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

/// Combination of surface and its configuration.
#[derive(Debug)]
pub struct RenderSurface {
    pub surface: Surface,
    pub config: SurfaceConfiguration,
    pub dev_id: usize,
    pub format: TextureFormat,
}

pub struct WgpuRenderContext {
    pub instance: Instance,
    pub devices: Vec<DeviceHandle>,

    /// The context state stack. There is always at least one, until finishing.
    ctx_stack: Vec<CtxState>,
}

#[derive(Default)]
struct CtxState {
    transform: Affine,
}

impl WgpuRenderContext {
    pub fn new() -> Self {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::default();

        let mut context = Self {
            instance,
            devices: Vec::new(),
            ctx_stack: vec![CtxState::default()],
        };
        context
    }

    fn pop_state(&mut self) {
        // This is an unwrap because we protect the invariant.
        let old_state = self.ctx_stack.pop().unwrap();
    }

    /// Creates a new surface for the specified window and dimensions.
    pub async fn create_surface<W>(
        &mut self,
        window: &W,
        width: u32,
        height: u32,
    ) -> crate::Result<RenderSurface>
    where
        W: HasRawWindowHandle + HasRawDisplayHandle,
    {
        let surface = unsafe { self.instance.create_surface(window) }?;
        let dev_id = self
            .device(Some(&surface))
            .await
            .ok_or("Error creating device")?;

        let device_handle = &self.devices[dev_id];
        let capabilities = surface.get_capabilities(&device_handle.adapter);
        let format = capabilities
            .formats
            .into_iter()
            .find(|it| matches!(it, TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm))
            .expect("surface should support Rgba8Unorm or Bgra8Unorm");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        let surface = RenderSurface {
            surface,
            config,
            dev_id,
            format,
        };
        self.configure_surface(&surface);
        Ok(surface)
    }

    /// Resizes the surface to the new dimensions.
    pub fn resize_surface(&self, surface: &mut RenderSurface, width: u32, height: u32) {
        surface.config.width = width;
        surface.config.height = height;
        self.configure_surface(surface);
    }

    pub fn set_present_mode(&self, surface: &mut RenderSurface, present_mode: wgpu::PresentMode) {
        surface.config.present_mode = present_mode;
        self.configure_surface(surface);
    }

    fn configure_surface(&self, surface: &RenderSurface) {
        let device = &self.devices[surface.dev_id].device;
        // Temporary workaround for https://github.com/gfx-rs/wgpu/issues/4214
        // It's still possible for this to panic if the device is being used on another thread
        // but this unbreaks most current users
        device.poll(wgpu::MaintainBase::Wait);
        surface.surface.configure(device, &surface.config);
    }

    /// Finds or creates a compatible device handle id.
    pub async fn device(&mut self, compatible_surface: Option<&Surface>) -> Option<usize> {
        let compatible = match compatible_surface {
            Some(s) => self
                .devices
                .iter()
                .enumerate()
                .find(|(_, d)| d.adapter.is_surface_supported(s))
                .map(|(i, _)| i),
            None => (!self.devices.is_empty()).then_some(0),
        };
        if compatible.is_none() {
            return self.new_device(compatible_surface).await;
        }
        compatible
    }

    /// Creates a compatible device handle id.
    async fn new_device(&mut self, compatible_surface: Option<&Surface>) -> Option<usize> {
        let adapter =
            wgpu::util::initialize_adapter_from_env_or_default(&self.instance, compatible_surface)
                .await?;
        let features = adapter.features();
        let limits = Limits::default();
        #[allow(unused_mut)]
        let mut maybe_features = wgpu::Features::CLEAR_TEXTURE;
        #[cfg(feature = "wgpu-profiler")]
        {
            maybe_features |= wgpu_profiler::GpuProfiler::ALL_WGPU_TIMER_FEATURES;
        };
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: features & maybe_features,
                    limits,
                },
                None,
            )
            .await
            .ok()?;
        let device_handle = DeviceHandle {
            adapter,
            device,
            queue,
        };
        self.devices.push(device_handle);
        Some(self.devices.len() - 1)
    }
}

impl RenderContext for WgpuRenderContext {
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

            info!("fill rect... {:?}   ", color);
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
