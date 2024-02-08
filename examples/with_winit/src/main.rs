use piet::{kurbo::Rect, Color, RenderContext};
use piet_wgpu::{renderer::WgpuRenderer, Piet};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
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

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    // Winit prevents sizing with CSS, so we have to set
    // the size manually when on web.
    window.set_inner_size(PhysicalSize::new(1000, 1000));

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let body = doc.body()?;
                let canvas = web_sys::Element::from(window.canvas());
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
    let mut rc = Piet::new(&mut renderer);
    generate(&mut rc);
    rc.finish().unwrap();
    std::mem::drop(rc);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !renderer.input() {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(physical_size.width, physical_size.height);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        renderer.resize(new_inner_size.width, new_inner_size.height);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            renderer.update();
            match renderer.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => {
                    renderer.resize(renderer.config.width, renderer.config.height)
                }
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}

fn main() {
    pollster::block_on(run());
}
