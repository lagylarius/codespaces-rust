
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

fn next_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}


use bilge::prelude::*;


pub const SUITS: [Suit; 4] = [Suit::Hearts, Suit::Spades, Suit::Diamonds, Suit::Clubs];
pub const VALS: [Val; 13] = [
        Val::Ace,
        Val::Two,
        Val::Three,
        Val::Four,
        Val::Five,
        Val::Six,
        Val::Seven,
        Val::Eight,
        Val::Nine,
        Val::Ten,
        Val::Jack,
        Val::Queen,
        Val::King
    ];

#[bitsize(2)]
#[derive(FromBits,PartialEq,Clone, Copy)]
pub enum Suit {
    Hearts = 0,
    Spades = 1,
    Diamonds = 2,
    Clubs = 3,
}

#[bitsize(4)]
#[derive(FromBits,PartialEq,Clone, Copy)]
pub enum Val {
    Ace   = 1,
    Two   = 2,
    Three = 3,
    Four  = 4,
    Five  = 5,
    Six   = 6,
    Seven = 7,
    Eight = 8,
    Nine  = 9,
    Ten   = 10,
    Jack  = 11,
    Queen = 12,
    King  = 13,
    #[fallback]
    pad0 = 0,
}

#[bitsize(2)]
#[derive(FromBits,PartialEq,Clone, Copy)]
pub enum CardType {
    Card = 0,
    Hidden = 1,
    Tableau = 2,
    _Pad = 3,
}


#[bitsize(4)]
#[derive(FromBits,PartialEq,Clone,Copy)]
pub enum TableauType {
    Normal = 0,
    Burn = 1,
    #[fallback]
    pad0 = 2
}
impl From<TableauType> for Val {
    fn from(t: TableauType) -> Self {
        let raw: UInt<u8, 4> = UInt::<u8, 4>::from(t);
        Val::from(raw)
    }
}
impl From<Val> for TableauType {
    fn from(t: Val) -> Self {
        let raw: UInt<u8, 4> = UInt::<u8, 4>::from(t);
        TableauType::from(raw)
    }
}

#[bitsize(64)]
#[derive(FromBits,PartialEq,Clone, Copy)]
pub struct Card {
    pub id: u32,
    pub suit: Suit,
    pub val: Val,
    pub t: CardType,
    pad: u24,
}


impl Card {
    pub fn new_card(suit: Suit, value: Val) -> Card {
                        // let b = u4::from_u32(val);
                // let mut c = Card::new_card(suit, Val::from(b));
        Card::new(
            next_id(), 
            suit,
            value, 
            CardType::Card, 
            u24::ZERO,
        )
    }
    pub fn tableau_type(&self) -> Option<TableauType> {
        if self.t() != CardType::Tableau {
            return None;
        }
        return Some(self.val().into());
    }
    pub fn new_burn_tableau() -> Card {
        Card::new(
            next_id(), 
            Suit::from(UInt::ZERO),
            TableauType::Burn.into(),
            CardType::Tableau, 
            u24::ZERO,
        )
    }
    pub fn new_tableau() -> Card {
        Card::new(
            next_id(), 
            Suit::from(UInt::ZERO),
            Val::from(UInt::ZERO), 
            CardType::Tableau, 
            u24::ZERO,
        )
    }
    pub fn hide(&mut self) {
        self.set_t(CardType::Hidden);
    }
    pub fn unhide(&mut self) {
        if self.t() == CardType::Hidden {
            self.set_t(CardType::Card);
        }
    }
    pub fn get_color(&self) -> u32 {
        let bit = match self.suit() {
            Suit::Hearts | Suit::Diamonds => 0,
            Suit::Spades | Suit::Clubs => 1,
        };
        return bit
    }

    pub fn tableau_is_burn(&self) -> bool {
        if let Some(t) = self.tableau_type() && t == TableauType::Burn {
            return true;
        }
        return false;
    }

    pub fn val_numeric(&self) -> u8 {
        let raw_a: UInt<u8, 4> = self.val().into();
        return raw_a.value();
    }


    pub fn get_bits(&self) -> u64 {
        return self.value;
    }
}
