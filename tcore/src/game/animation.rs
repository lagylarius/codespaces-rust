use std::collections::HashMap;

use crate::game::card::Card;

pub struct Animation {
    pub previous_tableau: u32,
    pub previous_stack_idx: u32,
    pub t: f32
}
pub struct AnimationQueue {
    active: HashMap<u32, Animation>,
}

impl AnimationQueue {
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
        }
    }
    pub fn new_animation_batch<'a,I>(&mut self, iter: I, tableau_id: u32) where I: Iterator<Item = (usize, &'a Card)>{
        let mut current_t = 0.0;
        for (i,c) in iter {
            self.active.insert(c.id(), 
            Animation { 
                previous_tableau: tableau_id, 
                previous_stack_idx: i as u32,
                t: current_t
            });
            current_t -= 0.2;
        }
    }
    pub fn new_animation(&mut self, card: &Card, previous_tableau: u32, previous_stack_idx: u32) {
        self.active.insert(card.id(), 
            Animation { 
                previous_tableau: previous_tableau, 
                previous_stack_idx: previous_stack_idx,
                t: 0.0
            });
    }
    pub fn animation_for_card(&self, card: &Card) -> Option<&Animation> {
        self.active.get(&card.id())
    }
    pub fn advance_animations(&mut self) {
        self.active.retain(|_, v| {
            v.t += 0.2;
            v.t < 1.0
        });
    }
}

impl Default for AnimationQueue {
    fn default() -> Self {
        Self::new()
    }
}
