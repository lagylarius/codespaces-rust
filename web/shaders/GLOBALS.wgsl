const WORKGROUP_SIZE = 16;

const SUIT_RESERVED = 0;
const SUIT_TABLEAU = 1;
const SUIT_HEARTS = 2;
const SUIT_SPADES = 3;
const SUIT_DIAMONDS = 4;
const SUIT_CLUBS = 5;

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

const SUIT_BITSIZE: u32 = 4;
const VALUE_BITSIZE: u32 = 4;
const FLAGS_BITSIZE: u32 = 8;

const FLAG_HIDDEN:      u32 = 1u;
const FLAG_BLOODY:      u32 = 2u;
const FLAG_GHOST:      u32 = 3u;
const FLAG_INVERTED:      u32 = 4u;
const FLAG_HYPER:      u32 = 5u;
const FLAG_METAL:      u32 = 5u;

const SUIT_BITPOS: u32 = 0;
const VALUE_BITPOS: u32 = 4;
const FLAGS_BITPOS: u32 = 8;

fn get_bits(value: u32, bitpos: u32, bitsize: u32) -> u32 {
    return (value >> bitpos) & ((1u << bitsize) - 1);
}

fn get_suit(card: Card) -> u32 {
    return get_bits(card.value,SUIT_BITPOS,SUIT_BITSIZE);
}
fn get_flags(value: u32) -> u32 {
    return get_bits(value, FLAGS_BITPOS, FLAGS_BITSIZE);
}
fn flag_mask(flag: u32) -> u32 {
    return 1u << (flag - 1u);
}
fn has_flag(card: Card, flag: u32) -> bool {
    let flags = get_flags(card.value);
    return (flags & flag_mask(flag)) != 0u;
}
fn is_hidden(card: Card) -> bool {
    return has_flag(card, FLAG_HIDDEN);
}

fn get_value(card: Card) -> u32 {
    return get_bits(card.value,VALUE_BITPOS,VALUE_BITSIZE);
}


const MOUSE_TABLEAU_ID = 0;
const BURN_TABLEAU_ID = 1;
const DISCARD_PILE_ID = 2;

const RESERVED_TABLEAUS = 3;

fn zoom_point(p: vec2<f32>, center: vec2<f32>, zoom: f32) -> vec2<f32> {
    return center + (p - center) * zoom;
}
fn unzoom_point(p: vec2<f32>, center: vec2<f32>, zoom: f32) -> vec2<f32> {
    return center + (p - center) / zoom;
}

fn compute_zoom_to_fit(max_cards: u32, screen_height: f32) -> f32 {
    let required = compute_required_height(max_cards);
    let required_zoom = (screen_height*0.66) / required;
    let zoom = clamp(required_zoom,0.26,1.0);
    //Snap to nice ratios
    let steps = array<f32,10>(1.0, 0.875, 0.75, 0.667, 0.625, 0.5, 0.375, 0.333, 0.25, 0.125);
    var best = steps[9];
    for (var i = 0u; i < 10u; i++) {
        if (steps[i] <= zoom) {
            best = steps[i];
            break;
        }
    }
    return best;
}

const CARD_STACK_ORIGIN = vec2<f32>(50.0,50.0);
const CARD_STACK_OFFSET_Y = 40.0;
const CARD_SIZE = vec2<f32>(92.0, 132.0);

fn compute_required_height(max_cards: u32) -> f32 {
    let stack_offset = CARD_STACK_OFFSET_Y * f32(max_cards - 1u);
    let card_height = CARD_SIZE.y;
    let origin_y = CARD_STACK_ORIGIN.y;
    return origin_y + stack_offset + card_height;
}

fn get_world_position_and_size(c: Card, mouse_pos: vec2<f32>,resolution:vec2<f32>) -> vec4<f32> {
    var origin = CARD_STACK_ORIGIN;

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
        origin += vec2<f32>(120.0*f32(col),(resolution.y-CARD_STACK_ORIGIN.y*2-CARD_SIZE.y)*f32(is_bottom));
    }

    if (c.tableau == 1) {
        origin += vec2<f32>(3.0*f32(c.stack_idx) % 7.0,10.0*f32(c.stack_idx)*select(1.0,-1.0,is_bottom==1));
    }
    else {
        origin += vec2<f32>(0.0,CARD_STACK_OFFSET_Y*f32(c.stack_idx)*select(1.0,-1.0,is_bottom==1));
    }

    return vec4<f32>(origin,CARD_SIZE);
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
    pos_x: u32,
    pos_y: u32,
    hovering_max_z: u32,
}
struct AtomicHoveringBuffer {
    hovering_id: u32,
    pos_x: u32,
    pos_y: u32,
    hovering_max_z: atomic<u32>,    
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
    max_cards_on_one_tableau: u32,
    _pad2: u32,
    cards: array<Card>
}


struct Animation {
    prev_tableau: u32,
    prev_stack_idx: u32,
    t: f32,
    _pad: f32
}


