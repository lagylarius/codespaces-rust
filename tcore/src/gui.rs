use egui::{Color32, Vec2};
use wgpu::TextureView;

use crate::{game::Game, gui::egui_renderer::{EguiRenderer, draw_panel}, state::State};


pub mod egui_renderer;


pub fn draw_ui(egui_renderer: &mut EguiRenderer, state: &State, canvas: &TextureView, g: &Game, now: f32,mousepos: (f32,f32)) {
    // Build UI
    let width = state.window.inner_size().width;
    let height = state.window.inner_size().height;
    egui_renderer.begin_frame(width, height, 1.0);
    let ctx = egui_renderer.context().clone();

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

    let mut text_vec: Vec<String> = vec!["test123".to_string(),"cosa".to_string()];
    if let Some(mut text_vec) = g.get_str_info() {
        let wave_y = (now * 3.0).cos() * 4.0;
        let mut drawpos = mousepos;
        drawpos = (drawpos.0+4.0,drawpos.1+wave_y);
        
        let screen_width = ctx.screen_rect().width();
        // let max_width = screen_width * 0.33;
        let max_width = (screen_width - drawpos.0).min(screen_width * 0.33);
        egui::Window::new("Info panel")
            .fixed_pos(drawpos)
            .collapsible(false)
            .resizable(false)
            .max_size(egui::Vec2::new(max_width, f32::INFINITY))
            .frame(egui::Frame {
                ..Default::default()
            })
            .title_bar(false)
            .show(&ctx, |ui| {
                let padding = 16.0;

                ui.set_max_width(max_width);

                ui.with_layout(
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                    ui.add_space(padding); //top padding

                    ui.horizontal(|ui| {
                        ui.add_space(padding); // left padding
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(text_vec[0].as_str()).strong());
                            
                            // Instead of ui.separator():
                            let separator_height = 4.0;
                            let separator_width = ui.min_rect().width(); // fixed width
                            let rect = ui.allocate_space(Vec2::new(separator_width, separator_height));

                            // Draw the separator
                            ui.painter().rect_filled(rect.1, 0.0, Color32::WHITE);

                            ui.add_space(4.0);

                            text_vec.remove(0);

                            for t in text_vec.iter() {
                                if let Some(pos) = t.find(':') {
                                    let (before, after) = t.split_at(pos + 1); // include ':'

                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(egui::RichText::new(before).strong());
                                        render_with_strikethrough(ui, &after[1..]); // rest of text after ':'
                                    });
                                } else {
                                    render_with_strikethrough(ui, t);
                                }
                            }
                        });
                        ui.add_space(padding); //right padding
                    });


                    ui.add_space(padding); //bottom padding
                });

                egui_renderer::draw_panel(ui);
            });
    }




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

fn render_with_strikethrough(ui: &mut egui::Ui, text: &str) {
    let mut rest = text;

    while let Some(start) = rest.find("<s>") {
        // normal text before <s>
        let (before, after_start) = rest.split_at(start);
        if !before.is_empty() {
            ui.label(before);
        }

        // skip "<s>"
        let after_start = &after_start[3..];

        if let Some(end) = after_start.find("</s>") {
            let (strike, after_end) = after_start.split_at(end);

            ui.label(egui::RichText::new(strike).strikethrough());

            // skip "</s>"
            rest = &after_end[4..];
        } else {
            // no closing tag → render rest normally
            ui.label(after_start);
            return;
        }
    }

    // remaining text
    if !rest.is_empty() {
        ui.label(rest);
    }
}