use wasm_bindgen::prelude::*;

use log::info;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[wasm_bindgen(start)] pub fn start() { 
    #[cfg(target_arch = "wasm32")] 
    console_error_panic_hook::set_once(); 
    //run_app(); 

    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("WASM Winit App")
        .build(&event_loop)
        .unwrap();

    console_log::init_with_level(log::Level::Info).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas().unwrap();

        let node = canvas.into(); 

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&node)
            .unwrap();

        //let size = window.inner_size();

        //window.request_redraw();

    event_loop
        .run(move |event, control_flow| {
            info!("loop");
            match event {
                winit::event::Event::AboutToWait => {
                    info!("wait");
                    window.request_redraw();
                }
                winit::event::Event::WindowEvent { event, .. } => {
                    match event {
                        winit::event::WindowEvent::RedrawRequested => {
                            let size = window.inner_size();
                            info!("width: {}, height: {}", size.width, size.height);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .unwrap();
    }
}


#[wasm_bindgen]
pub fn greet() -> String {
    "hello".to_string()
}