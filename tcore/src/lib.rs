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
mod game;

mod gui;


use crate::{game::Game, gui::{draw_ui, egui_renderer}, passes::{Depth, Dispatch, NoDepth, SelfDispatch}, state::{LoopEvent, State}, utils::load_shader_modules};

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


#[cfg(target_arch = "wasm32")]
use web_time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;


pub async fn run() {
    log_print!("Initializing..");
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
    let animation_data_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Card Data Buffer"),
        size: 1024 * 1024,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let hovering_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Card Data Buffer"),
        size: 4*2,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });


    let readback_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback Buffer"),
        size: 4,
        usage: wgpu::BufferUsages::MAP_READ
            | wgpu::BufferUsages::COPY_DST,
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



    //----------------Passes----------------
    let render_background = passes::RenderPass::<Dispatch,NoDepth>::new(state.device.clone(), state.queue.clone(),
    &[
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    ],  shaders.get("bg_vertex").unwrap(),
        shaders.get("bg_fragment").unwrap(),
            state.surface_format);

    let render_cards = passes::RenderPassTexture::<SelfDispatch,Depth>::new(
        state.device.clone(),
        state.queue.clone(),
        &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        shaders.get("card_vertex").unwrap(),
        shaders.get("card_fragment").unwrap(),
        state.surface_format,
        sprite_sheet,  // your wgpu::Texture
    );

    let card_layout_logic = passes::ComputePassCommon::<Dispatch>::new(state.device.clone(),state.queue.clone(),
        &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]
        ,
        shaders.get("card_layout_logic").unwrap());

    //Egui

    let mut mousepos: (f32,f32) = (0.0,0.0);
    let mut egui_renderer = egui_renderer::EguiRenderer::new(
        &state.device,
        state.surface_format,
        None,
        1,
    );
    let ctx = egui_renderer.context();
    ctx.set_visuals(egui::Visuals {
        window_shadow: egui::Shadow::NONE,
        window_fill: egui::Color32::TRANSPARENT,
        ..egui::Visuals::dark()
    });


    //Game loop
    let mut g = Game::new();

    let timestamp_0 = Instant::now();


    let mut last_time = Instant::now();
    let mut frame_count = 0u32;

    State::run( 
        state.event_loop.take().unwrap(),
        move |event| {
            match event {
                LoopEvent::OnResizing(size) => {
                    if size.width > 0 && size.height > 0 {
                        state.resize(size);
                        state.queue.write_buffer(
                            &render_uniform_buffer,
                            0,
                            bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
                        );
                    }
                },
                LoopEvent::Render => { 

                    g.frame_step();

                    g.gpu_sync(&state.queue, &card_data_buffer,&animation_data_buffer);

                    let canvas_texture = state.start_draw();
                    let depth_texture = state.depth_view();
                    let depth = Depth {depth_view: depth_texture};

                    card_layout_logic.do_pass(&[&card_data_buffer,&input_uniform_buffer,&hovering_buffer]);
                    render_background.do_pass(&canvas_texture, &[&render_uniform_buffer],NoDepth);
                    render_cards.do_pass(&canvas_texture, 
                        &[
                            &card_data_buffer, 
                            &render_uniform_buffer, 
                            &input_uniform_buffer,
                            &hovering_buffer,
                            &animation_data_buffer
                        ],
                        depth);

                    draw_ui(&mut egui_renderer, &state, &canvas_texture,&g,timestamp_0.elapsed().as_secs_f32(),mousepos);

                    state.end_draw();

                    frame_count += 1;
                    let elapsed = last_time.elapsed();
                    if elapsed.as_secs_f32() >= 1.0 {
                        log_print!("FPS: {}", frame_count);
                        frame_count = 0;
                        last_time = Instant::now();
                    }
                },
                LoopEvent::OnMouseMove(x, y) => {
                    mousepos = (x as f32,y as f32);
                    state.queue.write_buffer(
                        &input_uniform_buffer,
                        0,
                        bytemuck::cast_slice(&[x as f32, y as f32]),
                    );
                },
                LoopEvent::Click => {
                    let id = {
                        #[cfg(target_arch = "wasm32")] {
                            wasm_bindgen_futures::spawn_local(async move {
                                utils::gpu_readback_byte(&state.device, &state.queue, &hovering_buffer, &readback_buffer,0)
                            });
                        }
                        #[cfg(not(target_arch = "wasm32"))] {
                            pollster::block_on(
                            utils::gpu_readback_byte(&state.device, &state.queue, &hovering_buffer, &readback_buffer,0)
                            )
                        }
                    };
                    g.pick_card(id);
                },
                LoopEvent::ActionF1 => {
                    g.deal();
                }
            }
        },
    );

}