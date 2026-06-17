#include "GLOBALS.wgsl"


struct RenderUniforms {
  resolution: vec2<f32>
};

struct InputUniforms {
    mouse_pos: vec2<f32>,
    pad: f32,
    pad1: f32
}

fn to_ndc(pixel: vec2<f32>) -> vec2<f32> {
    let v = (pixel / render_u.resolution) * 2.0 - 1.0;
    return vec2<f32>(v.x,-v.y);
}

@group(0) @binding(0) var<storage,read> card_data: ReadOnlyCardArray;
@group(0) @binding(1) var<uniform> render_u: RenderUniforms;
@group(0) @binding(2) var<uniform> input_u: InputUniforms;
@group(0) @binding(3) var<storage,read_write> hovering_buffer: HoveringBuffer;


@group(1) @binding(0) var textureSampler: sampler;
@group(1) @binding(1) var spritesheet: texture_2d<f32>;

struct VSOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) size: vec2<u32>,
    @location(2) @interpolate(flat) instanceIndex: u32,
    @location(3) @interpolate(flat) hovering: u32
};

// Texture atlas
const ATLAS_SIZE = vec2<f32>(460.0,660.0);
const TILE_SIZE = vec2<f32>(23.0,33.0);
fn getColorFromTexture(sprite_pos_x : u32, sprite_pos_y : u32, localUV : vec2<f32>) -> vec4<f32> {
    let clamped_uv = clamp(localUV,vec2<f32>(0.0),vec2<f32>(1.0));
    let uv = vec2<f32>(clamped_uv.x,1 - clamped_uv.y);
    
    
    let tileUvSize = TILE_SIZE / ATLAS_SIZE; // = (0.25, 0.25)

    let tileSpritePos = vec2<f32>(f32(sprite_pos_x),f32(sprite_pos_y));
    let tileUVOffset = tileSpritePos / (1/tileUvSize);;

    let tileUV = uv * tileUvSize + tileUVOffset;
    return textureSampleLevel(spritesheet, textureSampler, tileUV, 0.0);
}

fn textureIsAtUvDistanceOfSprite(sprite_pos_x : u32, sprite_pos_y : u32, localUV : vec2<f32>, distance: vec2<f32>) -> vec2<bool> {
    
    //Computes if at uv distance @distance there is an opaque cell on the sprite
    let right_uv = localUV + vec2<f32>(distance.x, 0.0);
    var right = getColorFromTexture(sprite_pos_x, sprite_pos_y, right_uv).a != 0.0;
    if (right_uv.x < 0.0 || right_uv.y < 0.0 || right_uv.x > 1.0 || right_uv.y > 1.0) {
        right = false;
    }

    let left_uv = localUV + vec2<f32>(-distance.x, 0.0);
    var left = getColorFromTexture(sprite_pos_x, sprite_pos_y, left_uv).a != 0.0;
    if (left_uv.x < 0.0 || left_uv.y < 0.0 || left_uv.x > 1.0 || left_uv.y > 1.0) {
        left = false;
    }

    let up_uv = localUV + vec2<f32>(0.0, distance.y);
    var up = getColorFromTexture(sprite_pos_x, sprite_pos_y, up_uv).a != 0.0;
    if (up_uv.x < 0.0 || up_uv.y < 0.0 || up_uv.x > 1.0 || up_uv.y > 1.0) {
        up = false;
    }

    let down_uv = localUV + vec2<f32>(0.0, -distance.y);
    var down = getColorFromTexture(sprite_pos_x, sprite_pos_y, down_uv).a != 0.0;
    if (down_uv.x < 0.0 || down_uv.y < 0.0 || down_uv.x > 1.0 || down_uv.y > 1.0) {
        down = false;
    }

    let up_right_uv = localUV + vec2<f32>( distance.x,  distance.y);
    var up_right = getColorFromTexture(sprite_pos_x, sprite_pos_y, up_right_uv).a != 0.0;
    if (up_right_uv.x < 0.0 || up_right_uv.y < 0.0 || up_right_uv.x > 1.0 || up_right_uv.y > 1.0) {
        up_right = false;
    }

    let up_left_uv = localUV + vec2<f32>(-distance.x,  distance.y);
    var up_left = getColorFromTexture(sprite_pos_x, sprite_pos_y, up_left_uv).a != 0.0;
    if (up_left_uv.x < 0.0 || up_left_uv.y < 0.0 || up_left_uv.x > 1.0 || up_left_uv.y > 1.0) {
        up_left = false;
    }

    let down_right_uv = localUV + vec2<f32>( distance.x, -distance.y);
    var down_right = getColorFromTexture(sprite_pos_x, sprite_pos_y, down_right_uv).a != 0.0;
    if (down_right_uv.x < 0.0 || down_right_uv.y < 0.0 || down_right_uv.x > 1.0 || down_right_uv.y > 1.0) {
        down_right = false;
    }

    let down_left_uv = localUV + vec2<f32>(-distance.x, -distance.y);
    var down_left = getColorFromTexture(sprite_pos_x, sprite_pos_y, down_left_uv).a != 0.0;
    if (down_left_uv.x < 0.0 || down_left_uv.y < 0.0 || down_left_uv.x > 1.0 || down_left_uv.y > 1.0) {
        down_left = false;
    }

    let border = up | right | down | left | down_left | up_left | down_right | up_right;
    return vec2<bool>(border,up_left); //Save upleft for shadowing for now, although i may want a bigger area so
}


const PIXEL_FACTOR = 4.0;
fn quantize(p: vec2<f32>, object_size: vec2<f32>) -> vec2<f32> {
    var p_px = p * object_size;

    p_px = floor(p_px / PIXEL_FACTOR) * PIXEL_FACTOR + 0.5 * PIXEL_FACTOR;

    return p_px / object_size;
}
fn quantized_pixel_size(object_size: vec2<f32>) -> vec2<f32> {
    return PIXEL_FACTOR / object_size;
}







const CARD_SPRITE_SIZE = vec2<f32>(92.0,132.0);

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;

    let uv_pixel = 1.0 / vec2<f32>(in.size);


    let quv = quantize(uv,CARD_SPRITE_SIZE);

    let c = card_data.cards[in.instanceIndex];

    var suit = get_suit(c);
    var value = get_value_numeric(c) + 1u;
    if (get_type(c) == TYPE_CARD_HIDDEN) {
        suit = 0u;
        value = 0u;
    }


    var outline_check_distance = uv_pixel*select(HOVER_OUTLINE_WIDTH,10.0,c.tableau == 0u);


    var color = getColorFromTexture(suit,value, quv);
    let check = textureIsAtUvDistanceOfSprite(suit,value,uv,outline_check_distance);
    let is_border = check.x;
    let is_shadow = check.y;


    if (uv.x > 1.0 || uv.y > 1.0 || uv.x < 0.0 || uv.y < 0.0) {
        color = vec4<f32>(1.0,1.0,1.0,0.0);
    }
    if (color.a == 0.0 && bool(in.hovering) && is_border) {
        if (c.tableau != 0u) {
            return vec4<f32>(HIGHLIGHT_COLOR,0.99);
        }
        else if is_shadow {
            return vec4<f32>(0.0,0.0,0.0,0.4);
        }
    }

    return color;
}

const HIGHLIGHT_COLOR = vec3<f32>(1.0, 0.867, 0.0);
// const HIGHLIGHT_COLOR = vec3<f32>(1.0, 1.0, 0.0);

const MAX_TABLEAUS = 255u;


const HOVER_OUTLINE_WIDTH = 3.0;

@vertex
fn vs_main(    
    @builtin(vertex_index) vertexIndex: u32,
    @builtin(instance_index) instanceIndex: u32
    ) -> VSOut {
    var pos = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>( 1.0, 1.0)
    );

    var uv = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 1.0), // bottom-left
        vec2<f32>(1.0, 1.0), // bottom-right
        vec2<f32>(0.0, 0.0), // top-left
        vec2<f32>(1.0, 0.0)  // top-right
    );

    let c = card_data.cards[instanceIndex];

    let on_mouse = c.tableau == 0;
    let locked = get_type(c) == TYPE_CARD_HIDDEN;

    let aabb = get_world_position_and_size(c, input_u.mouse_pos);
    let total_cards: u32 = card_data.total;
    let max_depth = max_depth(total_cards);
    let z = get_depth(c,total_cards);

    var origin = aabb.xy;
    var size = aabb.zw;

    var hovering = z == hovering_buffer.hovering_max_z;

    if (c.tableau == 0u) {
        hovering = true;
    }

    if (hovering) { //Zoom
        const HOVER_ZOOM = 0.2;

        let offset = HOVER_ZOOM*size/2.0;
        origin -= offset/2.0;
        size += offset;
    }
    if (hovering) { //Outline
        let outline_width = select(HOVER_OUTLINE_WIDTH,10.0,c.tableau == 0u);
        let uv_offset = vec2<f32>(outline_width, outline_width) / size;

        origin -= vec2<f32>(outline_width,outline_width);
        size += vec2<f32>(outline_width*2.0,outline_width*2.0);

        uv = array<vec2<f32>, 4>(
            vec2<f32>(0.0, 1.0) + vec2<f32>(-uv_offset.x,uv_offset.y), // top-left
            vec2<f32>(1.0, 1.0) + uv_offset, // top-right
            vec2<f32>(0.0, 0.0) - uv_offset, // bottom-left
            vec2<f32>(1.0, 0.0) + vec2<f32>(uv_offset.x,-uv_offset.y), // bottom-right
        );
    }
    if (hovering && !on_mouse) { //Magnet effect
        let center = origin + size * 0.5;

        let mouse = input_u.mouse_pos;

        // convert to screen space direction
        let dir = mouse - center;

        // tune strength (smaller = subtler)
        let strength = select(0.04,0.02,locked);

        let offset = dir * strength;

        origin += offset;
    }
    // //Snap to avoid inconsistent pixels
    // let center = origin + size * 0.5;
    // let snap = CARD_SPRITE_SIZE / 4.0;
    // size = floor(size / snap + 0.5) * snap;
    // origin = center - size * 0.5;

    let pos_px = array<vec2<f32>, 4>(
        origin,
        origin + vec2<f32>(size.x, 0.0),
        origin + vec2<f32>(0.0, size.y),
        origin + size,
    );



    let t = clamp(1.0 - f32(z) / f32(max_depth), 0.0, 1.0);
    var depth = 1-0 - (0.05 + t * 0.90);


    var out: VSOut;
    out.position = vec4<f32>(to_ndc(pos_px[vertexIndex]), depth, 1.0);
    out.uv = uv[vertexIndex];
    out.size = vec2<u32>(size);
    out.instanceIndex = instanceIndex;
    out.hovering = select(0u,1u,hovering);
    return out;
}