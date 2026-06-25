mod card;
mod sequence;
use card::*;


use std::{collections::{HashMap, VecDeque}, sync::Arc};





pub struct CardArray {
    points: u32,
    objective: Objective,
    tableaus: Vec<Vec<Card>>,
    burn_pile: Vec<Card>,
    animation_queue: AnimationQueue
}

struct Objective {
    cards: u32,
    burns: u32,
    phase: u32,
}

enum ObjectiveStatus {
    Completed,
    Failed,
    Ongoing
}

impl Objective {
    const INITIAL_CARDS: u32 = 6;
    // const INITIAL_BURNS: u32 = 4;
    const INITIAL_BURNS: u32 = 40;

    fn new() -> Self { Self { cards: Self::INITIAL_CARDS, burns: Self::INITIAL_BURNS, phase: 1 } }
    
    fn burn(&mut self, count: u32) -> ObjectiveStatus {
        self.cards = self.cards.saturating_sub(count);
        self.burns -= 1;
        if self.cards == 0 {
            self.phase += 1;
            self.cards = Self::INITIAL_CARDS * self.phase;
            self.burns = Self::INITIAL_BURNS + self.phase * 2;
            return ObjectiveStatus::Completed;
        }

        if self.burns == 0 {
            return ObjectiveStatus::Failed
        }

        return ObjectiveStatus::Ongoing

    }
}

struct CardPos {
    t: usize,
    c: usize,
}


use rand::seq::SliceRandom;
use rand::rng;

use crate::game::sequence::{CardSequence, CardSequenceOp};

// pub fn can_be_placed(sequence_picked: &[Card], to: &[Card]) -> bool {
//     //You can always place cards on burn tableaus, no matter what
//     if to.get(0).is_some_and(|c| {c.tableau_is_burn()}) {return true;}

//     let to_last = &to[to.len() - 1..];
//     let sequence: &CardSequence = &[to_last,sequence_picked];
//     sequence.is_valid_sequence()
// }
// pub fn can_be_picked(sequence_picked: &[Card]) -> bool {
//     let sequence: &CardSequence = &[sequence_picked];
//     sequence.is_valid_sequence()
// }

impl CardArray {
    pub fn update(&mut self, tableau : usize) {
        if self.tableaus[tableau].is_empty() {return;}
        if self.tableaus[tableau].first().unwrap().tableau_is_burn() {
            let mut removed: Vec<Card> = self.tableaus[tableau].drain(1..).collect();

            let count = removed.len();

            //Burning tableau: -15p
            //Will burn cards. Cards burnt will give points if you burn at least 3 at the same time
            let mut points: i32 = -15;
            log_print!("Burning {} cards",removed.len());
            for c in removed.iter() {
                log_print!("+{} points",c.val_numeric());
                //Ace: 2x
                points += c.val_numeric() as i32;
            }

            removed.iter_mut().for_each(|c| {
                c.hide();
            });
            
            self.burn_pile.extend(removed);

            self.points = self.points.saturating_add_signed(points);


            match self.objective.burn(count as u32) {
                ObjectiveStatus::Completed => {

                },
                ObjectiveStatus::Failed => {
                    panic!("game over")
                },
                ObjectiveStatus::Ongoing => {

                },
            }
        }

        if self.tableaus[tableau].last().unwrap().t() == CardType::Hidden {
            self.tableaus[tableau].last_mut().unwrap().unhide();
        }
    }



    pub fn pick_card(&mut self, id: u32) {
        if id == 0xFFFFFFFF { return;}
        let Some(pos) = self.get_pos_by_id(id) else {
            log_print!("Unknown card with id {}!",id);
            return;
        };
        if self.tableaus[0].is_empty() {
            let pos = self.get_pos_by_id(id).unwrap();
            if (&self.tableaus[pos.t][pos.c..]).can_be_picked() {
                self.move_view(pos.t, pos.c, 0);
                self.update(pos.t);
            }
        }
        else {
            if (&self.tableaus[0][..]).can_be_placed_on(&self.tableaus[pos.t][..]) {
                self.move_tableau(0,pos.t);
                self.update(pos.t);
            }
        }
    }

    fn initialize(&mut self) {
        self.add_c(Card::new_burn_tableau(),1);
        for i in 2..8 {
            self.add_c(Card::new_tableau(),i);
        }
        let mut deck: Vec<Card> = Vec::new();

        for suit in SUITS {
            for val in VALS {
                let mut c = Card::new_card(suit, val);
                c.hide();
                deck.push(c);
            }
        }


        deck.shuffle(&mut rng());

        self.burn_pile = deck;
    }

    fn get_pos_by_id(&self, id: u32) -> Option<CardPos> {
        self.tableaus.iter().enumerate().find_map(|(t_idx, t)| {
            t.iter().position(|c| c.id() == id)
                .map(|c_idx| CardPos {t: t_idx, c: c_idx })
        })
    }
    fn get_card_by_id(&self, id: u32) -> Option<&Card> {
        self.tableaus.iter().flatten().find(|c| c.id() == id)
    }

    fn get_t(&self, tableau : u32) -> &[Card] {
        if self.tableaus.len() <= tableau as usize {
            return &[];
        }
        return &self.tableaus[tableau as usize];
    }
    fn get_t_mut(&mut self, tableau : u32) -> &mut Vec<Card> {
        if self.tableaus.len() <= tableau as usize {
            self.tableaus.resize_with(tableau as usize + 1, Vec::new);
        }
        return &mut self.tableaus[tableau as usize];
    }

    fn add_c(&mut self, card: Card, tableau: u32) {
        if self.tableaus.len() <= tableau as usize {
            self.tableaus.resize_with(tableau as usize + 1, Vec::new);
        }

        self.tableaus[tableau as usize].push(card);
    }




    //###############################################################
    //#--- Picks up cards from position @c_idx onwards in tableau @t_idx and moves them to tableau @to. 
    //#--- Used when picking up a sequence to hold in hand (tableau 0).
    //###############################################################
    fn move_view(&mut self, t_idx: usize, c_idx: usize, to: u32) {
        let split_off = self.tableaus[t_idx].split_off(c_idx);
        for (i,c) in split_off.iter().enumerate() {
            self.animation_queue.active.insert(c.id(), 
            Animation { 
                previous_tableau: t_idx as u32, 
                previous_stack_idx: (c_idx+i) as u32,
                t: 0.0,
                _pad: 0.0,
            });
        }
        self.tableaus[to as usize].extend(split_off);
    }
    fn move_tableau(&mut self, from: usize, to: usize) {
        let from_cards = std::mem::take(&mut self.tableaus[from]);
        for (i,c) in from_cards.iter().enumerate() {
            self.animation_queue.active.insert(c.id(), 
            Animation { 
                previous_tableau: from as u32, 
                previous_stack_idx: i as u32,
                t: 0.0,
                _pad: 0.0,
            });
        }
        self.tableaus[to].extend(from_cards);
    }

    pub fn deal(&mut self) {
        self.burn_pile.shuffle(&mut rng());

        let mut deck: Vec<Card> = std::mem::take(&mut self.burn_pile);

        let mut current_t = 0.0;
        
        let tableau_id: u32 = 0xFFFFFFF0;

        for (i,c) in deck.iter().enumerate() {
            self.animation_queue.active.insert(c.id(), 
            Animation { 
                previous_tableau: tableau_id, 
                previous_stack_idx: i as u32,
                t: current_t,
                _pad: 0.0,
            });
            current_t -= 0.2;
        }


        let piles = 6;
        let base = deck.len() / piles;
        let extra = deck.len() % piles;


        for (i,t) in (2..8).enumerate() {
            let cmax = base + if i < extra { 1 } else { 0 };
            for c in 0..cmax {
                let mut card = deck.pop().unwrap();
                if t < c {
                    card.unhide();
                }
                else {
                    card.hide();
                }
                self.add_c(card, t as u32);
            }

            self.update(t);
        }
    }





    pub fn new() -> Self {
        let mut s = Self {
            points: 0,
            objective: Objective::new(),
            // objective: (6, 4),
            // objective: (6, 40),
            burn_pile: vec![],
            tableaus: vec![] ,
            animation_queue: AnimationQueue::new()
        };
        s.initialize();
        s
    }

    pub fn advance_animations(&mut self) {
        self.animation_queue.active.retain(|_, v| {
            v.t += 0.1;
            v.t < 1.0
        });
    }

    pub fn flush_to_buffer(&self, queue: &Arc<wgpu::Queue>, buffer: &wgpu::Buffer, animation_buffer: &wgpu::Buffer) {

        log_print!("Points: {}",self.points);
        log_print!("Objective: {} in {}",self.objective.cards,self.objective.burns);

        let mut flat: Vec<GpuCard> = Vec::new();
        
        let mut flat_animations: Vec<GpuAnimation> = Vec::new();

        let mut total: u32 = self.tableaus.iter().map(|t| t.len() as u32).sum();

        let mut total_animations = 0;

        for (tableau_idx, tableau) in self.tableaus.iter().enumerate() {
            for (stack_idx, card) in tableau.iter().enumerate() {

                let animation_id = if let Some(animation) = self.animation_queue.animation_for_card(card) {
                    total_animations += 1;
                    flat_animations.push(GpuAnimation { 
                        previous_tableau: animation.previous_tableau, 
                        previous_stack_idx: animation.previous_stack_idx,
                        t: animation.t,
                        _pad: 0.0,
                    });
                    total_animations - 1
                }
                else {
                    0xFFFFFFFF
                };

                flat.push(GpuCard {
                    id_and_value: card.get_bits(),
                    tableau: tableau_idx as u32,
                    stack_idx: stack_idx as u32,
                    animation_id,
                    _pad: 0,
                });
            }
        }

        total += self.burn_pile.len() as u32;
        for (stack_idx, card) in self.burn_pile.iter().enumerate() {
            let animation_id = if let Some(animation) = self.animation_queue.animation_for_card(card) {
                total_animations += 1;
                flat_animations.push(GpuAnimation { 
                    previous_tableau: animation.previous_tableau, 
                    previous_stack_idx: animation.previous_stack_idx,
                    t: animation.t,
                    _pad: 0.0,
                });
                total_animations - 1
            }
            else {
                0xFFFFFFFF
            };
            
            flat.push(GpuCard {
                id_and_value: card.get_bits(),
                tableau: 0xFFFFFFF0,
                stack_idx: stack_idx as u32,
                animation_id,
                _pad: 0,
            });
        }


        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&total));
        queue.write_buffer(buffer, 4, bytemuck::bytes_of(&total.div_ceil(256)));
        queue.write_buffer(buffer, 16, bytemuck::cast_slice(&flat));

        
        queue.write_buffer(animation_buffer, 0, bytemuck::cast_slice(&flat_animations));
    }

    pub fn get_point_info(&self) -> (u32,u32,u32) {
        (self.points,self.objective.cards,self.objective.burns)
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

struct Animation {
    previous_tableau: u32,
    previous_stack_idx: u32,
    t: f32,
    _pad: f32
}
struct AnimationQueue {
    queue: VecDeque<Animation>,
    active: HashMap<u32, Animation>,
}

impl AnimationQueue {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            active: HashMap::new(),
        }
    }
    fn animation_for_card(&self, card: &Card) -> Option<&Animation> {
        self.active.get(&card.id())
    }
}

impl Default for AnimationQueue {
    fn default() -> Self {
        Self::new()
    }
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