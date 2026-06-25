use crate::game::{card::{Card, CardType, Suit, Val}};

pub type CardSequence<'a> = &'a [&'a [Card]];



trait SequenceOp<'a, T: 'a> {
    fn flat_len(&self) -> usize;
    fn flat_index(&self, i: usize) -> Option<&'a T>;
    fn flat_enumerate(&self) -> impl Iterator<Item = (usize, &'a T)>;
    fn slices(&self) -> impl Iterator<Item = &'a [T]>;
}

impl<'a,T> SequenceOp<'a,T> for &[&'a [T]] {
    fn flat_len(&self) -> usize {
        self.iter().map(|s| s.len()).sum()
    }
    fn flat_index(&self, i: usize) -> Option<&'a T> {
        let mut offset = i;
        for slice in *self {
            if offset < slice.len() {
                return Some(&slice[offset]);
            }
            offset -= slice.len();
        }
        None
    }
    fn flat_enumerate(&self) -> impl Iterator<Item = (usize, &'a T)> {
        let mut offset = 0;
        self.iter().flat_map(move |&slice| {
            let start = offset;
            offset += slice.len();
            (0..slice.len()).map(move |i| (start + i, &slice[i]))
        })
    }
    fn slices(&self) -> impl Iterator<Item = &'a [T]> {
        self.iter().copied()
    }
}

impl<'a,T> SequenceOp<'a,T> for &'a [T] {
    fn flat_len(&self) -> usize {
        self.len()
    }
    fn flat_index(&self, i: usize) -> Option<&'a T> {
        self.get(i)
    }
    fn flat_enumerate(&self) -> impl Iterator<Item = (usize, &'a T)> {
        self.iter().enumerate()
    }
    fn slices(&self) -> impl Iterator<Item = &'a [T]> {
        std::iter::once(*self)
    }
}

pub trait CardSequenceOp<'a, T> {
    fn is_valid_sequence_from(&self,from: usize) -> bool;
    fn is_valid_sequence(&self) -> bool;
    fn is_valid_card(&self, card: &Card, pos: usize) -> bool;
    fn can_be_picked(&self) -> bool;
    fn can_be_placed_on<B>(&self, placing_onto: B) -> bool where B: SequenceOp<'a, Card>;
}

// pub fn can_be_placed_on<'a, A, B>(being_placed: A, placing_onto: B) -> bool
//     where 
//         A: SequenceOp<'a, Card>, 
//         B: SequenceOp<'a, Card>,
//     {
//     if placing_onto.flat_index(0).is_some_and(|c| {c.tableau_is_burn()}) {return true;}

//     let onto_len = placing_onto.flat_len();

//     let mut combined: Vec<&[Card]> = placing_onto.slices().collect();
//     combined.extend(being_placed.slices());
//     let combined: &[&[Card]] = combined.as_slice();
//     combined.is_valid_sequence_from(onto_len)
// }

impl<'a,S> CardSequenceOp<'a,S> for S where S: SequenceOp<'a,Card>  {
    fn can_be_placed_on<B>(&self, placing_onto: B) -> bool where B: SequenceOp<'a, Card> {
        //Can always place on burnt tableau
        if placing_onto.flat_index(0).is_some_and(|c| c.tableau_is_burn()) { return true; }
        let onto_len = placing_onto.flat_len();
        let mut combined: Vec<&[Card]> = placing_onto.slices().collect();
        combined.extend(self.slices());
        let combined: &[&[Card]] = combined.as_slice();
        combined.is_valid_sequence_from(onto_len)
    }
    fn is_valid_sequence_from(&self, from: usize) -> bool {
        assert!(from < self.flat_len(), "from index {} out of bounds (len {})", from, self.flat_len());
        for (i,card) in self.flat_enumerate().skip(from) {
            if !self.is_valid_card(card, i) {
                return false;
            }
        }
        true
    }
    fn is_valid_sequence(&self) -> bool {
        for (i,card) in self.flat_enumerate() {
            if !self.is_valid_card(card, i) {
                return false;
            }
        }
        true
    }
    fn is_valid_card(&self, card: &Card, pos: usize) -> bool {
        let stacked_on = self.flat_index(pos.wrapping_sub(1));

        match card.t() {
            //Numbered/face cards: Can be stacked on other numbered/face cards, with one number lower and of different color
            CardType::Card => {
                //besides kings, which can be stacked on any card that is not numbered/face
                if let Some(stacked_on) = stacked_on {
                    let is_king_over_non_card = stacked_on.t() != CardType::Card && card.val() == Val::King;

                    let stacked_on_card = stacked_on.t() == CardType::Card;
                    let is_on_alternate_color = match card.suit() {
                        Suit::Hearts | Suit::Diamonds => stacked_on.suit() == Suit::Spades || stacked_on.suit() == Suit::Clubs,
                        Suit::Spades | Suit::Clubs => stacked_on.suit() == Suit::Hearts || stacked_on.suit() == Suit::Diamonds,
                    };
                    let is_on_descending = stacked_on.val_numeric() != 0 && card.val_numeric() == stacked_on.val_numeric() - 1;
                    return is_king_over_non_card || (stacked_on_card && is_on_alternate_color && is_on_descending);
                }
                return true;
            },
            //Hidden cards: Cannot be stacked on any card. Cannot have any card below. 
            // Will unhid if they are no cards below this one. 
            CardType::Hidden => {
                return false
            }
            //Tableau: Cannot be stacked on any card. Cannot have any card below.
            CardType::Tableau => {
                if let Some(_) = stacked_on {
                    return false;
                }
                else {
                    return false;
                }
            }
            CardType::_Pad => false,
        }
    }
    fn can_be_picked(&self) -> bool {
        self.is_valid_sequence()
    }
}