use wgpu::TextureView;

use crate::{game::CardArray, gui::egui_renderer::{EguiRenderer, draw_panel}, state::State};


pub mod egui_renderer;


pub fn draw_ui(egui_renderer: &mut EguiRenderer, state: &State, canvas: &TextureView, g: &CardArray) {
    // Build UI
    let width = state.window.inner_size().width;
    let height = state.window.inner_size().height;
    egui_renderer.begin_frame(width, height, 1.0);
    let ctx = egui_renderer.context().clone();

    let max_width = 100.0;

    egui::Window::new("Point info")
            .fixed_pos((10.0,10.0))
            .collapsible(false)
            .resizable(false)
            .max_size(egui::Vec2::new(f32::INFINITY, f32::INFINITY))
            .frame(egui::Frame {
                // fill: egui::Color32::BLACK, // panel background
                ..Default::default()
            })
            .title_bar(false)
            .show(&ctx, |ui| {
                let info = g.get_point_info();
                let info_str = format!("Points: {} | Objective: {} cards burned in {} burns", info.0, info.1, info.2);
                ui.horizontal(|ui| {
                    ui.label(info_str);
                });
            });


    let drawpos = (200.0,200.0);
    egui::Window::new("Glass Marble")
            .fixed_pos(drawpos)
            .collapsible(false)
            .resizable(false)
            .max_size(egui::Vec2::new(f32::INFINITY, 1.0))
            .frame(egui::Frame {
                // fill: egui::Color32::BLACK, // panel background
                ..Default::default()
            })
            .title_bar(false)
            .show(&ctx, |ui| {
                ui.label("hello");
            });

    // egui::Window::new("My Panel")
    //     .frame(egui::Frame::none())
    //     .title_bar(false)
    //     .default_size(egui::Vec2::new(300.0, 200.0))
    //     .show(&ctx, |ui| {
    //         // ui.set_min_size(egui::Vec2::new(300.0, 200.0));
    //         ui.label("hello");
    //         // draw_panel(ui);
    //     });

        // Draw UI onto canvas
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: 1.0,
        };
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("egui encoder"),
        });
        egui_renderer.end_frame_and_draw(
            &state.device,
            &state.queue,
            &mut encoder,
            &canvas,   // the TextureView from state.start_draw()
            screen_descriptor,
        );
        state.queue.submit(std::iter::once(encoder.finish()));
}