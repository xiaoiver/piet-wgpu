use piet::{kurbo::Rect, Color, RenderContext};
use piet_wgpu::{renderer::WgpuRenderer, Piet};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey::Code},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

fn generate(ctx: &mut impl RenderContext) {
    let rect = Rect::new(0., 0., 100.0, 100.0);
    let color = Color::rgb8(19, 86, 162);
    ctx.fill(rect, &color);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            tracing_wasm::set_as_global_default();
        } else {
            tracing_subscriber::fmt::init()
        }
    }

    // @see https://github.com/nanovis/Wenderer/blob/main/src/main.rs
    let event_loop = EventLoop::new().unwrap();

    let builder = WindowBuilder::new()
        .with_title("piet wgpu example with winit")
        .with_inner_size(PhysicalSize::new(100, 100));
    #[cfg(target_arch = "wasm32")]
    let builder = {
        use winit::platform::web::WindowBuilderExtWebSys;
        builder.with_append(true)
    };
    let window = builder.build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        let canvas = window.canvas().unwrap();
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| {
                let width = win.inner_width().unwrap().as_f64().unwrap() as u32;
                let height = win.inner_height().unwrap().as_f64().unwrap() as u32;
                let factor = window.scale_factor();
                let logical = LogicalSize { width, height };
                let PhysicalSize { width, height }: PhysicalSize<u32> = logical.to_physical(factor);
                window.request_inner_size(PhysicalSize::new(width, height));
                win.document()
            })
            .and_then(|doc| {
                let body = doc.body()?;
                let canvas = web_sys::Element::from(canvas);
                body.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("couldn't append canvas to document body");
    }

    let mut renderer = WgpuRenderer::new(
        &window,
        window.inner_size().width,
        window.inner_size().height,
    )
    .await;
    let mut rc = Piet::new(&renderer);
    generate(&mut rc);
    rc.finish().unwrap();
    std::mem::drop(rc);

    event_loop
        .run(|event, event_loop_window_target| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match &event {
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize((*physical_size).width, (*physical_size).height)
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        renderer.resize(window.inner_size().width, window.inner_size().height);
                    }
                    WindowEvent::CloseRequested => event_loop_window_target.exit(),
                    WindowEvent::KeyboardInput { event, .. } => {
                        if renderer.input() {
                            window.request_redraw();
                            return;
                        }
                        if event.state.is_pressed() {
                            match event.physical_key {
                                Code(KeyCode::Escape) => {
                                    event_loop_window_target.exit();
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        renderer.update();
                        match renderer.render() {
                            Ok(_) => {}
                            // Recreate the swap_chain if lost
                            Err(wgpu::SurfaceError::Lost) => {
                                renderer.resize(renderer.config.width, renderer.config.height)
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop_window_target.exit(),
                            Err(e) => eprintln!("Some unhandled error {:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .unwrap();
}

fn main() {
    pollster::block_on(run());
}
