const WORKGROUP_SIZE = 16;

const SUIT_HEARTS = 0;
const SUIT_SPADES = 1;
const SUIT_DIAMONDS = 2;
const SUIT_CLUBS = 3;

const VAL_ACE   = 1;
const VAL_TWO   = 2;
const VAL_THREE = 3;
const VAL_FOUR  = 4;
const VAL_FIVE  = 5;
const VAL_SIX   = 6;
const VAL_SEVEN = 7;
const VAL_EIGHT = 8;
const VAL_NINE  = 9;
const VAL_TEN   = 10;
const VAL_JACK  = 11;
const VAL_QUEEN = 12;
const VAL_KING  = 13;

const TYPE_CARD = 0;
const TYPE_CARD_HIDDEN = 1;
const TYPE_CARD_TABLEAU = 2;

const SUIT_BITSIZE: u32 = 2;
const VALUE_BITSIZE: u32 = 4;
const TYPE_BITSIZE: u32 = 2;

const SUIT_BITPOS: u32 = 0;
const VALUE_BITPOS: u32 = 2;
const TYPE_BITPOS: u32 = 6;

fn get_bits(value: u32, bitpos: u32, bitsize: u32) -> u32 {
    return (value >> bitpos) & ((1u << bitsize) - 1);
}

fn get_suit(card: Card) -> u32 {
    return get_bits(card.value,SUIT_BITPOS,SUIT_BITSIZE);
}
fn get_type(card: Card) -> u32 {
    return get_bits(card.value,TYPE_BITPOS,TYPE_BITSIZE);
}

fn get_value(card: Card) -> u32 {
    return get_bits(card.value,VALUE_BITPOS,VALUE_BITSIZE);
}


const MOUSE_TABLEAU_ID = 0;
const BURN_TABLEAU_ID = 1;
const DISCARD_PILE_ID = 2;

const RESERVED_TABLEAUS = 3;


fn get_world_position_and_size(c: Card, mouse_pos: vec2<f32>) -> vec4<f32> {
    var origin = vec2<f32>(50.0,50.0);

    let t_pos = c.tableau-RESERVED_TABLEAUS;
    let col = t_pos / 2;
    var is_bottom = (t_pos % 2);

    if (c.tableau == 0) { //Mouse tableau
        origin = mouse_pos;
        is_bottom = 0;
    }
    else if (c.tableau == 1) { //Discard pile
        origin = vec2<f32>(1100.0,200.0);
        is_bottom = 0;
    }
    else if (c.tableau == 2) { //Burn tableau
        origin = vec2<f32>(900.0,400.0);
        is_bottom = 0;
    }
    else {
        origin += vec2<f32>(120.0*f32(col),800.0*f32(is_bottom));
    }

    if (c.tableau == 2) {
        origin += vec2<f32>(3.0*f32(c.stack_idx) % 7.0,10.0*f32(c.stack_idx)*select(1.0,-1.0,is_bottom==1));
    }
    else {
        origin += vec2<f32>(0.0,40.0*f32(c.stack_idx)*select(1.0,-1.0,is_bottom==1));
    }

    
    var size = vec2<f32>(92.0,132.0);

    return vec4<f32>(origin,size);
}

fn bit_width(x: u32) -> u32 {
    return 32u - countLeadingZeros(x - 1u);
}
fn max_depth(max_cards: u32) -> u32 {
    return 256u * (1u << bit_width(max_cards)) - 1u;
}
fn get_depth(c: Card, max_cards: u32) -> u32 {
    let n = bit_width(max_cards);
    let mask = (1u << n) - 1u;
    let tableau = c.tableau << n;

    var card_z: u32 = ~c.stack_idx;

    card_z = card_z & mask;

    return tableau + card_z;
}


struct HoveringBuffer {
    hovering_id: u32,
    hovering_max_z: u32
}
struct AtomicHoveringBuffer {
    hovering_id: u32,
    hovering_max_z: atomic<u32>
}



struct Card {
    id: u32,
    value: u32,
    tableau: u32,
    stack_idx: u32,
    animation_id: u32,
    _pad: u32
}

struct CardArray {
    total: u32,
    total_workgroups: u32,
    _pad1: u32,
    _pad2: atomic<u32>,
    cards: array<Card>
}


struct Animation {
    prev_tableau: u32,
    prev_stack_idx: u32,
    t: f32,
    _pad: f32
}


