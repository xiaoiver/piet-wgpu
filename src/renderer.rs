use super::Result;
use naga_oil::compose::Composer;
use std::iter;
use tracing::info;
use wgpu::{Adapter, Device, Instance, Limits, Queue, Surface, SurfaceConfiguration, TextureFormat};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::composer::init_composer;

pub struct WgpuRenderer {
    // surface: Surface,
    // config: SurfaceConfiguration,
    // composer: Composer,
    // pub size: PhysicalSize<u32>,
    pub instance: Instance,
    pub devices: Vec<DeviceHandle>,
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

impl WgpuRenderer {
    pub fn new() -> Result<Self> {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::default();

        // let adapter = instance
        //     .request_adapter(&wgpu::RequestAdapterOptions {
        //         power_preference: wgpu::PowerPreference::default(),
        //         compatible_surface: Some(&surface),
        //         force_fallback_adapter: false,
        //     })
        //     .await
        //     .unwrap();

        // let (device, queue) = adapter
        //     .request_device(
        //         &wgpu::DeviceDescriptor {
        //             label: None,
        //             features: wgpu::Features::empty(),
        //             // WebGL doesn't support all of wgpu's features, so if
        //             // we're building for the web we'll have to disable some.
        //             limits: if cfg!(target_arch = "wasm32") {
        //                 wgpu::Limits::downlevel_webgl2_defaults()
        //             } else {
        //                 wgpu::Limits::default()
        //             },
        //         },
        //         None, // Trace path
        //     )
        //     .await
        //     .unwrap();

        // let surface_caps = surface.get_capabilities(&adapter);
        // // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // // one will result all the colors comming out darker. If you want to support non
        // // Srgb surfaces, you'll need to account for that when drawing to the frame.
        // let surface_format = surface_caps
        //     .formats
        //     .iter()
        //     .copied()
        //     .find(|f| f.is_srgb())
        //     .unwrap_or(surface_caps.formats[0]);
        // let config = wgpu::SurfaceConfiguration {
        //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        //     format: surface_format,
        //     width: size.width,
        //     height: size.height,
        //     present_mode: surface_caps.present_modes[0],
        //     alpha_mode: surface_caps.alpha_modes[0],
        //     view_formats: vec![],
        //     // desired_maximum_frame_latency: 2,
        // };
        // surface.configure(&device, &config);

        Ok(Self {
            instance,
            devices: Vec::new(),
        })
    }


    /// Creates a new surface for the specified window and dimensions.
    pub async fn create_surface<W>(
        &mut self,
        window: &W,
        width: u32,
        height: u32,
    ) -> Result<RenderSurface>
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

    // pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    //     let output = self.surface.get_current_texture()?;
    //     let view = output
    //         .texture
    //         .create_view(&wgpu::TextureViewDescriptor::default());

    //     let mut encoder = self
    //         .device
    //         .create_command_encoder(&wgpu::CommandEncoderDescriptor {
    //             label: Some("Render Encoder"),
    //         });

    //     // {
    //     //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //     //         label: Some("Render Pass"),
    //     //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //     //             view: &view,
    //     //             resolve_target: None,
    //     //             ops: wgpu::Operations {
    //     //                 load: wgpu::LoadOp::Clear(wgpu::Color {
    //     //                     r: 0.1,
    //     //                     g: 0.2,
    //     //                     b: 0.3,
    //     //                     a: 1.0,
    //     //                 }),
    //     //                 store: wgpu::StoreOp::Store,
    //     //             },
    //     //         })],
    //     //         depth_stencil_attachment: None,
    //     //         occlusion_query_set: None,
    //     //         timestamp_writes: None,
    //     //     });

    //     //     render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    //     //     render_pass.set_pipeline(&self.render_pipeline);
    //     //     render_pass.draw(0..self.num_vertices, 0..1);
    //     // }

    //     self.queue.submit(iter::once(encoder.finish()));
    //     output.present();

    //     Ok(())
    // }

    // #[allow(unused_variables)]
    // pub fn input(&mut self, event: &WindowEvent) -> bool {
    //     false
    // }

    // pub fn update(&mut self) {}
}
