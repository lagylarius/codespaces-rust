use std::sync::Arc;

use egui::epaint::text::FontInsert;
use egui::{Context, FontData, FontDefinitions, FontFamily, FontId, LayerId, RawInput, TextStyle, Ui};
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, StoreOp, TextureFormat, TextureView};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor, wgpu};


pub fn draw_panel(ui:&mut Ui) {
            
    let full_rect = ui.min_rect();
    let back_rect = full_rect.clone();

    ui.painter().clone().with_layer_id(LayerId::background()).rect_filled(back_rect.shrink2(egui::vec2(4.0, 0.0)), 0.0, egui::Color32::BLACK);
    ui.painter().clone().with_layer_id(LayerId::background()).rect_filled(back_rect.shrink2(egui::vec2(0.0, 4.0)), 0.0, egui::Color32::BLACK);

    let rect = full_rect.shrink(6.0);
    let stroke_width = 4.0;
    let gap = 4.0; // size of missing corner
    let stroke = egui::Stroke::new(stroke_width, egui::Color32::WHITE);
    let p = ui.painter();

    // Top edge (shortened by gap on left and right)
    let top_rect = egui::Rect::from_min_max(
        rect.left_top() + egui::vec2(gap, 0.0),
        rect.right_top() - egui::vec2(gap, -stroke_width),
    );
    p.rect_filled(top_rect, 0.0, stroke.color);

    // Bottom edge (shortened by gap)
    let bottom_rect = egui::Rect::from_min_max(
        rect.left_bottom() + egui::vec2(gap, -stroke_width),
        rect.right_bottom() - egui::vec2(gap, 0.0),
    );
    p.rect_filled(bottom_rect, 0.0, stroke.color);

    // Left edge (shortened by gap)
    let left_rect = egui::Rect::from_min_max(
        rect.left_top() + egui::vec2(0.0, gap),
        rect.left_bottom() - egui::vec2(-stroke_width, gap),
    );
    p.rect_filled(left_rect, 0.0, stroke.color);

    // Right edge (shortened by gap)
    let right_rect = egui::Rect::from_min_max(
        rect.right_top() + egui::vec2(-stroke_width, gap),
        rect.right_bottom() - egui::vec2(0.0, gap),
    );
    p.rect_filled(right_rect, 0.0, stroke.color);

    let corners = [
        rect.left_top() + egui::vec2(gap, gap),                    // top-left
        rect.right_top() + egui::vec2(-2.0*gap, gap),                  // top-right
        rect.left_bottom() + egui::vec2(gap, -2.0*gap),                // bottom-left
        rect.right_bottom() + egui::vec2(-2.0*gap, -2.0*gap),               // bottom-right
    ];

    for &corner in &corners {
        let block = egui::Rect::from_min_size(corner, egui::vec2(gap, gap));
        p.rect_filled(block, 0.0, stroke.color);
    }
}


pub struct EguiRenderer {
    ctx: Context,
    renderer: Renderer,
    raw_input: RawInput,
    frame_started: bool,
}


impl EguiRenderer {
    pub fn context(&self) -> &Context {
        &self.ctx
    }

    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
    ) -> Self {
        let ctx = Context::default();

        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_mono".to_owned(),
            Arc::new(FontData::from_static(include_bytes!(
                "../../img/04B_03__.ttf"
            ))),
        );
        fonts.families.insert(FontFamily::Monospace, vec!["my_mono".to_owned()]);
        ctx.set_fonts(fonts);
        let mut style = (*ctx.style()).clone();
        for text_style in [
            TextStyle::Small,
            TextStyle::Body,
            TextStyle::Button,
            TextStyle::Heading,
            TextStyle::Name("Heading".into()), // optional, if you added custom styles
        ] {
            style.text_styles.insert(text_style, FontId::new(30.0, FontFamily::Monospace));
        }
        ctx.set_style(style);


        let renderer = Renderer::new(
            device,
            output_color_format,
            RendererOptions::default()
        );

        Self {
            ctx,
            renderer,
            raw_input: RawInput::default(),
            frame_started: false,
        }
    }

    pub fn frame_started(&self ) -> bool {
        self.frame_started
    }

    /// Call this every frame before building UI
    pub fn begin_frame(&mut self, width: u32, height: u32, _pixels_per_point: f32) {
        // Construct RawInput with only screen_rect
        self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(width as f32, height as f32),
        ));

        // For now, we don’t set pixels_per_point here
        let input = std::mem::take(&mut self.raw_input);
        self.ctx.begin_pass(input);

        self.frame_started = true;
    }

    pub fn end_frame_and_draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        if !self.frame_started {
            panic!("begin_frame must be called first");
        }

        let full_output = self.ctx.end_pass();

 
        let height = screen_descriptor.size_in_pixels[1] as f32;
        let mut tris = self.ctx.tessellate(full_output.shapes, self.ctx.pixels_per_point());

        // for clipped in &mut tris {
        //     // Flip the clip rect vertically
        //     let clip = &mut clipped.clip_rect;
        //     let min = clip.min;
        //     let max = clip.max;
        //     clip.min.y = height - max.y;
        //     clip.max.y = height - min.y;

        //     if let egui::epaint::Primitive::Mesh(mesh) = &mut clipped.primitive {
        //         for v in &mut mesh.vertices {
        //             v.pos.y = height - v.pos.y;
        //         }
        //     }
        // }

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, image_delta);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            multiview_mask: None,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer.render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        self.frame_started = false;
    }
}