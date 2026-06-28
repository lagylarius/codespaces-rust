use egui::{Color32, Vec2};
use wgpu::TextureView;

use crate::{game::Game, gui::egui_renderer::{EguiRenderer, draw_panel}, state::State};


pub mod egui_renderer;


pub struct PanelInfo {
    pub pos_x: f32,
    pub pos_y: f32,
    pub text: Vec<String>
}

pub fn draw_ui(egui_renderer: &mut EguiRenderer, state: &State, canvas: &TextureView, now: f32,point_info: (u32,u32,u32),panel_info: Option<PanelInfo>) {
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
                let info_str = format!("Points: {} | Objective: {} cards burned in {} burns", 
                                            point_info.0, point_info.1, point_info.2);
                ui.horizontal(|ui| {
                    ui.label(info_str);
                });
            });


    if let Some(mut panel_info) = panel_info {
        let wave_y = (now * 3.0).cos() * 4.0;
        let mut drawpos = (panel_info.pos_x,panel_info.pos_y);
        drawpos = (drawpos.0+4.0,drawpos.1+wave_y);
        
        let screen_width = ctx.screen_rect().width();
        // let max_width = screen_width * 0.33;
        let max_width = (screen_width - drawpos.0).max(1.0).min(screen_width * 0.33);
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
                            ui.label(egui::RichText::new(panel_info.text[0].as_str()).strong());
                            
                            // Instead of ui.separator():
                            let separator_height = 4.0;
                            let separator_width = ui.min_rect().width(); // fixed width
                            let rect = ui.allocate_space(Vec2::new(separator_width, separator_height));

                            // Draw the separator
                            ui.painter().rect_filled(rect.1, 0.0, Color32::WHITE);

                            ui.add_space(4.0);

                            panel_info.text.remove(0);

                            for t in panel_info.text.iter() {
                                if let Some(pos) = t.find(':') {
                                    let (before, after) = t.split_at(pos + 1); // include ':'

                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(egui::RichText::new(before).strong());
                                        render_rich_text(ui, &after[1..]); // rest of text after ':'
                                    });
                                } else {
                                    render_rich_text(ui, t);
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

fn render_rich_text(ui: &mut egui::Ui, text: &str) {
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(start) = rest.find('<') {
            let before = &rest[..start];
            if !before.is_empty() {
                ui.label(before);
            }
            rest = &rest[start..];

            if rest.starts_with("<s>") {
                rest = &rest[3..];
                if let Some(end) = rest.find("</s>") {
                    ui.label(egui::RichText::new(&rest[..end]).strikethrough());
                    rest = &rest[end + 4..];
                } else {
                    ui.label(rest);
                    return;
                }
            } else if rest.starts_with("<c=") {
                let color_start = 3;
                if let Some(close) = rest.find('>') {
                    let color_str = &rest[color_start..close];
                    let color = match color_str {
                        "red" => egui::Color32::RED,
                        "green" => egui::Color32::GREEN,
                        _ => egui::Color32::WHITE,
                    };
                    rest = &rest[close + 1..];
                    if let Some(end) = rest.find("</c>") {
                        ui.label(egui::RichText::new(&rest[..end]).color(color));
                        rest = &rest[end + 4..];
                    } else {
                        ui.label(rest);
                        return;
                    }
                } else {
                    ui.label(rest);
                    return;
                }
            } else {
                // not a recognized tag, emit the '<' and move on
                ui.label("<");
                rest = &rest[1..];
            }
        } else {
            ui.label(rest);
            return;
        }
    }
}