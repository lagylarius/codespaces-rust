use winit::window::Window;


use std::sync::Arc;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder
};

// use winit::application::ApplicationHandler;

pub struct State {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,
    surface_texture: Option<wgpu::SurfaceTexture>,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    pub event_loop: Option<EventLoop<()>>
}

pub enum LoopEvent {
    Render,
    OnResizing(winit::dpi::PhysicalSize<u32>),
    OnMouseMove(f64, f64),
}


impl State {

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        #[cfg(not(target_arch = "wasm32"))] { // on wasm this is handled by the browser and causes a feedback loop
            self.surface.configure(&self.device, &self.config)
        };
    }

    pub fn run<F: FnMut(LoopEvent)>(eloop: EventLoop<()>,mut update: F) {
        let _ = eloop.run(move |event, control_flow| {
            control_flow.set_control_flow(ControlFlow::Poll);
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => control_flow.exit(),
                        WindowEvent::Resized(size) => update(LoopEvent::OnResizing(size)),
                        WindowEvent::CursorMoved { position, .. } => update(LoopEvent::OnMouseMove(position.x, position.y)),
                        _ => {}
                    }
                    
                }
                Event::AboutToWait => {
                    update(LoopEvent::Render);
                }
                _ => {}
            }
        });
    }
    fn get_texture_surface(surface: &wgpu::Surface) -> Option<wgpu::SurfaceTexture> {
        let frame = surface.get_current_texture();

        match frame {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => return Some(t),

            wgpu::CurrentSurfaceTexture::Timeout => {
                panic!("Surface timeout")
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                return None
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("Surface lost")
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                panic!("Surface occluded")
            }
            _ => {
                panic!("Error fetching texture from surface")
            }
        };
    }

    pub fn depth_view(&mut self) -> wgpu::TextureView {
        let depth_texture = self.device.clone().create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&Default::default());
        return depth_view;
    }

    pub fn start_draw(&mut self) -> wgpu::TextureView {
        let surface_texture = loop {
            match Self::get_texture_surface(&self.surface) {
                Some(t) => break t,
                None => self.surface.configure(&self.device, &self.config),
            }
        };
        let size = surface_texture.texture.size();
        self.config.width = size.width;
        self.config.height = size.height;
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.surface_texture = Some(surface_texture);



        view
    }

    pub fn end_draw(&mut self) {
        self.surface_texture.take().unwrap().present();
        self.window.request_redraw();
    }


    pub async fn initialize_environment() -> Self {
        let event_loop = EventLoop::new().unwrap();

        // let window = WindowBuilder::new()
        //     .with_title("wgpu app")
        //     // .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        //     .build(&event_loop)
        //     .unwrap();

        
        // #[cfg(target_arch = "wasm32")] {
        //     use winit::platform::web::WindowExtWebSys;

        //     let canvas = window.canvas().unwrap();
        //     canvas.set_width(600);
        //     canvas.set_height(400);

        //     canvas.set_id("wgpu-canvas");
            
        //     let node = canvas.into(); 

        //     web_sys::window()
        //         .unwrap()
        //         .document()
        //         .unwrap()
        //         .body()
        //         .unwrap()
        //         .append_child(&node)
        //         .unwrap();
        // }

        let window = {#[cfg(target_arch = "wasm32")] {
                use wasm_bindgen::JsCast;
                use winit::platform::web::WindowBuilderExtWebSys;
                
                let canvas = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("canvas")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .unwrap();
                
                WindowBuilder::new()
                    .with_canvas(Some(canvas))
                    .build(&event_loop)
                    .unwrap()
            }
            #[cfg(not(target_arch = "wasm32"))] {
                WindowBuilder::new()
                    .with_title("wgpu app")
                    .build(&event_loop)
                    .unwrap()
            }
        };

        const FULLSCREEN: bool = false;

        if FULLSCREEN {
            #[cfg(not(target_arch = "wasm32"))] {
                let monitor = window.current_monitor().unwrap();
                let video_mode = monitor.video_modes().next().unwrap();
                window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(video_mode)));
            }
            #[cfg(target_arch = "wasm32")] {
                use winit::platform::web::WindowExtWebSys;
                let canvas = window.canvas().unwrap();
                canvas.request_fullscreen().unwrap();
            }
        }


        let window = std::sync::Arc::new(window);

        // let size = window.inner_size();
        #[cfg(target_arch = "wasm32")]
        let size = winit::dpi::PhysicalSize::new(600, 400);
        #[cfg(not(target_arch = "wasm32"))]
        let size = window.inner_size();

        // // 1. Instance
        let instance = wgpu::Instance::default();

        // // 2. Surface 
        let surface = instance
            .create_surface(window.clone())
            .unwrap();

        // // 3. Adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // // 4. Device + Queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .unwrap();


        // // 5. Surface format (your CANVAS_FORMAT)
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // // 6. Surface config (canvas_context.configure equivalent)
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, //Vsync
            // present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);


        log_print!("surface format: {:?}", config.format);
        log_print!("surface size: {}x{}", config.width, config.height);

        
        log_print!("Adapter: {:?}", adapter.get_info());

        Self {
            window: window,
            surface,
            device: Arc::new(device),
            queue: Arc::new(queue),
            config,
            size,
            surface_texture: None,
            surface_format: format,

            event_loop: Some(event_loop)
        }
    }
}