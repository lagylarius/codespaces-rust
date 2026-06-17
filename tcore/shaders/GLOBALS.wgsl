const WORKGROUP_SIZE = 16;

const SUIT_HEARTS = 0x00;
const SUIT_SPADES = 0x10;
const SUIT_DIAMONDS = 0x20;
const SUIT_CLUBS = 0x30;

const VAL_ACE   = 0x01;
const VAL_TWO   = 0x02;
const VAL_THREE = 0x03;
const VAL_FOUR  = 0x04;
const VAL_FIVE  = 0x05;
const VAL_SIX   = 0x06;
const VAL_SEVEN = 0x07;
const VAL_EIGHT = 0x08;
const VAL_NINE  = 0x09;
const VAL_TEN   = 0x0A;
const VAL_JACK  = 0x0B;
const VAL_QUEEN = 0x0C;
const VAL_KING  = 0x0D;

const TYPE_CARD = 0x100;
const TYPE_CARD_HIDDEN = 0x200;
const TYPE_CARD_TABLEAU = 0x300;

const VALUE_MASK: u32 = 0x00Fu;
const SUIT_MASK: u32 =  0x0F0u;
const TYPE_MASK: u32 =  0xF00u;

fn get_suit(card: Card) -> u32 {
    return (card.value & SUIT_MASK) >> 4;
}
fn get_suit_numeric(card: Card) -> u32 {
    return (card.value & SUIT_MASK) >> 4;
}

fn get_type(card: Card) -> u32 {
    return card.value & TYPE_MASK;
}

fn get_value(card: Card) -> u32 {
    return (card.value & VALUE_MASK);
}
fn get_value_numeric(card: Card) -> u32 {
    return (card.value & VALUE_MASK);
}

fn get_world_position_and_size(c: Card, mouse_pos: vec2<f32>) -> vec4<f32> {
    var origin = vec2<f32>(50.0,50.0);

    if (c.tableau != 0) {
        origin += vec2<f32>(120.0*f32(c.tableau-1u),0.0);
    }
    else {
        origin = mouse_pos;
    }

    origin += vec2<f32>(0.0,30.0*f32(c.stack_idx));
    
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
    stack_idx: u32
}

struct CardArray {
    total: u32,
    total_workgroups: u32,
    hovering: u32,
    hovering_max_z: atomic<u32>,
    cards: array<Card>
}



struct ReadOnlyCardArray {
    total: u32,
    total_workgroups: u32,
    hovering: u32,
    hovering_max_z: u32,
    cards: array<Card>
}


struct Tableau {
    size: u32,
    _pad: vec3<u32>,
}