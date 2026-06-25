use std::sync::Arc;

use crate::game::{AnimationQueue, card::Card};


pub struct GPUContext {
    flat: Vec<GpuCard>,
    total_cards: u32,
    flat_animations: Vec<GpuAnimation>,
    total_animations: u32
}

impl GPUContext {
    pub fn new() -> Self {
        Self { 
            total_animations: 0,
            total_cards: 0,
            flat: Vec::new(),
            flat_animations: Vec::new(),
        }
    }
    pub fn flush_to_gpu(&self, queue: &Arc<wgpu::Queue>, buffer: &wgpu::Buffer, animation_buffer: &wgpu::Buffer) {
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&self.total_cards));
        queue.write_buffer(buffer, 4, bytemuck::bytes_of(&self.total_cards.div_ceil(256)));
        queue.write_buffer(buffer, 16, bytemuck::cast_slice(&self.flat));

        queue.write_buffer(animation_buffer, 0, bytemuck::cast_slice(&self.flat_animations));
    }
    pub fn push_cards(&mut self, cards: &Vec<Card>, tableau_idx: u32,animation_queue: &AnimationQueue) {
        self.total_cards += cards.len() as u32;
        for (stack_idx, card) in cards.iter().enumerate() {
            let animation_id = if let Some(animation) = animation_queue.animation_for_card(card) {
                self.total_animations += 1;
                self.flat_animations.push(GpuAnimation { 
                    previous_tableau: animation.previous_tableau, 
                    previous_stack_idx: animation.previous_stack_idx,
                    t: animation.t,
                    _pad: 0.0,
                });
                self.total_animations - 1
            }
            else {
                0xFFFFFFFF
            };

            self.flat.push(GpuCard {
                id_and_value: card.get_bits(),
                tableau: tableau_idx as u32,
                stack_idx: stack_idx as u32,
                animation_id,
                _pad: 0,
            });
        }
    }
}



#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
struct GpuAnimation {
    previous_tableau: u32,
    previous_stack_idx: u32,
    t: f32,
    _pad: f32
}




#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
struct GpuCard {
    id_and_value: u64,
    tableau: u32,
    stack_idx: u32,
    animation_id: u32,
    _pad: u32
}

