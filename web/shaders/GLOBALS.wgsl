const WORKGROUP_SIZE = 16;

const SUIT_SPADES = 0x0;
const SUIT_HEARTS = 0x1;
const SUIT_CLUBS = 0x2;
const SUIT_DIAMONDS = 0x3;

const VAL_ACE   = 0x10;
const VAL_TWO   = 0x20;
const VAL_THREE = 0x30;
const VAL_FOUR  = 0x40;
const VAL_FIVE  = 0x50;
const VAL_SIX   = 0x60;
const VAL_SEVEN = 0x70;
const VAL_EIGHT = 0x80;
const VAL_NINE  = 0x90;
const VAL_TEN   = 0xA0;
const VAL_JACK  = 0xB0;
const VAL_QUEEN = 0xC0;
const VAL_KING  = 0xD0;

const TYPE_CARD = 0x100;
const TYPE_CARD_HIDDEN = 0x200;
const TYPE_CARD_TABLEAU = 0x300;

const SUIT_MASK: u32 =  0x00Fu;
const VALUE_MASK: u32 = 0x0F0u;
const TYPE_MASK: u32 =  0xF00u;

fn get_suit(card: Card) -> u32 {
    return card.value & SUIT_MASK;
}

fn get_type(card: Card) -> u32 {
    return card.value & TYPE_MASK;
}

fn get_value(card: Card) -> u32 {
    return (card.value & VALUE_MASK);
}
fn get_value_numeric(card: Card) -> u32 {
    return (card.value & VALUE_MASK) >> 4u;
}

fn get_world_position_and_size(c: Card, mouse_pos: vec2<f32>) -> vec4<f32> {
    var origin = vec2<f32>(50.0,50.0);

    if (c.tableau != 255u) {
        origin += vec2<f32>(120.0*f32(c.tableau),0.0);
    }
    else {
        origin = mouse_pos;
    }

    origin += vec2<f32>(0.0,30.0*f32(c.stack_idx));
    
    var size = vec2<f32>(92.0,132.0);

    return vec4<f32>(origin,size);
}

struct Card {
    value: u32,
    tableau: u32,
    stack_idx: u32,
    _pad: u32
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