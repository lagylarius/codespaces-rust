
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use log::info;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;


use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::state::State;

#[macro_export]
macro_rules! log_print {
    ($($arg:tt)*) => {{
        #[cfg(target_arch = "wasm32")]
        {
            log::info!($($arg)*);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            println!($($arg)*);
        }
    }};
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)] pub fn start() { 
    // #[cfg(target_arch = "wasm32")] 
    console_error_panic_hook::set_once(); 
    // //run_app(); 

    // let event_loop = EventLoop::new().unwrap();

    // let window = WindowBuilder::new()
    //     .with_title("WASM Winit App")
    //     .build(&event_loop)
    //     .unwrap();

    console_log::init_with_level(log::Level::Info).unwrap();

    // #[cfg(target_arch = "wasm32")]
    // {
    //     use winit::platform::web::WindowExtWebSys;

    //     let canvas = window.canvas().unwrap();

    //     let node = canvas.into(); 

    //     web_sys::window()
    //         .unwrap()
    //         .document()
    //         .unwrap()
    //         .body()
    //         .unwrap()
    //         .append_child(&node)
    //         .unwrap();

    // event_loop
    //     .run(move |event, control_flow| {
    //         info!("loop");
    //         match event {
    //             winit::event::Event::AboutToWait => {
    //                 info!("wait");
    //                 window.request_redraw();
    //             }
    //             winit::event::Event::WindowEvent { event, .. } => {
    //                 match event {
    //                     winit::event::WindowEvent::RedrawRequested => {
    //                         let size = window.inner_size();
    //                         info!("width: {}, height: {}", size.width, size.height);
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //             _ => {}
    //         }
    //     })
    //     .unwrap();
    // }


    run();
}


mod state;


pub fn run() {
    log_print!("Initializing..");
    let mut mouse_x: f64 = -10000.0;
    let mut mouse_y: f64 = -10000.0;
    let mut state = pollster::block_on(State::initialize_environment());


}