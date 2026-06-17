use std::sync::Arc;

use bytemuck::Zeroable;
use anyhow::anyhow;


const SUITS: [u32; 4] = [SUIT_SPADES, SUIT_HEARTS, SUIT_CLUBS, SUIT_DIAMONDS];
const VALS: [u32; 13] = [
    VAL_ACE, VAL_TWO, VAL_THREE, VAL_FOUR, VAL_FIVE, VAL_SIX, VAL_SEVEN,
    VAL_EIGHT, VAL_NINE, VAL_TEN, VAL_JACK, VAL_QUEEN, VAL_KING,
];

const VAL_ACE: u32   = 0x01;
const VAL_TWO: u32   = 0x02;
const VAL_THREE: u32 = 0x03;
const VAL_FOUR: u32  = 0x04;
const VAL_FIVE: u32  = 0x05;
const VAL_SIX: u32   = 0x06;
const VAL_SEVEN: u32 = 0x07;
const VAL_EIGHT: u32 = 0x08;
const VAL_NINE: u32  = 0x09;
const VAL_TEN: u32   = 0x0A;
const VAL_JACK: u32  = 0x0B;
const VAL_QUEEN: u32 = 0x0C;
const VAL_KING: u32  = 0x0D;

const SUIT_HEARTS   : u32 = 0x00;
const SUIT_SPADES   : u32 = 0x10;
const SUIT_DIAMONDS : u32 = 0x20;
const SUIT_CLUBS    : u32 = 0x30;

const COLOR_MASK: u32 = 0b0001 << 4;

const TYPE_CARD: u32 = 0x100;
const TYPE_HIDDEN: u32 = 0x200;
const TYPE_TABLEAU: u32 = 0x300;


const VALUE_MASK: u32 = 0x00F;
const SUIT_MASK: u32  = 0x0F0;
const TYPE_MASK: u32  = 0xF00;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
struct GpuCard {
    id: u32,
    value: u32,
    tableau: u32,
    stack_idx: u32
}


use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

fn next_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

struct Card {
    id: u32,
    value: u32
}

impl Card {
    fn suit(&self) -> u32 { self.value & SUIT_MASK }
    fn t(&self) -> u32 { self.value & TYPE_MASK }
    fn val(&self) -> u32  { self.value & VALUE_MASK }
    
    fn hide(&mut self) {
        self.value = TYPE_HIDDEN | (self.value & !TYPE_MASK);
    }
    fn unhide(&mut self) {
        if self.is_tableau() {return;}
        let payload = self.value & !TYPE_MASK;
        self.value = TYPE_CARD | payload;
    }
    fn is_tableau(&self) -> bool {
        self.t() == TYPE_TABLEAU
    }
    fn is_hidden(&self) -> bool {
        self.t() == TYPE_HIDDEN
    }
    fn new(suit: u32, value: u32) -> Card {
        Card {
            id: next_id(),
            value: TYPE_HIDDEN | value | suit
        }
    }
    fn new_tableau() -> Card {
        Card {
            id: next_id(),
            value: TYPE_TABLEAU
        }
    }

    fn get_color(&self) -> u32 {self.suit() & COLOR_MASK}



    // fn can_be_placed_on(&self, tab: &Vec<Card>) -> bool {
    //     let Some(c) = tab.last() else {
    //         return false;
    //     };

    //     if (c.is_tableau() && c.)
    // }
}

pub struct CardArray {
    tableaus: Vec<Vec<Card>>
}

struct CardPos {
    t: usize,
    c: usize,
}

impl CardArray {


    fn get_pos_by_id(&self, id: u32) -> Option<CardPos> {
        self.tableaus.iter().enumerate().find_map(|(t_idx, t)| {
            t.iter().position(|c| c.id == id)
                .map(|c_idx| CardPos {t: t_idx, c: c_idx })
        })
    }
    fn get_card_by_id(&self, id: u32) -> Option<&Card> {
        self.tableaus.iter().flatten().find(|c| c.id == id)
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

    fn reveal_top(&mut self, tableau: usize) {
        if let Some(c) = self.tableaus[tableau].last_mut() && !c.is_tableau() {
            c.unhide();
        }
    }
    fn reveal_top_all(&mut self) {
        for i in 0..self.tableaus.len() {
            self.reveal_top(i);
        }
    }

    fn initialize(&mut self) {
        for i in 1..7 {
            self.add_c(Card::new_tableau(),i);
        }

        let mut deck: Vec<Card> = Vec::new();

        for suit in SUITS {
            for val in 1..14 {
                deck.push(Card::new(suit, val));
            }
        }

        use rand::seq::SliceRandom;
        use rand::rng;

        deck.shuffle(&mut rng());

        let mut t: u32 = 1;

        for card in deck {
            self.add_c(card, t);

            t += 1;
            if t >= 7 {
                t = 1;
            }
        }
        // self.add_c(Card::new(SUIT_HEARTS,0x02), 1);
        // self.add_c(Card::new(SUIT_HEARTS,0x02), 1);
        // self.add_c(Card::new(SUIT_HEARTS,0x02), 1);

        
        // self.add_c(Card::new(SUIT_DIAMONDS,0x09), 2);

        self.reveal_top_all();
    }


    fn move_view(&mut self, t_idx: usize, c_idx: usize, to: u32) {
        let split_off = self.tableaus[t_idx].split_off(c_idx);
        self.tableaus[to as usize].extend(split_off);
    }
    fn move_tableau(&mut self, from: usize, to: usize) {
        let from_cards = std::mem::take(&mut self.tableaus[from]);
        self.tableaus[to].extend(from_cards);
    }

    fn is_valid_sequence<'a>(cards: impl Iterator<Item = &'a Card>) -> bool {
        let mut peekable = cards.peekable();
        while let Some(a) = peekable.next() {
            if let Some(b) = peekable.peek() {
                if a.is_tableau() && b.val() == VAL_KING {return true;}
                let diff_color = a.get_color() != b.get_color();
                let descending = a.val() == b.val() + 1;
                if !diff_color || !descending { return false; }
            }
        }
        true
    }

    pub fn can_be_placed(from: &[Card], to: &[Card]) -> bool {
        let i = to.last().into_iter().chain(from.iter());
        Self::is_valid_sequence(i)
    }

    pub fn can_be_picked(from: &[Card]) -> bool {
        Self::is_valid_sequence(from.iter())
    }

    pub fn pick_card(&mut self, id: u32) {
        if id == 0xFFFFFFFF { return;}
        let Some(pos) = self.get_pos_by_id(id) else {
            log_print!("Unknown card with id {}!",id);
            return;
        };
        if self.tableaus[0].is_empty() {
            let pos = self.get_pos_by_id(id).unwrap();
            if !self.tableaus[pos.t][pos.c].is_tableau() && !self.tableaus[pos.t][pos.c].is_hidden() {
                if Self::can_be_picked(&self.tableaus[pos.t][pos.c..]) {
                    self.move_view(pos.t, pos.c, 0);
                    self.reveal_top(pos.t);
                }
            }
        }
        else {
            if Self::can_be_placed(&self.tableaus[0], &self.tableaus[pos.t]) {
                self.move_tableau(0,pos.t);
            }

        }
    }

    pub fn new() -> Self {
        let mut s = Self { tableaus: vec![] };
        s.initialize();
        s
    }

    pub fn flush_to_buffer(&self, queue: &Arc<wgpu::Queue>, buffer: &wgpu::Buffer) {
        let mut flat: Vec<GpuCard> = Vec::new();
        let total: u32 = self.tableaus.iter().map(|t| t.len() as u32).sum();

        for (tableau_idx, tableau) in self.tableaus.iter().enumerate() {
            for (stack_idx, card) in tableau.iter().enumerate() {
                flat.push(GpuCard {
                    id: card.id as u32,
                    value: card.value,
                    tableau: tableau_idx as u32,
                    stack_idx: stack_idx as u32,
                });
            }
        }

        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&total));
        queue.write_buffer(&buffer, 4, bytemuck::bytes_of(&total.div_ceil(256)));
        queue.write_buffer(buffer, 16, bytemuck::cast_slice(&flat));
    }
}