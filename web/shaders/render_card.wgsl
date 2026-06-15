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
    let uv = vec2<f32>(localUV.x,1 - localUV.y);
    
    
    let tileUvSize = TILE_SIZE / ATLAS_SIZE; // = (0.25, 0.25)

    let tileSpritePos = vec2<f32>(f32(sprite_pos_x),f32(sprite_pos_y));
    let tileUVOffset = tileSpritePos / (1/tileUvSize);;

    let tileUV = uv * tileUvSize + tileUVOffset;
    return textureSample(spritesheet, textureSampler, tileUV);
}

const PIXEL_FACTOR = 4.0;
fn quantize(p: vec2<f32>) -> vec2<f32> {
    var p_px = p * render_u.resolution;

    p_px = floor(p_px / PIXEL_FACTOR) * PIXEL_FACTOR + 0.5 * PIXEL_FACTOR;

    return p_px / render_u.resolution;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;

    let c = card_data.cards[in.instanceIndex];

    var suit = get_suit(c);
    var value = get_value_numeric(c) + 1u;



    if (get_type(c) == TYPE_CARD_HIDDEN) {
        suit = 0u;
        value = 0u;
    }

    var color = getColorFromTexture(suit,value, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)));

    let outside =
        uv.x > 1.0 || uv.y > 1.0 || uv.x < 0.0 || uv.y < 0.0;
    let in_corner =
        (uv.x < 0.0 && uv.y < 0.0) ||
        (uv.x < 0.0 && uv.y > 1.0) ||
        (uv.x > 1.0 && uv.y < 0.0) ||
        (uv.x > 1.0 && uv.y > 1.0);




    const HIGHLIGHT_COLOR = vec3<f32>(1.0, 0.867, 0.0);

    if (bool(in.hovering) && outside && !in_corner) {
        return vec4<f32>(HIGHLIGHT_COLOR,1.0);
    }

    if (!outside && bool(in.hovering) && color.a == 0.0) {
        return vec4<f32>(HIGHLIGHT_COLOR,1.0);
    }
    return color;
}


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

    // var origin = vec2<f32>(50.0,50.0);

    // if (c.tableau != 255u) {
    //     origin += vec2<f32>(120.0*f32(c.tableau >> 1u),0.0);
    // }
    // else {
    //     origin = input_u.mouse_pos;
    // }

    // origin += vec2<f32>(0.0,30.0*f32(c.stack_idx));


    // if ((c.tableau & 1u) == 1u) {
    //     let t = card_data.tableaus[c.tableau-1u];
    //     origin += vec2<f32>(0.0,30.0*f32(t.size));
    // }




    
    // var size = vec2<f32>(92.0,132.0);

    let mouse = input_u.mouse_pos;

    let aabb = get_world_position_and_size(c, input_u.mouse_pos);

    let z = c.stack_idx;
    const MAX_Z = 52u;

    var origin = aabb.xy;
    var size = aabb.zw;

    var hovering = instanceIndex == (atomicLoad(&card_data.hovering_max_z) - 1u);


    if (hovering) {
        const HOVER_WIDTH = 4.0;

        let uv_offset = vec2<f32>(HOVER_WIDTH, HOVER_WIDTH) / size;

        origin -= vec2<f32>(HOVER_WIDTH*1.5,HOVER_WIDTH*1.5);
        size += vec2<f32>(HOVER_WIDTH*3.0,HOVER_WIDTH*3.0);


        uv = array<vec2<f32>, 4>(
            vec2<f32>(0.0, 1.0) + vec2<f32>(-uv_offset.x,uv_offset.y), // top-left
            vec2<f32>(1.0, 1.0) + uv_offset, // top-right
            vec2<f32>(0.0, 0.0) - uv_offset, // bottom-left
            vec2<f32>(1.0, 0.0) + vec2<f32>(uv_offset.x,-uv_offset.y), // bottom-right
        );
    }

    let pos_px = array<vec2<f32>, 4>(
        origin,
        origin + vec2<f32>(size.x, 0.0),
        origin + vec2<f32>(0.0, size.y),
        origin + size,
    );

    // let depth = f32(MAX_Z - c.stack_idx) / f32(MAX_Z);

    // var depth = 1.0 - f32(z)/f32(MAX_Z);
    let t = clamp(1.0 - f32(z) / f32(MAX_Z), 0.0, 1.0);
    let depth = 0.05 + t * 0.90;

    // if (instanceIndex == 1u) {
    //     depth = 0.7;
    // }

    var out: VSOut;
    out.position = vec4<f32>(to_ndc(pos_px[vertexIndex]), depth, 1.0);
    out.uv = uv[vertexIndex];
    out.size = vec2<u32>(size);
    out.instanceIndex = instanceIndex;
    out.hovering = select(0u,1u,hovering);
    return out;
}