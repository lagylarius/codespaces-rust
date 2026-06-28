pub mod card_kind;

use bilge::prelude::*;
use crate::game::sequence::CardSequenceMut;
use crate::game::{sequence::CardSequence};
use crate::game::card::card_kind::*;
use std::sync::atomic::{AtomicU32, Ordering};


static NEXT_ID: AtomicU32 = AtomicU32::new(0);

fn next_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

pub const STANDARD_SUITS: [StandardSuit; 4] = [StandardSuit::Hearts, StandardSuit::Spades, StandardSuit::Diamonds, StandardSuit::Clubs];
pub const STANDARD_VALS: [StandardVal; 13] = [
    StandardVal::Numbered(NumberVal::Ace),
    StandardVal::Numbered(NumberVal::Two),
    StandardVal::Numbered(NumberVal::Three),
    StandardVal::Numbered(NumberVal::Four),
    StandardVal::Numbered(NumberVal::Five),
    StandardVal::Numbered(NumberVal::Six),
    StandardVal::Numbered(NumberVal::Seven),
    StandardVal::Numbered(NumberVal::Eight),
    StandardVal::Numbered(NumberVal::Nine),
    StandardVal::Numbered(NumberVal::Ten),
    StandardVal::Face(FaceVal::Jack),
    StandardVal::Face(FaceVal::Queen),
    StandardVal::Face(FaceVal::King),
];

// macro_rules! flag_get_set {
//     ($name:ident) => {
//         paste::paste! {
//             pub fn $name(&self) -> bool {
//                 self.flags().$name()
//             }

//             pub fn [<set_ $name>](&mut self, v: bool) {
//                 let mut flags = self.flags();
//                 flags.[<set_ $name>](v);
//                 self.set_flags(flags);
//             }
//         }
//     };
// }
macro_rules! flag_get_set {
    ($name:ident, $setter:ident) => {
        pub fn $name(&self) -> bool {
            self.flags().$name()
        }
        pub fn $setter(&mut self, v: bool) {
            let mut flags = self.flags();
            flags.$setter(v);
            self.set_flags(flags);
        }
    };
}

#[bitsize(8)]
#[derive(FromBits,Clone, Copy,PartialEq,Eq)]
pub struct CardFlags {
    pub hidden: bool,
    pub bloody: bool,
    pub ghost: bool,
    pub inverted: bool,
    pub hyper: bool,
    pub metal: bool,
    pub _pad4: bool,
    pub _pad5: bool
}
impl CardFlags {
    fn empty() -> Self {
        CardFlags::from(u8::ZERO)
    }
}





#[bitsize(64)]
#[derive(FromBits,PartialEq,Clone, Copy)]
pub struct Card {
    pub id: u32,
    pub suit: SuitBitPack,
    pub val: u4,
    pub flags: CardFlags,
    pad: u16,
}



impl Card {
    pub fn set_hidden(&mut self, v: bool) {
        let mut flags = self.flags();
        flags.set_hidden(v);
        self.set_flags(flags);
    }
    pub fn hidden(&self) -> bool {
        self.flags().hidden()
    }
    flag_get_set!(ghost,set_ghost);
    flag_get_set!(bloody,set_bloody);
    flag_get_set!(inverted,set_inverted);
    flag_get_set!(hyper,set_hyper);
    flag_get_set!(metal,set_metal);

    pub fn copy_from(&mut self, other: &Card) {
        self.set_suit(other.suit());
        self.set_val(other.val());
        self.set_flags(other.flags());
    }

    pub fn new_common(v: CardKind) -> Self {
        let (suit,value) = v.into();
        Card::new(
            next_id(), 
            suit,
            value, 
            CardFlags::empty(),
            u16::ZERO,
        )
    }
    pub fn new_tableau(v: TableauVal) -> Self {
        Self::new_common(CardKind::Tableau(v))
    }
    pub fn new_card(s: StandardSuit,v:StandardVal) -> Self {
        Self::new_common(CardKind::Standard(s,v))
    }
    pub fn get_bits(&self) -> u64 {
        return self.value;
    }

    pub fn kind(&self) -> CardKind {
        (self.suit(),self.val()).into()
    }

    pub fn is_numbered_or_face(&self) -> bool {
        let k = self.kind();
        k.is_numbered() || k.is_face()
    }

    pub fn order_numeric(&self) -> u8 {
        let raw_a: UInt<u8, 4> = self.val().into();
        return raw_a.value();
    }



    
    

    pub fn get_points(&self) -> Option<i32> {
        let mut base = self.kind().points();
        return (base != 0).then_some(base);
    }
    pub fn get_mult(&self) -> Option<f32> {
        let base = self.kind().mult();
        return (base != 1.0).then_some(base);
    }

    pub fn update<'a>(&mut self, sequence: &CardSequenceMut<'a>, pos: usize) -> Vec<usize> {
        let mut to_update_next = vec![];
        if self.hidden() && sequence.is_last_position(pos) {
            self.set_hidden(false);
            to_update_next.push(pos.wrapping_sub(1));
            to_update_next.push(pos+1);
        }
        
        let stacked_on = sequence.get_stacked_on_from(pos);

        if matches!(self.kind(),CardKind::Joker(JokerKind::Mimic(JLevel::Low))) {
            self.copy_from(stacked_on);
            to_update_next.push(pos.wrapping_sub(1));
            to_update_next.push(pos+1);
        }
        if matches!(stacked_on.kind(),CardKind::Joker(JokerKind::Mimic(JLevel::High))) {
            self.copy_from(stacked_on);
            to_update_next.push(pos.wrapping_sub(1));
            to_update_next.push(pos+1);
        }
        return to_update_next;
    }



    pub fn is_valid<'a>(&self, sequence: &CardSequence<'a>, pos: usize) -> bool {
        let mut stacked_on = sequence.get_stacked_on_from(pos);

        // //Hidden cards: Invalid
        if self.hidden() {
            return false;
        }
        
        let is_being_picked_up = matches!(stacked_on.kind(), CardKind::_Reserved(ReservedVal::MouseCard));
        let placing_on_burn = matches!(stacked_on.kind(), CardKind::Tableau(TableauVal::Burn));
        
        let mut default: bool = is_being_picked_up || placing_on_burn; //(or on burn tableau)

        if !matches!(self.kind(), CardKind::Joker(JokerKind::Joker(JLevel::High))) && matches!(stacked_on.kind(), CardKind::Joker(JokerKind::Joker(JLevel::High))) {
            default = true
        }
        if !matches!(self.kind(), CardKind::Joker(JokerKind::Mimic(JLevel::High))) && matches!(stacked_on.kind(), CardKind::Joker(JokerKind::Mimic(JLevel::High))) {
            default = true
        }
        if matches!(self.kind(), CardKind::Joker(JokerKind::Blank)) {
            default = true
        }

        //Any Some(false) => A rule has been broken, disallow
        //Any Some(true) => At least one rule explicitly says its okay, allow
        //All None => No rule says its okay, default to disallow
        if let CardKind::Standard(suit,val) = self.kind() {
            if !stacked_on.is_numbered_or_face() && val == StandardVal::Face(FaceVal::King) {
                default = true
            }
            else {
                let alternating = suit.is_valid(sequence, pos);
                let descending = val.is_valid(sequence, pos);
                if alternating.is_some_and(|r| !r) || descending.is_some_and(|r| !r) { return false; }
                if alternating == Some(true) && descending == Some(true) { default = true; }
            }
        }

        if let CardKind::Tableau(v) = self.kind() {
            let r = v.is_valid(sequence, pos);
            if r.is_some_and(|r| !r) {return false} //Any rule broken = invalid
            else if !r.is_none() {default = true} //All rules allowed = valid
        }

        if let CardKind::Joker(v) = self.kind() {
            let r = v.is_valid(sequence, pos);
            if r.is_some_and(|r| !r) {return false} //Any rule broken = invalid
            else if !r.is_none() {default = true} //All rules allowed = valid
        }

        default
    }

    pub fn name(&self) -> String {
        let mut prep: String = "".into();

        if self.ghost() { prep.push_str("Ghost "); }
        if self.bloody() { prep.push_str("Bloody "); }
        if self.hyper() { prep.push_str("Hyper "); }
        format!("{}{}",prep,self.kind().name())
    }

    pub fn desc(&self) -> Vec<String> {
        let mut info = vec![];
        let mut desc = self.kind().desc();
        for s in &mut desc {
            s.insert(0, '-');
        }

        info.extend(desc);

        if self.ghost() {
            info.push("Ghost card: Cards stacked on this one behave as if they are stacked on the card on top.".into())
        }

        info
    }



    pub fn get_info(&self) -> Vec<String> {
        if self.hidden() {
            return vec!["Face-down card".to_string(),
                        "Always invalid.".to_string(),
                        "This card will be revealed when there are no cards stacked on it.".to_string()
                    ]
        }

        let mut info = vec![self.name()];

        if let Some(p) = self.get_points() {
            let tag = if p < 0 { "red" } else { "green" };
            info.push(format!("<c={tag}>{:+} points</c>", p));
        }
        if let Some(m) = self.get_mult() {
            let tag = if m < 1.0 { "red" } else { "green" };
            info.push(format!("<c={tag}>x{} mult</c>", m));
        }

        info.extend(self.desc());

        return info;
    }
}





//------------------------------Bit packing--------------------------------------
#[bitsize(4)]
#[derive(FromBits,Clone, Copy,PartialEq,Eq)]
pub enum SuitBitPack {
    #[fallback]
    _Reserved = 0,
    Tableau = 1,
    Hearts = 2,
    Spades = 3,
    Diamonds = 4,
    Clubs = 5,
    Joker = 6,
}
impl From<CardKind> for (SuitBitPack,u4) {
    fn from(s: CardKind) -> Self {
        let (suit,val): (SuitBitPack,u4) = match s {
            CardKind::_Reserved(reserved_val) => (SuitBitPack::_Reserved,reserved_val.into()),
            CardKind::Tableau(tableau_val) => (SuitBitPack::Tableau,tableau_val.into()),
            CardKind::Standard(suit,val ) => (match suit {
                StandardSuit::Hearts => SuitBitPack::Hearts,
                StandardSuit::Diamonds => SuitBitPack::Diamonds,
                StandardSuit::Spades => SuitBitPack::Spades,
                StandardSuit::Clubs => SuitBitPack::Clubs,
            },val.into()),
            CardKind::Joker(joker_val) => (SuitBitPack::Joker,joker_val.into()),
        };
        (suit,val)
    }
}
impl From<(SuitBitPack,u4)> for CardKind {
    fn from(value: (SuitBitPack,u4)) -> Self {
        let (suit,val) = value;
        match suit {
            SuitBitPack::Tableau => CardKind::Tableau(val.into()),
            SuitBitPack::Joker =>   CardKind::Joker(val.into()),
            SuitBitPack::Clubs =>   CardKind::Standard(StandardSuit::Clubs, val.into()),
            SuitBitPack::Hearts =>  CardKind::Standard(StandardSuit::Hearts, val.into()),
            SuitBitPack::Diamonds =>CardKind::Standard(StandardSuit::Diamonds, val.into()),
            SuitBitPack::Spades =>  CardKind::Standard(StandardSuit::Spades, val.into()),
            SuitBitPack::_Reserved => CardKind::_Reserved(val.into()),
        }
    }
}
impl From<StandardVal> for u4 {
    fn from(s: StandardVal) -> Self {
        match s {
            StandardVal::Numbered(v) => v.into(),
            StandardVal::Face(v) => v.into(),
        }
    }
}
impl From<u4> for StandardVal {
    fn from(val: u4) -> Self {
        let v: u8 = val.value();
        if v >= 11 {
            StandardVal::Face(val.into())
        } else {
            StandardVal::Numbered(val.into())
        }
    }
}
impl From<JokerKind> for u4 {
    fn from(s: JokerKind) -> Self {
        u4::new(match s {
            JokerKind::Blank => 0,
            JokerKind::Joker(l) => 1 + u1::from(l).value(),
            JokerKind::Mimic(l) => 3 + u1::from(l).value(),
        })
    }
}
impl From<u4> for JokerKind {
    fn from(val: u4) -> Self {
        let v = val.value();
        match v {
            0 => JokerKind::Blank,
            1..=2 => JokerKind::Joker(u1::new(v - 1).into()),
            3..=4 => JokerKind::Mimic(u1::new(v - 3).into()),
            _ => unreachable!(),
        }
    }
}
