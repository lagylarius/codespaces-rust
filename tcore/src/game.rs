use std::sync::Arc;

use bytemuck::Zeroable;


#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
struct Card {
    value: u32,
    tableau: u32,
    stack_idx: u32,
    _pad: u32
}


const TYPE_CARD: u32 = 0x100;
const TYPE_HIDDEN: u32 = 0x200;
const TYPE_TABLEAU: u32 = 0x300;

const SUIT_HEARTS: u32 = 0x1;

const VAL_ACE : u32 = 0x10;


const TYPE_MASK: u32 =  0xF00;

fn new_card(suit: u32, value: u32, tableau: u32, hidden: bool) -> Card {
    let t = if hidden {TYPE_HIDDEN} else {TYPE_CARD};

    Card {
        value: t | value | suit,
        tableau: tableau,
        stack_idx: 0,
        _pad: 0
    }
}

fn new_hidden(tableau: u32) -> Card {
    Card { value: TYPE_TABLEAU, tableau, stack_idx: 0, _pad: 0 }
}


impl Card {
    fn hide(&mut self) {
        self.value = TYPE_HIDDEN | (self.value & !TYPE_MASK);
    }
    fn unhide(&mut self) {
        let payload = self.value & !TYPE_MASK;
        self.value = TYPE_CARD | payload;
    }
}



pub struct CardArray {
    cards: Vec<Card>,
    tableau_top: [u32;255],
    tableau_count: [u32;255],

}

impl CardArray {
    fn new_card(&mut self, suit: u32, value: u32, tableau: u32) {
        let mut c = new_card(suit, value, tableau, false);

        c.stack_idx = self.tableau_count[tableau as usize];


        if self.tableau_top[tableau as usize] as usize != 0xFFFFFF  {
            self.cards[self.tableau_top[tableau as usize] as usize].hide();
        }

        
        self.cards.push(c);

        self.tableau_count[tableau as usize] += 1;
        self.tableau_top[tableau as usize] = self.cards.len() as u32 - 1;
    }
}

impl CardArray {
    pub fn new() -> Self {

        
        let mut s = Self {
            cards: vec![Card::zeroed(); 0],
            tableau_top: [0xFFFFFF;255],
            tableau_count: [0;255]
        };

        s.new_card(0x2, 0x00, 1);
        s.new_card(0x2, 0x00, 1);
        
            // s.cards.push(new_card(0x2, 0x00, 1, false));


            // s.cards.push(new_card(0x2, 0x00, 1, false));

        // for i in 0..12 {
        //     s.cards.push(new_card(0x2, 0x00, 1, false));
        //     s.cards[i].stack_idx = i as u32;
        // }
        return s;
    }

    pub fn flush_to_buffer(&self, queue: &Arc<wgpu::Queue>, buffer: &wgpu::Buffer) {
        queue.write_buffer(&buffer, 0, bytemuck::bytes_of(&self.cards.len()));
        queue.write_buffer(&buffer, 4, bytemuck::bytes_of(&self.cards.len().div_ceil(256)));
        queue.write_buffer(&buffer, 16, bytemuck::cast_slice(&self.cards));
    }
}
