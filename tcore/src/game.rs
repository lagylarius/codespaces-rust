mod card;
mod sequence;
use card::*;

mod gpu;


use std::{collections::HashMap, sync::Arc};
use rand::seq::SliceRandom;
use rand::rng;
use crate::game::{gpu::GPUContext, sequence::CardSequenceOp};



const MOUSE_TABLEAU_ID: usize = 0;
const DISCARD_PILE_ID: usize = 1;
const BURN_TABLEAU_ID: usize = 2;

const RESERVED_TABLEAUS: usize = 3;








struct CardPos {
    t: TableauIndex,
    c: usize,
}

#[derive(Clone, Copy)]
struct TableauIndex {
    is_reserved: bool,
    index: usize
}

impl TableauIndex {
    const MOUSE: TableauIndex = TableauIndex {is_reserved: true, index: MOUSE_TABLEAU_ID};
    const DISCARD_PILE: TableauIndex = TableauIndex {is_reserved: true, index: DISCARD_PILE_ID};

    pub fn tableau_index(&self) -> usize {
        if self.is_reserved {
            self.index
        } else {
            RESERVED_TABLEAUS + (self.index * 2)
        }
    }
    fn non_reserved(index: usize) -> Self {
        Self { is_reserved: false, index: index }
    }
    fn reserved(index: usize) -> Self {
        Self { is_reserved: true, index: index }
    }
}




//###############################################################
//#--- Game
//###############################################################
pub struct Game {
    points: u32,
    objective: Objective,
    tableaus: Vec<Vec<Card>>,
    shop_tableaus: Vec<Vec<Card>>,
    reserved_tableaus: [Vec<Card>;RESERVED_TABLEAUS],
    animation_queue: AnimationQueue
}

impl Game {
    fn update(&mut self, tableau : TableauIndex) {
        if self.get_tableau(tableau).is_empty() {return;}
        if self.get_tableau(tableau).first().unwrap().tableau_is_burn() {
            let mut removed: Vec<Card> = self.get_tableau_mut(tableau).drain(1..).collect();

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
            
            self.get_tableau_mut(TableauIndex::DISCARD_PILE).extend(removed);

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

        if self.get_tableau(tableau).last().unwrap().t() == CardType::Hidden {
            self.get_tableau_mut(tableau).last_mut().unwrap().unhide();
        }
    }


    pub fn pick_card(&mut self, id: u32) {
        if id == 0xFFFFFFFF { return;}
        let Some(pos) = self.get_pos_by_id(id) else {
            log_print!("Unknown card with id {}!",id);
            return;
        };
        if self.reserved_tableaus[MOUSE_TABLEAU_ID].is_empty() {
            let pos = self.get_pos_by_id(id).unwrap();
            if (&self.get_tableau(pos.t)[pos.c..]).can_be_picked() {
                self.move_tableau_pos_onwards(pos.t, pos.c, TableauIndex::MOUSE);
                self.update(pos.t);
            }
        }
        else {
            if (&self.get_tableau(TableauIndex::MOUSE)[..]).can_be_placed_on(&self.get_tableau(pos.t)[..]) {
                self.move_tableau(TableauIndex::MOUSE,pos.t);
                self.update(pos.t);
            }
        }
    }

    fn initialize(&mut self) {
        // self.add_c(Card::new_burn_tableau(),1);
        self.reserved_tableaus[BURN_TABLEAU_ID].push(Card::new_burn_tableau());
        for i in 0..6 {
            self.add_c(Card::new_tableau(),i as u32);
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

        self.reserved_tableaus[DISCARD_PILE_ID] = deck;
    }


    //###############################################################
    //#--- Brute searches by @id
    //#--- and returns the position CardPos (card at tableau t at index i)
    //#--- if it exists (None otherwise)
    //###############################################################
    fn get_pos_by_id(&self, id: u32) -> Option<CardPos> {
        let r = self.tableaus.iter().enumerate().find_map(|(t_idx, t)| {
            t.iter().position(|c| c.id() == id)
                .map(|c_idx| {
                    let t_index = TableauIndex::non_reserved(t_idx);
                    CardPos {t: t_index, c: c_idx }
                })
        });
        if !r.is_none() {
            r
        }
        else {
            self.reserved_tableaus.iter().enumerate().find_map(|(t_idx, t)| {
                t.iter().position(|c| c.id() == id)
                    .map(|c_idx| {
                        let t_index = TableauIndex::reserved(t_idx);
                        CardPos {t: t_index, c: c_idx }
                    })
            })
        }

    }

    fn add_c(&mut self, card: Card, tableau: u32) {
        if self.tableaus.len() <= tableau as usize {
            self.tableaus.resize_with(tableau as usize + 1, Vec::new);
        }

        self.tableaus[tableau as usize].push(card);
    }

    fn get_tableau(&self, ti: TableauIndex) -> &Vec<Card> {
        if !ti.is_reserved {
            &self.tableaus[ti.index]
        }
        else {
            &self.reserved_tableaus[ti.index]
        }
    }

    fn get_tableau_mut(&mut self, ti: TableauIndex) -> &mut Vec<Card> {
        if !ti.is_reserved {
            &mut self.tableaus[ti.index]
        }
        else {
            &mut self.reserved_tableaus[ti.index]
        }
    }

    //###############################################################
    //#--- Picks up cards from position @c_idx onwards in tableau @from and moves them to tableau @to. 
    //#--- Used when picking up a sequence to hold in hand (tableau 0).
    //###############################################################
    fn move_tableau_pos_onwards(&mut self, from: TableauIndex, c_idx: usize, to: TableauIndex) {
        let split_off = self.get_tableau_mut(from).split_off(c_idx);
        for (i,c) in split_off.iter().enumerate() {
            self.animation_queue.new_animation(c, from.tableau_index() as u32, (c_idx + i) as u32);
        }
        self.get_tableau_mut(to).extend(split_off);
    }
    //###############################################################
    //#--- Picks up all cards from tableau @from onwards and moves them to tableau @to.
    //###############################################################
    fn move_tableau(&mut self, from: TableauIndex, to: TableauIndex) {
        let from_cards = std::mem::take(self.get_tableau_mut(from));
        for (i,c) in from_cards.iter().enumerate() {
            self.animation_queue.new_animation(c, from.tableau_index() as u32, i as u32);
        }
        self.get_tableau_mut(to).extend(from_cards);
    }

    pub fn deal(&mut self) {
        self.reserved_tableaus[DISCARD_PILE_ID].shuffle(&mut rng());

        let mut deck: Vec<Card> = std::mem::take(&mut self.reserved_tableaus[DISCARD_PILE_ID]);

        let tableau_id: u32 = DISCARD_PILE_ID as u32;
        
        self.animation_queue.new_animation_batch(deck.iter().enumerate(),tableau_id);

        let piles = 6;
        let base = deck.len() / piles;
        let extra = deck.len() % piles;


        for (i,t) in (0..6).enumerate() {
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

            self.update(TableauIndex::non_reserved(t));
        }
    }


    pub fn new() -> Self {
        let mut s = Self {
            points: 0,
            objective: Objective::new(),
            tableaus: vec![vec![]; RESERVED_TABLEAUS],
            shop_tableaus: vec![vec![];3],
            reserved_tableaus: [vec![],vec![],vec![]],
            animation_queue: AnimationQueue::new()
        };
        s.initialize();
        s
    }

    pub fn frame_step(&mut self) {
        self.animation_queue.advance_animations();
    }

    pub fn gpu_sync(&self, queue: &Arc<wgpu::Queue>, buffer: &wgpu::Buffer, animation_buffer: &wgpu::Buffer) {
        let mut gpu_context = GPUContext::new();

        for (tableau_idx, tableau) in self.reserved_tableaus.iter().enumerate() {
            gpu_context.push_cards(tableau, TableauIndex::reserved(tableau_idx).tableau_index() as u32,  &self.animation_queue);
        }

        for (tableau_idx, tableau) in self.tableaus.iter().enumerate() {
            gpu_context.push_cards(tableau, TableauIndex::non_reserved(tableau_idx).tableau_index() as u32,  &self.animation_queue);
        }

        gpu_context.flush_to_gpu(queue, buffer, animation_buffer);
    }

    pub fn get_point_info(&self) -> (u32,u32,u32) {
        (self.points,self.objective.cards,self.objective.burns)
    }

    pub fn get_str_info(&self) -> Option<Vec<String>> {
        return None;
    }
}


//###############################################################
//#--- Objective
//###############################################################
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



//###############################################################
//#--- Animations
//###############################################################
struct Animation {
    previous_tableau: u32,
    previous_stack_idx: u32,
    t: f32,
    _pad: f32
}
struct AnimationQueue {
    active: HashMap<u32, Animation>,
}

impl AnimationQueue {
    fn new() -> Self {
        Self {
            active: HashMap::new(),
        }
    }
    fn new_animation_batch<'a,I>(&mut self, iter: I, tableau_id: u32) where I: Iterator<Item = (usize, &'a Card)>{
        let mut current_t = 0.0;
        for (i,c) in iter {
            self.active.insert(c.id(), 
            Animation { 
                previous_tableau: tableau_id, 
                previous_stack_idx: i as u32,
                t: current_t,
                _pad: 0.0,
            });
            current_t -= 0.2;
        }
    }
    fn new_animation(&mut self, card: &Card, previous_tableau: u32, previous_stack_idx: u32) {
        self.active.insert(card.id(), 
            Animation { 
                previous_tableau: previous_tableau, 
                previous_stack_idx: previous_stack_idx,
                t: 0.0,
                _pad: 0.0,
            });
    }
    fn animation_for_card(&self, card: &Card) -> Option<&Animation> {
        self.active.get(&card.id())
    }
    fn advance_animations(&mut self) {
        self.active.retain(|_, v| {
            v.t += 0.1;
            v.t < 1.0
        });
    }
}

impl Default for AnimationQueue {
    fn default() -> Self {
        Self::new()
    }
}
