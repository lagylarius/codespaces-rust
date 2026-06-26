
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

fn next_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}


use bilge::prelude::*;


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
    Jack  = 11,
    Queen = 12,
    King  = 13,
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

#[derive(Clone, Copy,PartialEq,Eq)]
pub enum StandardVal {
    Numbered(NumberVal),
    Face(FaceVal)
}
impl StandardVal {
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


#[bitsize(4)]
#[derive(FromBits,Clone,Copy,PartialEq,Eq)]
pub enum TableauVal {
    Burn = 1,
    Shop = 2,
    #[fallback]
    Normal = 0,
}
#[bitsize(4)]
#[derive(FromBits,Clone,Copy,PartialEq,Eq)]
pub enum JokerVal {
    Low = 1,
    High = 2,
    #[fallback]
    Pad0 = 0,
}


#[derive(Clone, Copy,PartialEq,Eq)]
pub enum StandardSuit {
    Hearts,
    Diamonds,
    Spades,
    Clubs
}







#[bitsize(4)]
#[derive(FromBits,Clone, Copy,PartialEq,Eq)]
pub enum Suit {
    Tableau = 1,
    Hearts = 2,
    Spades = 3,
    Diamonds = 4,
    Clubs = 5,
    Joker = 6,
    #[fallback]
    _Reserved = 0,
}


#[bitsize(2)]
#[derive(FromBits,Clone, Copy,PartialEq,Eq)]
pub struct CardFlags {
    pub hidden: bool,
    pub _pad: bool
}
impl CardFlags {
    fn empty() -> Self {
        CardFlags::from(u2::ZERO)
    }
}



#[derive(PartialEq,Eq)]
pub enum CardKind {
    Tableau(TableauVal),
    Joker(JokerVal),
    Standard(StandardSuit,StandardVal)
}

impl CardKind {
    fn into_pack(self) -> (Suit, u4) {
        let (suit,val): (Suit,u4) = match self {
            CardKind::Tableau(tableau_val) => (Suit::Tableau,tableau_val.into()),
            CardKind::Joker(joker_val) => (Suit::Joker,joker_val.into()),
            CardKind::Standard(suit,val ) => (match suit {
                StandardSuit::Hearts => Suit::Hearts,
                StandardSuit::Diamonds => Suit::Diamonds,
                StandardSuit::Spades => Suit::Spades,
                StandardSuit::Clubs => Suit::Clubs,
            },val.into())
        };
        (suit,val)
    }
    fn from_pack(suit: Suit, val: u4) -> CardKind {
        match suit {
            Suit::Tableau => CardKind::Tableau(val.into()),
            Suit::Joker =>   CardKind::Joker(val.into()),
            Suit::Clubs =>   CardKind::Standard(StandardSuit::Clubs, val.into()),
            Suit::Hearts =>  CardKind::Standard(StandardSuit::Hearts, val.into()),
            Suit::Diamonds =>CardKind::Standard(StandardSuit::Diamonds, val.into()),
            Suit::Spades =>  CardKind::Standard(StandardSuit::Spades, val.into()),
            Suit::_Reserved => todo!(),
        }
    }

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
}






#[bitsize(64)]
#[derive(FromBits,PartialEq,Clone, Copy)]
pub struct Card {
    pub id: u32,
    pub suit: Suit,
    pub val: u4,
    pub flags: CardFlags,
    pad: u22,
}



impl Card {
    pub fn new_common(v: CardKind) -> Self {
// let v = StandardVal::Numbered(NumberVal::Ace);
// let bits: u4 = v.into();
// let back: StandardVal = bits.into();
// assert_eq!(v, back);

        let (suit,value) = v.into_pack();
        Card::new(
            next_id(), 
            suit,
            value, 
            CardFlags::empty(),
            u22::ZERO,
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
        CardKind::from_pack(self.suit(), self.val())
    }

    pub fn is_numbered_or_face(&self) -> bool {
        let k = self.kind();
        k.is_numbered() || k.is_face()
    }

    pub fn val_numeric(&self) -> u8 {
        let raw_a: UInt<u8, 4> = self.val().into();
        return raw_a.value();
    }
    pub fn hide(&mut self) {
        let mut flags = self.flags();
        flags.set_hidden(true);
        self.set_flags(flags);
    }
    pub fn unhide(&mut self) {
        let mut flags = self.flags();
        flags.set_hidden(false);
        self.set_flags(flags);
    }
    pub fn is_hidden(&self) -> bool {
        self.flags().hidden()
    }

    // pub fn new_card(suit: Suit, value: Val) -> Card {
    //     Card::new(
    //         next_id(), 
    //         suit,
    //         value, 
    //         CardFlags::empty(),
    //         u22::ZERO,
    //     )
    // }
    // pub fn tableau_type(&self) -> Option<TableauVal> {
    //     if self.suit() != Suit::Tableau {
    //         return None;
    //     }
    //     return Some(self.val().into());
    // }
    // pub fn new_tableau(value: TableauVal) -> Card {
    //     Card::new(
    //         next_id(), 
    //         Suit::Tableau,
    //         Val::from(value), 
    //         CardFlags::empty(), 
    //         u22::ZERO,
    //     )
    // }
    // pub fn is_numbered_or_face(&self) -> bool {
    //     if self.flags().hidden() {
    //         return false;
    //     }
    //     match self.suit() {
    //         Suit::Tableau | Suit::Joker => false,
    //         Suit::Hearts | Suit::Spades | Suit::Diamonds | Suit::Clubs => {
    //             return self.val_numeric() >= 1;
    //         },
    //         Suit::_Reserved => false,
    //     }
    // }
    // pub fn get_color(&self) -> u32 {
    //     let bit = match self.suit() {
    //         Suit::Hearts | Suit::Diamonds => 0,
    //         Suit::Spades | Suit::Clubs => 1,
    //         Suit::Tableau => 99,
    //         Suit::_Reserved => 99,
    //         Suit::Joker => 99,
    //     };
    //     return bit
    // }

    // pub fn tableau_is_burn(&self) -> bool {
    //     self.tableau_type().is_some_and(|t| t == TableauVal::Burn)
    // }

    // pub fn val_numeric(&self) -> u8 {
    //     let raw_a: UInt<u8, 4> = self.val().into();
    //     return raw_a.value();
    // }




    // pub fn multitype_val(&self) -> CardTypeVal {
    //     match self.suit() {
    //         Suit::Tableau => CardTypeVal::Tableau(self.val().into()),
    //         Suit::Hearts | Suit::Spades | Suit::Diamonds | Suit::Clubs => CardTypeVal::Standard(self.val()),
    //         Suit::_Reserved => CardTypeVal::Standard(self.val()),
    //         Suit::Joker => CardTypeVal::Standard(self.val()),
    //     }
    // }



    // pub fn get_points(&self) -> Option<i32> {
    //     if self.is_hidden() {return None;}
    //     let base: i32 = match self.multitype_val() {
    //         CardTypeVal::Tableau(val) => {
    //             match val {
    //                 TableauVal::Burn => -10,
    //                 _ => 0,
    //             }
    //         },
    //         CardTypeVal::Standard(val) => {
    //             match val {
    //                 Val::Jack => 10,
    //                 Val::Queen => 10,
    //                 Val::King => 10,
    //                 _ => self.val_numeric() as u32 as i32
    //             }
    //         },
    //         CardTypeVal::Joker(joker_val) => 0,
    //     };

    //     (base != 0).then_some(base)
    // }

    // pub fn get_mult(&self) -> Option<f32> {
    //     if self.is_hidden() {return None;}

    //     let base: f32 = match self.multitype_val() {
    //         CardTypeVal::Tableau(_) => {
    //             1.0
    //         },
    //         CardTypeVal::Standard(val) => {
    //             match val {
    //                 Val::Ace => 2.0,
    //                 _ => 1.0
    //             }
    //         },
    //         CardTypeVal::Joker(joker_val) => 1.0,
    //     };
        
    //     (base != 1.0).then_some(base)
    // }



    pub fn get_info(&self) -> Vec<String> {
        if self.is_hidden() {
            return vec!["Face-down card".to_string(),
                        "Cannot be stacked on any card. No card can stack on this one.".to_string(),
                        "This card will be revealed when there are no cards stacked on it.".to_string()
                    ]
        }

        let mut name = "".to_string();

        match self.kind() {
            CardKind::Tableau(tableau_val) => {
                name += match tableau_val {
                    TableauVal::Burn => "Burn ",
                    TableauVal::Shop => "Shop ",
                    TableauVal::Normal => "",
                };
                name += "Tableau"
            },
            CardKind::Joker(joker_val) => {
                name += match joker_val {
                    JokerVal::Low => "Low ",
                    JokerVal::High => "High ",
                    _ => ""
                };
                name += "Joker"
            },
            CardKind::Standard(standard_suit, standard_val) => {
                name += match standard_val {
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
                };
                name += " of ";
                name += match standard_suit {
                    StandardSuit::Hearts => "Hearts",
                    StandardSuit::Diamonds => "Diamonds",
                    StandardSuit::Spades => "Spades",
                    StandardSuit::Clubs => "Clubs",
                }
            },
        };

        let mut info = vec![name];

        // if let Some(p) = self.get_points() {
        //     let tag = if p < 0 { "red" } else { "green" };
        //     info.push(format!("<c={tag}>{:+} points</c>", p));
        // }
        // if let Some(m) = self.get_mult() {
        //     let tag = if m < 1.0 { "red" } else { "green" };
        //     info.push(format!("<c={tag}>x{} mult</c>", m));
        // }

        match self.kind() {
            CardKind::Tableau(tableau_val) => {
                info.push("Cannot be stacked on any card. Cannot be picked up.".into());
                match tableau_val {
                    TableauVal::Burn => 
                        info.push("Any card can be stacked on this one. After stacking cards, they will burn giving you points based on the cards abilities".into()),
                    TableauVal::Shop => {
                        info.push("All cards stacked on this one incur a cost in points when picked up and have their effects nullified".into());
                    },
                    TableauVal::Normal => {},
                };
            },
            CardKind::Joker(joker_val) => {
                match joker_val {
                    JokerVal::Low =>  {            
                        info.push("Can be stacked on any card".into());
                    },
                    JokerVal::High => {            
                        info.push("Any card can be stacked on this one".into());
                        info.push("Can be stacked on Tableaus or Low Jokers".into());
                    },
                    JokerVal::Pad0 => {
                        // HokerVal::Blank info.push("Can be stacked on a card if any other unhidden card on the board can be stacked on it".into());
                    },
                }
            },
            CardKind::Standard(standard_suit, standard_val) => {
                if matches!(standard_val,StandardVal::Face(FaceVal::King)) {
                    info.push("This card can be stacked on any non-face non-numbered card, including Tableaus".into())
                }
                else {
                    info.push("This card can be stacked on any other card of different color and one higher in value.".into())
                }
            },
        };

        return info;
    }
}
