mod card;
mod sequence;
mod gpu;
mod animation;

use card::*;
use std::{slice::SliceIndex, sync::Arc};
use rand::seq::SliceRandom;
use rand::rng;
use crate::{game::{animation::AnimationQueue, card::card_kind::{CardKind, JLevel, JokerKind, ReservedVal, TableauVal}, gpu::GPUContext, sequence::{AsCardSequence, AsMutCardSequence, CardSequence, CardSequenceMut}}, gui::PanelInfo};



const MOUSE_TABLEAU_ID: usize = 0;
const DISCARD_PILE_ID: usize = 1;
const BURN_TABLEAU_ID: usize = 2;

const RESERVED_TABLEAUS: usize = 3;








struct CardPos {
    t: TableauIndex,
    c: usize,
}

#[derive(Clone, Copy,PartialEq, Eq)]
enum TableauIndexType {
    Reserved,
    Top,
    Bottom
}

#[derive(Clone, Copy,PartialEq, Eq)]
struct TableauIndex {
    t: TableauIndexType,
    index: usize
}

impl TableauIndex {
    const MOUSE: TableauIndex = TableauIndex {t: TableauIndexType::Reserved, index: MOUSE_TABLEAU_ID};
    const DISCARD_PILE: TableauIndex = TableauIndex {t: TableauIndexType::Reserved, index: DISCARD_PILE_ID};
    const BURN_TABLEAU: TableauIndex = TableauIndex {t: TableauIndexType::Reserved, index: BURN_TABLEAU_ID};

    pub fn tableau_index(&self) -> usize {
        match self.t {
            TableauIndexType::Reserved => self.index,
            TableauIndexType::Top => RESERVED_TABLEAUS + (self.index * 2),
            TableauIndexType::Bottom => RESERVED_TABLEAUS + (self.index * 2) + 1,
        }
    }
    fn bottom(index: usize) -> Self {
        Self { t: TableauIndexType::Bottom, index: index }
    }
    fn top(index: usize) -> Self {
        Self { t: TableauIndexType::Top, index: index }
    }
    fn reserved(index: usize) -> Self {
        Self { t: TableauIndexType::Reserved, index: index }
    }
}




//###############################################################
//#--- Game
//###############################################################
pub struct Game {
    points: u32,
    objective: Objective,
    top_tableaus: Vec<Vec<Card>>,
    bottom_tableaus: Vec<Vec<Card>>,
    reserved_tableaus: [Vec<Card>;RESERVED_TABLEAUS],
    def_mouse_card: Card,
    def_burn_card: Card,
    def_null_card: Card,
    animation_queue: AnimationQueue
}

impl Game {
    
    fn update_at(&mut self, tableau : TableauIndex, c: usize) {
        log_print!("{}",c);
        if self.get_tableau(tableau).is_empty() {return;}

        // self.get_t_as_mut_sequence(tableau, ..).update_end();

        self.get_t_as_mut_sequence(tableau, ..).update_at(c);
        
        if self.get_tableau(tableau).get(c).is_some_and(|card| {card.kind() == CardKind::Tableau(TableauVal::Burn)}) {
            let mut removed: Vec<Card> = self.get_tableau_mut(tableau).drain((c+1)..).collect();
            let mut points: i32 = 0;
            removed.iter_mut().for_each(|c| {
                let p = c.get_points().unwrap_or(0);
                log_print!("+{} points",p);
                points += p;
                c.set_hidden(true);
            });
            self.points = self.points.saturating_add_signed(points);
            self.get_tableau_mut(TableauIndex::DISCARD_PILE).extend(removed);
        }
    }

    pub fn pick_card(&mut self, id: u32) {
        if id == 0xFFFFFFFF { return;}
        let Some(pos) = self.get_pos_by_id(id) else {
            log_print!("Unknown card with id {}!",id);
            return;
        };
        if self.reserved_tableaus[MOUSE_TABLEAU_ID].is_empty() {
            log_print!("????");
            if self.get_t_as_sequence(pos.t,pos.c..).can_be_placed_on(self.get_t_as_sequence(TableauIndex::MOUSE,..)) {
                self.move_tableau_pos_onwards(pos.t, pos.c, TableauIndex::MOUSE);
                self.update_at(pos.t,pos.c.wrapping_sub(1));
                // self.update(pos.t);
                // self.update_at(tableau, c);
            }
        }
        else {
            if self.get_t_as_sequence(TableauIndex::MOUSE,..).can_be_placed_on(self.get_t_as_sequence(pos.t,..)) {
                self.move_tableau(TableauIndex::MOUSE,pos.t);
                self.update_at(pos.t,pos.c);
                // self.update(pos.t);
            }
        }
    }

    fn initialize(&mut self) {
        self.get_tableau_mut(TableauIndex::BURN_TABLEAU).push(Card::new_tableau(TableauVal::Burn));
        for i in 0..6 {
            self.add_c(Card::new_tableau(TableauVal::Normal),TableauIndex::top(i ));
        }
        for i in 0..4 {
            self.add_c(Card::new_tableau(TableauVal::Shop),TableauIndex::bottom(i ));
            // self.add_c(Card::new_card(Suit::Joker, value))
            // if i < 2 {

            // self.add_c(Card::new_common(CardKind::Joker(JokerKind::Joker(JLevel::High))),TableauIndex::bottom(i ));
            // }
            // else {
            // self.add_c(Card::new_common(CardKind::Joker(JokerKind::Joker(JLevel::Low))),TableauIndex::bottom(i ));
            // }
        }
        let mut c = Card::new_common(CardKind::Joker(JokerKind::Joker(JLevel::High)));
        c.set_ghost(true);
        self.add_c(c,TableauIndex::bottom(0 ));
        let mut c = Card::new_common(CardKind::Joker(JokerKind::Joker(JLevel::Low)));
        c.set_metal(true);
        self.add_c(c,TableauIndex::bottom(1 ));
        self.add_c(Card::new_common(CardKind::Joker(JokerKind::Mimic(JLevel::High))),TableauIndex::bottom(2 ));
        self.add_c(Card::new_common(CardKind::Joker(JokerKind::Mimic(JLevel::Low))),TableauIndex::bottom(3));

        let mut deck: Vec<Card> = Vec::new();

        for _ in 0..1 {
            for suit in STANDARD_SUITS {
                for val in STANDARD_VALS {
                    let mut c = Card::new_card(suit, val);
                    c.set_hidden(true);
                    deck.push(c);
                }
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
        let r = self.top_tableaus.iter().enumerate().find_map(|(t_idx, t)| {
            t.iter().position(|c| c.id() == id)
                .map(|c_idx| {
                    let t_index = TableauIndex::top(t_idx);
                    CardPos {t: t_index, c: c_idx }
                })
        });
        if r.is_some() {
            return r;
        }
        let r = self.bottom_tableaus.iter().enumerate().find_map(|(t_idx, t)| {
            t.iter().position(|c| c.id() == id)
                .map(|c_idx| {
                    let t_index = TableauIndex::bottom(t_idx);
                    CardPos {t: t_index, c: c_idx }
                })
        });
        if r.is_some() {
            return r;
        }
        else {
            self.reserved_tableaus.iter().enumerate().find_map(|(t_idx, t)| {
                t.iter().position(|c| c.id() == id)
                    .map(|c_idx| {
                        let t_index = TableauIndex::reserved(t_idx);
                        return CardPos {t: t_index, c: c_idx };
                    })
            })
        }
    }

    fn add_c(&mut self, card: Card, ti: TableauIndex) {


        match ti.t {
            TableauIndexType::Reserved => {
                if self.reserved_tableaus.len() <= ti.index as usize {
                    panic!("Incorrect add to reserved tableau on index {}, which is past index {}",ti.index,self.reserved_tableaus.len());
                }
            },
            TableauIndexType::Top => {
                if self.top_tableaus.len() <= ti.index as usize {
                    self.top_tableaus.resize_with(ti.index as usize + 1, Vec::new);
                }
            },
            TableauIndexType::Bottom => {
                if self.bottom_tableaus.len() <= ti.index as usize {
                    self.bottom_tableaus.resize_with(ti.index as usize + 1, Vec::new);
                }
            },
        }

        let tableaus: &mut [Vec<Card>] = match ti.t {
            TableauIndexType::Reserved => &mut self.reserved_tableaus,
            TableauIndexType::Top => &mut self.top_tableaus,
            TableauIndexType::Bottom => &mut self.bottom_tableaus,
        };

        tableaus[ti.index].push(card);
    }

    fn get_tableau(&self, ti: TableauIndex) -> &Vec<Card> {
        match ti.t {
            TableauIndexType::Reserved => &self.reserved_tableaus[ti.index],
            TableauIndexType::Top => &self.top_tableaus[ti.index],
            TableauIndexType::Bottom => &self.bottom_tableaus[ti.index],
        }
    }

    // fn get_t_as_sequence<R: RangeBounds<usize> + std::slice::SliceIndex<[Card]>>(&self, ti: TableauIndex, range: R) -> CardSequence {
    //     let t: &[Card] = &self.get_tableau(ti)[range];
    //     t.as_sequence(self.default_card)
    // }

    fn get_t_as_sequence<'s,R>(&'s self, ti: TableauIndex, range: R) -> CardSequence<'s>
        where R: SliceIndex<[Card], Output = [Card]>,
    {
        let slice: &[Card] = &self.get_tableau(ti)[range];
        let default = match ti {
            TableauIndex::MOUSE => self.def_mouse_card,
            TableauIndex::BURN_TABLEAU => self.def_burn_card,
            _ => self.def_null_card,
        };
        slice.as_sequence(default)
    }
    fn get_t_as_mut_sequence<'s,R>(&'s mut self, ti: TableauIndex, range: R) -> CardSequenceMut<'s>
        where R: SliceIndex<[Card], Output = [Card]>,
    {
        let default = match ti {
            TableauIndex::MOUSE => self.def_mouse_card,
            TableauIndex::BURN_TABLEAU => self.def_burn_card,
            _ => self.def_null_card,
        };
        let slice: &mut [Card] = &mut self.get_tableau_mut(ti)[range];
        slice.as_mut_sequence(default)
    }

    fn get_tableau_mut(&mut self, ti: TableauIndex) -> &mut Vec<Card> {
        match ti.t {
            TableauIndexType::Reserved => &mut self.reserved_tableaus[ti.index],
            TableauIndexType::Top => &mut self.top_tableaus[ti.index],
            TableauIndexType::Bottom => &mut self.bottom_tableaus[ti.index],
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
            let prev_size = self.get_tableau(TableauIndex::top(t)).len();
            let cmax = base + if i < extra { 1 } else { 0 };
            for c in 0..cmax {
                let mut card = deck.pop().unwrap();
                if t < c {
                    card.set_hidden(false);
                }
                else {
                    card.set_hidden(true);
                }
                self.add_c(card, TableauIndex::top(t));
            }

            for c in prev_size..prev_size+cmax {
                self.update_at(TableauIndex::top(t),c);
            }

            // for c in 1..cmax+1 {
            //     if t+2 < c {
            //         self.get_tableau_mut(TableauIndex::top(t))[c].set_hidden(false);
            //     }
            //     else {
            //         self.get_tableau_mut(TableauIndex::top(t))[c].set_hidden(true);
            //     }
            // }

        }
    }


    pub fn new() -> Self {
        let mut s = Self {
            points: 0,
            objective: Objective::new(),
            top_tableaus: vec![vec![]; RESERVED_TABLEAUS],
            bottom_tableaus: vec![vec![];3],
            reserved_tableaus: [vec![],vec![],vec![]],
            animation_queue: AnimationQueue::new(),
            def_mouse_card: Card::new_common(CardKind::_Reserved(ReservedVal::MouseCard)),
            def_burn_card: Card::new_common(CardKind::_Reserved(ReservedVal::BurnCard)),
            def_null_card: Card::new_common(CardKind::_Reserved(ReservedVal::NullCard)),
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
            let t = TableauIndex::reserved(tableau_idx);
            if t == TableauIndex::DISCARD_PILE {
                gpu_context.push_cards(tableau, TableauIndex::reserved(tableau_idx).tableau_index() as u32,  &self.animation_queue);
            }
            else {
                gpu_context.push_cards_zoom_calc(tableau, TableauIndex::reserved(tableau_idx).tableau_index() as u32,  &self.animation_queue);
            }
        }

        for (tableau_idx, tableau) in self.top_tableaus.iter().enumerate() {
            gpu_context.push_cards_zoom_calc(tableau, TableauIndex::top(tableau_idx).tableau_index() as u32,  &self.animation_queue);
        }

        for (tableau_idx, tableau) in self.bottom_tableaus.iter().enumerate() {
            gpu_context.push_cards_zoom_calc(tableau, TableauIndex::bottom(tableau_idx).tableau_index() as u32,  &self.animation_queue);
        }

        gpu_context.flush_to_gpu(queue, buffer, animation_buffer);
    }

    pub fn get_point_info(&self) -> (u32,u32,u32) {
        (self.points,self.objective.cards,self.objective.burns)
    }

    pub fn get_str_info(&self, id: u32, pos_x: f32,pos_y: f32) -> Option<PanelInfo> {
        if id == 0xFFFFFFFF { return None;}
        let Some(pos) = self.get_pos_by_id(id) else {
            log_print!("Unknown card with id {}!",id);
            return None;
        };

        let card = self.get_tableau(pos.t).get(pos.c).unwrap();
        let mut info = card.get_info();
        if pos.t == TableauIndex::DISCARD_PILE {
            info = vec![info[0].clone(),
                    "This card is on the discard pile".to_string(),
                    "It cannot be picked up or interacted with".to_string()];
        }

        return Some(
            PanelInfo {
                    pos_x, 
                    pos_y, 
                    text: info
            }
        );
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