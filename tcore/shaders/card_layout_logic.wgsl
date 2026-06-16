#include "GLOBALS.wgsl"

struct RenderUniforms {
  resolution: vec2<f32>
};

struct InputUniforms {
    mouse_pos: vec2<f32>,
    pad: f32,
    pad1: f32
};


@group(0) @binding(0) var<storage,read_write> card_data: CardArray;
@group(0) @binding(1) var<uniform> input_u: InputUniforms;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {

    atomicStore(&card_data.hovering_max_z,0u);
    workgroupBarrier();

    if (gid.x >= card_data.total) {
        return;
    }

    let c = card_data.cards[gid.x];

    let mouse = input_u.mouse_pos;

    let aabb = get_world_position_and_size(c, input_u.mouse_pos);

    let origin = aabb.xy;
    let size = aabb.zw;

    var hovering =
        mouse.x >= origin.x &&
        mouse.x <= origin.x + size.x &&
        mouse.y >= origin.y &&
        mouse.y <= origin.y + size.y;
        
    if (!hovering) {return;}

    let z = c.stack_idx + 1u;

    atomicMax(&card_data.hovering_max_z, z);
    
    card_data.hovering = 2u;
}