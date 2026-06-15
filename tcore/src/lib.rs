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


mod state;
mod passes;
mod utils;


use crate::{state::State, utils::load_shader_modules};

#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::prelude::*,
    wasm_bindgen_futures::spawn_local,
    console_error_panic_hook,
    log::info,
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)] pub fn start() { 
    console_error_panic_hook::set_once(); 
    console_log::init_with_level(log::Level::Info).unwrap();
    wasm_bindgen_futures::spawn_local(run());
}


use wgpu::util::DeviceExt;
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};


pub async fn run() {
    log_print!("Initializing..");
    let mut mouse_x: f64 = -10000.0;
    let mut mouse_y: f64 = -10000.0;
    let mut state = State::initialize_environment().await;


    
    //----------------Data----------------
    let size = state.window.inner_size();
    let render_uniform_data: [f32; 2] = [
        size.width as f32,
        size.height as f32,
    ];
    let render_uniform_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Render Uniform Buffer"),
        contents: bytemuck::cast_slice(&render_uniform_data),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let input_uniform_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Input Uniform Buffer"),
        size: 4 * 4,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let card_data_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Card Data Buffer"),
        size: 1024 * 1024,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let sprite_path = {
        #[cfg(target_arch = "wasm32")]
        { "img/sprite.png" }
        #[cfg(not(target_arch = "wasm32"))]
        { "tcore/img/sprite.png" }
    };
    let sprite_sheet = utils::create_texture(state.device.clone(), state.queue.clone(), sprite_path).await;


    //----------------Load shaders----------------
    let shader_dir = {
        #[cfg(target_arch = "wasm32")]
        { "shaders" }
        #[cfg(not(target_arch = "wasm32"))]
        { "tcore/shaders" }
    };
    let shader_files: std::collections::HashMap<&str, String> = [
        ("bg_vertex", "render_background.wgsl"),
        ("bg_fragment", "render_background.wgsl"),
        ("card_vertex", "render_card.wgsl"),
        ("card_fragment", "render_card.wgsl"),
        ("card_layout_logic", "card_layout_logic.wgsl"),
    ]
    .into_iter()
    .map(|(key, file)| (key, format!("{shader_dir}/{file}")))
    .collect();

    let shaders = load_shader_modules(&state.device, &shader_files).await.unwrap();


}