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

@group(0) @binding(0) var<storage,read> card_data: CardArray;
@group(0) @binding(1) var<uniform> render_u: RenderUniforms;
@group(0) @binding(2) var<uniform> input_u: InputUniforms;
@group(0) @binding(3) var<storage,read> hovering_buffer: HoveringBuffer;
@group(0) @binding(4) var<storage,read> animation_data: array<Animation>;


@group(1) @binding(0) var textureSampler: sampler;
@group(1) @binding(1) var spritesheet: texture_2d<f32>;

struct VSOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) size: vec2<u32>,
    @location(2) @interpolate(flat) instanceIndex: u32,
    @location(3) @interpolate(flat) hovering: u32,
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

fn textureIsAtUvDistanceOfSprite(c: Card, b_x : u32, b_y : u32, f_x: u32, f_y: u32, localUV : vec2<f32>, distance: vec2<f32>) -> vec2<bool> {
    
    //Computes if at uv distance @distance there is an opaque cell on the sprite
    let right_uv = localUV + vec2<f32>(distance.x, 0.0);
    var right = get_sprite_color(c,b_x,b_y,f_x,f_y, right_uv).a != 0.0;
    if (right_uv.x < 0.0 || right_uv.y < 0.0 || right_uv.x > 1.0 || right_uv.y > 1.0) {
        right = false;
    }

    let left_uv = localUV + vec2<f32>(-distance.x, 0.0);
    var left = get_sprite_color(c,b_x,b_y,f_x,f_y, left_uv).a != 0.0;
    if (left_uv.x < 0.0 || left_uv.y < 0.0 || left_uv.x > 1.0 || left_uv.y > 1.0) {
        left = false;
    }

    let up_uv = localUV + vec2<f32>(0.0, distance.y);
    var up = get_sprite_color(c,b_x,b_y,f_x,f_y, up_uv).a != 0.0;
    if (up_uv.x < 0.0 || up_uv.y < 0.0 || up_uv.x > 1.0 || up_uv.y > 1.0) {
        up = false;
    }

    let down_uv = localUV + vec2<f32>(0.0, -distance.y);
    var down = get_sprite_color(c,b_x,b_y,f_x,f_y, down_uv).a != 0.0;
    if (down_uv.x < 0.0 || down_uv.y < 0.0 || down_uv.x > 1.0 || down_uv.y > 1.0) {
        down = false;
    }

    let up_right_uv = localUV + vec2<f32>( distance.x,  distance.y);
    var up_right = get_sprite_color(c,b_x,b_y,f_x,f_y, up_right_uv).a != 0.0;
    if (up_right_uv.x < 0.0 || up_right_uv.y < 0.0 || up_right_uv.x > 1.0 || up_right_uv.y > 1.0) {
        up_right = false;
    }

    let up_left_uv = localUV + vec2<f32>(-distance.x,  distance.y);
    var up_left = get_sprite_color(c,b_x,b_y,f_x,f_y, up_left_uv).a != 0.0;
    if (up_left_uv.x < 0.0 || up_left_uv.y < 0.0 || up_left_uv.x > 1.0 || up_left_uv.y > 1.0) {
        up_left = false;
    }

    let down_right_uv = localUV + vec2<f32>( distance.x, -distance.y);
    var down_right = get_sprite_color(c,b_x,b_y,f_x,f_y, down_right_uv).a != 0.0;
    if (down_right_uv.x < 0.0 || down_right_uv.y < 0.0 || down_right_uv.x > 1.0 || down_right_uv.y > 1.0) {
        down_right = false;
    }

    let down_left_uv = localUV + vec2<f32>(-distance.x, -distance.y);
    var down_left = get_sprite_color(c,b_x,b_y,f_x,f_y, down_left_uv).a != 0.0;
    if (down_left_uv.x < 0.0 || down_left_uv.y < 0.0 || down_left_uv.x > 1.0 || down_left_uv.y > 1.0) {
        down_left = false;
    }

    let border = up | right | down | left | down_left | up_left | down_right | up_right;
    return vec2<bool>(border,up_left); //Save upleft for shadowing for now, although i may want a bigger area
}

fn fake_light(base: vec3<f32>, uv: vec2<f32>, roughness: f32) -> vec3<f32> {
    let light_dir = normalize(vec3<f32>(-0.4, 0.6, 1.0));

    // fake normal from UV (gives soft gradient shading)
    let n = normalize(vec3<f32>(uv - 0.5, 0.8));

    let ndl = clamp(dot(n, light_dir), 0.0, 1.0);

    // matte = smooth falloff
    let diffuse = ndl;

    // specular = sharpness controlled by roughness
    let reflect_dir = reflect(-light_dir, n);
    let spec = pow(max(reflect_dir.z, 0.0), mix(2.0, 64.0, 1.0 - roughness));

    let color = base * (0.3 + 0.7 * diffuse) + vec3<f32>(spec);

    return color;
}


fn metallicize(color: vec3<f32>, uv: vec2<f32>, strength: f32) -> vec3<f32> {
    let lcolor = fake_light(color,uv,0.5);
    // 1. grayscale base (metal foundation)
    let gray = mix(vec3<f32>(luminance(color)),lcolor,0.4);

    // 2. contrast boost (cold metal)
    let contrast = 1.2;
    let exposure = 0.75; //Lower = darker
    let metal_base = (gray*exposure - 0.5) * contrast + 0.5;

    // 3. fake lighting gradient (directional metal)
    let light = 0.6 + 0.4 * (uv.y * 0.8 + uv.x * 0.2);

    // 4. specular streak (cheap shine)
    let streak = pow(1.0 - abs(uv.x - 0.5), 6.0);

    let spec = streak * 0.35;

    // 5. combine
    let fin = metal_base * light + spec * strength;

    return vec3<f32>(fin);
}

fn get_sprite_color(c: Card, b_x : u32, b_y: u32, f_x: u32, f_y: u32, localUV : vec2<f32>) -> vec4<f32> {

    if (has_flag(c,FLAG_HIDDEN)) {
        return getColorFromTexture(0u,1u, localUV);
    }

    var suit = get_suit(c);
    var value = get_value(c);
    var value_back = 0u;
    if (suit == SUIT_TABLEAU) {
        return getColorFromTexture(suit,value, localUV);
    }

    var color_back = getColorFromTexture(0u,value_back, localUV);
    var color_front = getColorFromTexture(suit,value, localUV);

    if (has_flag(c,FLAG_HYPER)) {
        color_back = color_over(getColorFromTexture(0u,5u, localUV),color_back);
    }

    if (has_flag(c,FLAG_GHOST)) {
        color_back *= 0.3;
    }

    var color = color_over(color_front,color_back);

    if (has_flag(c,FLAG_BLOODY)) {
        color = color_over(getColorFromTexture(0u,4u, localUV),color);
    }

    
    if (has_flag(c,FLAG_INVERTED)) {
        color = vec4<f32>(1.0 - color.rgb,color.a);
    }
    if (has_flag(c,FLAG_METAL)) {
        color = vec4<f32>(metallicize(color.rgb,localUV,0.5).rgb,color.a);
    }

    

    return color;
}


fn quantize(p: vec2<f32>, object_size: vec2<f32>,PIXEL_FACTOR: f32) -> vec2<f32> {
    var p_px = p * object_size;

    p_px = floor(p_px / PIXEL_FACTOR) * PIXEL_FACTOR + 0.5 * PIXEL_FACTOR;

    return p_px / object_size;
}


fn pixelate_uv(uv: vec2<f32>, factor: f32) -> vec2<f32> {
    return (floor(uv * factor) + 0.5) / factor;
}

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}
fn dirty_darken(c: vec3<f32>, amount: f32, dirt: f32) -> vec3<f32> {
    let dark = c * amount;
    let gray = vec3<f32>(luminance(c));
    return mix(dark, gray, dirt);
}



fn color_over(top: vec4<f32>, bottom: vec4<f32>) -> vec4<f32> {
    let out_rgb = top.rgb + bottom.rgb * (1.0 - top.a);
    let out_a = top.a + bottom.a * (1.0 - top.a);
    return vec4<f32>(out_rgb, out_a);
}


const CARD_SPRITE_SIZE = vec2<f32>(92.0,132.0);

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;

    let uv_pixel = 1.0 / vec2<f32>(in.size);


    let quv = quantize(uv,CARD_SPRITE_SIZE,4.0);
    // var quv = uv;
    // quv = pixelate_uv(uv,24.0);

    let c = card_data.cards[in.instanceIndex];

    var suit = get_suit(c);
    var value = get_value(c);

    var suit_frame = 0u;
    var value_frame = 10u;


    var suit_back = 0u;
    var value_back = 7u;

    

    // if (has_flag(c,FLAG_BONUS)) {
    //     value_back = 5u;
    // }

    // if (has_flag(c,FLAG_MBONUS)) {
    //     value_back = 6u;
    // }


    if (is_hidden(c)) {
        suit_back = 0u;
        value_back = 0u;
        suit = 0u;
        value = 10u;
    }
    if (suit == 1u) {
        suit_back = suit;
        value_back = value;
        suit = 0u;
        value = 10u;
    }






    var outline_check_distance = uv_pixel*select(HOVER_OUTLINE_WIDTH,10.0,c.tableau == 0u);

    var color = get_sprite_color(c,suit_back,value_back, suit, value, quv);


    // var color_back = getColorFromTexture(suit_back,value_back, quv);
    // var color_frame = getColorFromTexture(suit_frame,value_frame, quv);
    // var color = getColorFromTexture(suit,value, quv);

    // color = color_over(color,color_back);

    // if color_frame.a != 0 {
    //     if (color_frame.a == 1) {
    //         color = color_frame;
    //     }
    //     else {
    //         color = vec4<f32>(dirty_darken(color.rgb,0.8,0.2).rgb,color.a);
    //     }
    // }

    let check = textureIsAtUvDistanceOfSprite(c,suit_back,value_back,suit,value,uv,outline_check_distance);
    let is_border = check.x;
    let is_shadow = check.y;

    //Invert
    // color = vec4<f32>(vec3<f32>(1.0) - color.rgb,color.a);



    if (uv.x > 1.0 || uv.y > 1.0 || uv.x < 0.0 || uv.y < 0.0) {
        color = vec4<f32>(1.0,1.0,1.0,0.0);
    }
    if (color.a == 0.0 && bool(in.hovering) && is_border) {
        if (c.tableau != 0u) {
            color = vec4<f32>(HIGHLIGHT_COLOR,0.99);
        }
        else if is_shadow {
            color = vec4<f32>(0.0,0.0,0.0,0.4);
        }
    }


    if (color.a == 0.0) {
        discard;
    }

    color = vec4<f32>(color.rgb*0.9,color.a);

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

    
    let zoom = compute_zoom_to_fit(card_data.max_cards_on_one_tableau, render_u.resolution.y);
    
    let mouse = unzoom_point(input_u.mouse_pos, vec2<f32>(0.0), zoom);



    let c = card_data.cards[instanceIndex];

    let on_mouse = c.tableau == 0;
    let locked = is_hidden(c);

    let base = get_suit(c) == SUIT_TABLEAU;


    var aabb = get_world_position_and_size(c, mouse,render_u.resolution / zoom);

    if (c.animation_id != 0xFFFFFFFFu) {
        var c_prev = c;
        let animation = animation_data[c.animation_id];
        c_prev.tableau = animation.prev_tableau;
        c_prev.stack_idx = animation.prev_stack_idx;

        aabb = mix(aabb,get_world_position_and_size(c_prev, mouse,render_u.resolution / zoom),clamp(1.0 - animation.t,0.0,1.0));
    }

    let total_cards: u32 = card_data.total;
    let max_depth = max_depth(total_cards);
    let z = get_depth(c,total_cards);

    var origin = aabb.xy;
    var size = aabb.zw;

    var hovering = c.id == hovering_buffer.hovering_id;

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

        // convert to screen space direction
        let dir = mouse - center;

        // tune strength (smaller = subtler)
        let strength = select(0.04,0.02,locked);

        // let offset = dir * strength;
        let bias = vec2<f32>(1.0, -1.0); // positive x = right, negative y = up
        let offset = (dir + bias * length(dir)) * strength;

        origin += offset;
    }
    // //Snap to avoid inconsistent pixels
    // let center = origin + size * 0.5;
    // let snap = CARD_SPRITE_SIZE / 4.0;
    // size = floor(size / snap + 0.5) * snap;
    // origin = center - size * 0.5;

    // Apply zoom to all 4 positions
    let pos_px = array<vec2<f32>, 4>(
        zoom_point(origin,                          vec2<f32>(0.0), zoom),
        zoom_point(origin + vec2<f32>(size.x, 0.0), vec2<f32>(0.0), zoom),
        zoom_point(origin + vec2<f32>(0.0, size.y), vec2<f32>(0.0), zoom),
        zoom_point(origin + size,                   vec2<f32>(0.0), zoom),
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