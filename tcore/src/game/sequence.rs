use std::collections::VecDeque;

use crate::game::card::{Card, card_kind::{CardKind, JLevel, JokerKind}};

enum DefaultCard {
    Def,
    A
}

pub struct CardSequence<'a> {
    sequence: Vec<&'a [Card]>,
    default: Card
}

pub struct CardSequenceMut<'a> {
    sequence: Vec<&'a mut [Card]>,
    default: Card
}

pub trait AsCardSequence<'a> {
    fn as_sequence(self, default: Card) -> CardSequence<'a>;
}
pub trait AsMutCardSequence<'a> {
    fn as_mut_sequence(self, default: Card) -> CardSequenceMut<'a>;
}

impl<'a> AsCardSequence<'a> for &'a [Card] {
    fn as_sequence(self, default: Card) -> CardSequence<'a> {
        CardSequence { sequence: vec![self], default: default }
    }
}
impl<'a> AsMutCardSequence<'a> for &'a mut [Card] {
    fn as_mut_sequence(self, default: Card) -> CardSequenceMut<'a> {
        CardSequenceMut { sequence: vec![self], default: default }
    }
}


impl<'a> CardSequenceMut<'a> {
    // pub fn get_card_at(&mut self, idx: usize) -> &mut Card {
    //     if let Some(c) = self.sequence.as_mut_slice().flat_index(idx) {
    //         return c
    //     }
    //     return &mut self.default;
    // }
    pub fn get_card_at(&self, idx: usize) -> &Card {
        let mut offset = idx;
        for slice in &self.sequence {
            if offset < slice.len() {
                return &slice[offset];
            }
            offset -= slice.len();
        }
        &self.default
    }
    pub fn get_copy_card_at(&self, idx: usize) -> Card {
        if let Some(c) = self.sequence.as_slice().flat_index(idx) {
            return c.clone();
        }
        return self.default.clone();
    }
    pub fn set_card_at(&mut self, idx: usize, card: Card) {
        if let Some(c) = self.sequence.as_mut_slice().flat_index_mut(idx) {
            *c = card;
        }
        self.default = card;
    }
    fn update(&mut self, mut to_update_next: VecDeque<usize>) {
        for _ in 0..1000 {
            let Some(pos) = to_update_next.pop_front() else {
                return;
            };
            let mut card = self.get_copy_card_at(pos);

            let next = card.update(self, pos);

            to_update_next.extend(next);

            self.set_card_at(pos, card);
        }
    }
    pub fn update_at(&mut self, idx: usize) {
        let mut to_update_next: VecDeque<usize> = VecDeque::new();
        to_update_next.push_back(idx);
        to_update_next.push_back(idx+1);
        self.update(to_update_next);
    }
    pub fn update_end(&mut self) {
        let mut to_update_next: VecDeque<usize> = VecDeque::new();
        to_update_next.push_back(self.sequence.as_slice().flat_len()-1);
        self.update(to_update_next);
    }
    pub fn is_last_position(&self, idx: usize) -> bool {
        idx == self.sequence.as_slice().flat_len()-1
    }
    pub fn get_stacked_on_from(&self, pos: usize) -> &Card {
        let stacked_on = self.get_card_at(pos.wrapping_sub(1));
        // Ghost cards: Cards stakced on them behave as if they were stacked on the one on top
        if stacked_on.ghost() {
            return self.get_card_at(pos.wrapping_sub(2));
        }
        return stacked_on;
    }
}

impl<'a> CardSequence<'a> {
    fn is_valid_sequence(&self) -> bool {
        for (i,card) in self.sequence.as_slice().flat_enumerate() {
            if !card.is_valid(self, i) {
                return false;
            }
        }
        true
    }
    pub fn is_valid_sequence_from(&self, from: usize) -> bool {
        log_print!("{}",self.sequence.as_slice().flat_len());
        if self.sequence.as_slice().flat_len() == 1 {return true;}
        assert!(from < self.sequence.as_slice().flat_len(), "from index {} out of bounds (len {})", from, self.sequence.as_slice().flat_len());
        for (i,card) in self.sequence.as_slice().flat_enumerate().skip(from) {
            if !card.is_valid(self, i) {
                return false;
            }
        }
        true
    }
    pub fn can_be_placed_on(self, mut placing_onto: Self) -> bool {
        let onto_len = placing_onto.sequence.as_slice().flat_len();

        placing_onto.sequence.extend(self.sequence.iter());
        placing_onto.is_valid_sequence_from(onto_len)
    }
    pub fn get_card_at(&self, idx: usize) -> &Card {
        if let Some(c) = self.sequence.as_slice().flat_index(idx) {
            return c
        }
        return &self.default;
    }
    pub fn get_stacked_on_from(&self, pos: usize) -> &Card {
        let stacked_on = self.get_card_at(pos.wrapping_sub(1));
        // Ghost cards: Cards stakced on them behave as if they were stacked on the one on top
        if stacked_on.ghost() {
            return self.get_card_at(pos.wrapping_sub(2));
        }
        return stacked_on;
    }
}


trait SequenceOpMut<'a, T: 'a> {
    fn flat_index_mut(&mut self, i: usize) -> Option<&mut T>;
}

impl <'a,T> SequenceOpMut<'a,T> for &mut [&'a mut [T]] {
    fn flat_index_mut(&mut self, i: usize) -> Option<&mut T> {
        let mut offset = i;
        for slice in self.iter_mut() {
            if offset < slice.len() {
                return slice.get_mut(offset);
            }
            offset -= slice.len();
        }
        None
    }
}

trait SequenceOpNoLifetime<'a, T: 'a> {
    //IMPORTANT: Return type has lifetime 'self version. 
    //This reference can only be used as long as sequence lives,
    //NOT as long as the slice of cards lives
    // fn flat_last(&mut self) -> Option<&mut T>;
    fn flat_index(&self, i: usize) -> Option<& T>;
    fn flat_len(&self) -> usize;
}

impl <'a,T> SequenceOpNoLifetime<'a,T> for &[&'a mut [T]] {
    fn flat_len(&self) -> usize {
        self.iter().map(|s| s.len()).sum()
    }
    fn flat_index(&self, i: usize) -> Option<& T> {
        let mut offset = i;
        for slice in *self {
            if offset < slice.len() {
                return slice.get(offset);
            }
            offset -= slice.len();
        }
        None
    }
}

// impl<'a, T> SequenceOpMut<'a, T> for &mut [&'a mut [T]] {
//     fn flat_last(&mut self) -> Option<&mut T> {
//         if let Some(slice) = self.last_mut() {
//             return slice.last_mut();
//         }
//         None
//     }
//     fn flat_index(&mut self, i: usize) -> Option<&mut T> {
//         let mut offset = i;
//         for slice in self.iter_mut() {
//             if offset < slice.len() {
//                 return slice.get_mut(offset);
//             }
//             offset -= slice.len();
//         }
//         None
//     }
//     fn flat_len(&self) -> usize {
//         self.iter().map(|s| s.len()).sum()
//     }
// }


trait SequenceOp<'a, T: 'a> {
    fn flat_len(&self) -> usize;
    fn flat_last(&self) -> Option<&'a T>;
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
    fn flat_last(&self) -> Option<&'a T> {
        if let Some(slice) =  self.last() {
            if let Some(last) = slice.last() {
                return Some(last);
            }
        }
        return None;
    }
}