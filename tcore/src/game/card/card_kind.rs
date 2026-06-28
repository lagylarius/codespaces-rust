use bilge::{FromBits, bitsize};
use bilge::prelude::*;

use crate::game::sequence::CardSequence;


#[derive(PartialEq,Eq)]
pub enum CardKind {
    _Reserved(ReservedVal),
    Tableau(TableauVal),
    Standard(StandardSuit,StandardVal),
    Joker(JokerKind),
}




impl CardKind {

    pub fn is_numbered(&self) -> bool {
        match self {
            CardKind::Standard(_, standard_val) => standard_val.is_numbered(),
            _ => false
        }
    }
    pub fn is_face(&self) -> bool {
        match self {
            CardKind::Standard(_, standard_val) => standard_val.is_face(),
            _ => false
        }
    }

    pub(super) fn points(&self) -> i32 {
        match self {
            CardKind::_Reserved(_) => 0,
            CardKind::Tableau(_) => 0,
            CardKind::Standard(_, standard_val) => standard_val.points(),
            CardKind::Joker(joker_kind) => joker_kind.points(),
        }
    }

    pub(super) fn mult(&self) -> f32 {
        match self {
            CardKind::_Reserved(_) => 1.0,
            CardKind::Tableau(_) => 1.0,
            CardKind::Standard(_, standard_val) => standard_val.mult(),
            CardKind::Joker(_) => 1.0,
        }
    }

    pub(super) fn name(&self) -> String {
        match self {
            CardKind::Tableau(tableau_val) => format!("{}Tableau",tableau_val.name()),
            CardKind::Joker(joker_val) => joker_val.name(),
            CardKind::Standard(standard_suit, standard_val) =>
                format!("{} of {}",standard_val.name(),standard_suit.name()),
            CardKind::_Reserved(_) => todo!("Reserved cards have no name"),
        }
    }

    pub(super) fn desc(&self) -> Vec<String> {
        match self {
            CardKind::Tableau(tableau_val) => {
                let mut v = vec!["Cannot be picked up.".into()];
                v.extend(tableau_val.desc());
                v
            },
            CardKind::Joker(joker_val) => joker_val.desc(),
            CardKind::Standard(_, standard_val) => {
                if matches!(standard_val,StandardVal::Face(FaceVal::King)) {
                    return vec!["This card can be stacked on any non-face non-numbered card, including Tableaus".into()]
                }
                else {
                    return vec!["This card can be stacked on any other card of different color and one higher in value.".into()]
                }
            },
            CardKind::_Reserved(_) => todo!("Reserved cards have no description"),
        }
    }
}


//------------------------Reserved----------------------------
#[bitsize(4)]
#[derive(FromBits,Clone,Copy,PartialEq,Eq)]
pub enum ReservedVal {
    #[fallback]
    _Reserved = 0,
    __Reserved = 1,
    MouseCard = 2,
    BurnCard = 3,
    NullCard = 4,
}


//------------------------Tableau----------------------------
#[bitsize(4)]
#[derive(FromBits,Clone,Copy,PartialEq,Eq)]
pub enum TableauVal {
    Burn = 1,
    Shop = 2,
    #[fallback]
    Normal = 0,
}
impl TableauVal {
    pub fn is_valid(self, sequence: &CardSequence<'_>, pos: usize) -> Option<bool> {
        let stacked_on = sequence.get_stacked_on_from(pos);
        let is_being_picked_up = matches!(stacked_on.kind(), CardKind::_Reserved(ReservedVal::MouseCard));
        if is_being_picked_up { //Cannot be picked up
            return Some(false);
        }
        None
    }
    fn name(self) -> String {
        return match self {
            TableauVal::Burn => "Burn ",
            TableauVal::Shop => "Shop ",
            TableauVal::Normal => "",
        }.to_string();
    }
    fn desc(self) -> Vec<String> {
        match self {
            TableauVal::Burn => 
                vec!["Any card can be stacked on this one. After stacking a pile of cards over this one, they will burn".into()],
            TableauVal::Shop =>  
                vec!["All cards stacked on this one have their effects nullified, and will incur a cost in points when picked up".into()],
            TableauVal::Normal => vec![],
        }
    }
}

//------------------------Standard----------------------------
#[derive(Clone, Copy,PartialEq,Eq)]
pub enum StandardVal {
    Numbered(NumberVal),
    Face(FaceVal)
}
impl NumberVal {
    pub fn val_numeric(self) -> u8 {
        let raw_a: UInt<u8, 4> = self.into();
        return raw_a.value();
    }

}
impl StandardVal {
    pub fn order_numeric(self) -> u8 {
        let raw_a: UInt<u8, 4> = self.into();
        return raw_a.value();
    }
    pub fn is_valid(self, sequence: &CardSequence<'_>, pos: usize) -> Option<bool> {
        let stacked_on = sequence.get_stacked_on_from(pos);
        if stacked_on.order_numeric() != 0 && self.order_numeric() == stacked_on.order_numeric() - 1 {
            return Some(true);
        }
        None
    }

    fn is_numbered(self) -> bool {
        match self {
            StandardVal::Numbered(_) => true,
            StandardVal::Face(_) => false,
        }
    }
    fn is_face(self) -> bool {
        match self {
            StandardVal::Numbered(_) => false,
            StandardVal::Face(_) => true,
        }
    }
    fn points(self) -> i32 {
        match self {
            StandardVal::Numbered(number_val) => number_val.val_numeric() as u32 as i32,
            StandardVal::Face(_) => 10,
        }
    }
    fn mult(self) -> f32 {
        if matches!(self,StandardVal::Numbered(NumberVal::Ace)) {
            return 1.5;
        }
        return 1.0;
    }
    fn name(self) -> String {
        return match self {
            StandardVal::Numbered(NumberVal::Ace)   => "Ace",
            StandardVal::Numbered(NumberVal::Two)   => "Two",
            StandardVal::Numbered(NumberVal::Three) => "Three",
            StandardVal::Numbered(NumberVal::Four)  => "Four",
            StandardVal::Numbered(NumberVal::Five)  => "Five",
            StandardVal::Numbered(NumberVal::Six)   => "Six",
            StandardVal::Numbered(NumberVal::Seven) => "Seven",
            StandardVal::Numbered(NumberVal::Eight) => "Eight",
            StandardVal::Numbered(NumberVal::Nine)  => "Nine",
            StandardVal::Numbered(NumberVal::Ten)   => "Ten",
            StandardVal::Face(FaceVal::Jack)        => "Jack",
            StandardVal::Face(FaceVal::Queen)       => "Queen",
            StandardVal::Face(FaceVal::King)        => "King",
            _ => "",
        }.to_string();
    }
}
#[bitsize(4)]
#[derive(FromBits,Clone, Copy,PartialEq,Eq)]
pub enum StandardSuit {
    #[fallback]
    Hearts = 2,
    Diamonds = 3,
    Spades = 4,
    Clubs = 5
}
impl StandardSuit {
    pub fn is_valid(self, sequence: &CardSequence<'_>, pos: usize) -> Option<bool> {
        let stacked_on = sequence.get_stacked_on_from(pos);
        match self {
            StandardSuit::Hearts | StandardSuit::Diamonds => if matches!(stacked_on.kind(), CardKind::Standard(StandardSuit::Spades | StandardSuit::Clubs, _))
            {
                return Some(true);
            },
            StandardSuit::Spades | StandardSuit::Clubs => if matches!(stacked_on.kind(), CardKind::Standard(StandardSuit::Hearts | StandardSuit::Diamonds, _))
            {
                return Some(true);
            },
            _ => return None,
        };
        None
    }
    fn name(self) -> String {
        return match self {
            StandardSuit::Hearts => "Hearts",
            StandardSuit::Diamonds => "Diamonds",
            StandardSuit::Spades => "Spades",
            StandardSuit::Clubs => "Clubs",
        }.to_string();
    }
}
#[bitsize(4)]
#[derive(FromBits,Clone, Copy, PartialEq, Eq)]
pub enum NumberVal {
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
    #[fallback]
    Fallback = 15,
}
#[bitsize(4)]
#[derive(FromBits,Clone, Copy,PartialEq,Eq)]
pub enum FaceVal {
    Jack  = 11,
    Queen = 12,
    King  = 13,
    #[fallback]
    Fallback = 15,
}



//------------------------Joker----------------------------
#[derive(Clone, Copy,PartialEq,Eq)]
pub enum JokerKind {
    Blank,
    Joker(JLevel),
    Mimic(JLevel)
}
#[bitsize(1)]
#[derive(FromBits,Clone, Copy,PartialEq,Eq)]
pub enum JLevel {
    Low = 0,
    High = 1
}
impl JLevel {
    fn name(self) -> String {
        match self {
            JLevel::Low => "Low".into(),
            JLevel::High => "High".into(),
        }
    }
}
impl JokerKind {
    pub fn is_valid(self, sequence: &CardSequence<'_>, pos: usize) -> Option<bool> {
        let stacked_on = sequence.get_stacked_on_from(pos);
        match self {
            //Can be stacked on anything
            JokerKind::Joker(JLevel::Low) | JokerKind::Mimic(JLevel::Low) => {
                return Some(true)
            },
            //Can be stacked on low joker and tableaus
            JokerKind::Joker(JLevel::High) | JokerKind::Mimic(JLevel::High) => {
                // if !matches!(stacked_on.kind(),CardKind::Joker(JokerKind::Joker(JLevel::Low)) | CardKind::Tableau(_)) && 
                //     !matches!(stacked_on.kind(),CardKind::_Reserved(_)) {
                //     return Some(false);
                // }
                if matches!(stacked_on.kind(),CardKind::Joker(JokerKind::Joker(JLevel::Low)) | CardKind::Tableau(_))  {
                    return Some(true)
                }
                None
            },
            JokerKind::Blank => None
        }
    }
    fn points(self) -> i32 {
        match self {
            JokerKind::Blank => 0,
            JokerKind::Joker(_) => 7,
            JokerKind::Mimic(_) => 3,
        }
    }
    fn name(self) -> String {
        return match self {
            JokerKind::Blank => "Blank Card".into(),
            JokerKind::Joker(l) => format!("{} Joker",l.name()),
            JokerKind::Mimic(l) => format!("{} Mimic",l.name()),
        }.to_string();
    }
    fn desc(self) -> Vec<String> {
        match self {
            JokerKind::Blank => vec!["Does nothing".into()],
            JokerKind::Joker(JLevel::Low) => vec!["Can be stacked on any card".into()],
            JokerKind::Joker(JLevel::High) => vec!["Any card can be stacked on this one, with the exception of other High Jokers".into(),"Can be stacked on Tableaus or Low Jokers".into()],
            JokerKind::Mimic(JLevel::Low) => vec!["Can be stacked on any card. Turns intself into the card its stacked on".into()],
            JokerKind::Mimic(JLevel::High) => vec!["Any card can be stacked on this one, with the exception of other High Mimics".into(),"Will transform any card that stacks on this one into a copy of itself".into(),"Can be stacked on Tableaus or Low Jokers".into()],
        }
    }
}