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
@group(0) @binding(2) var<storage,read_write> hovering_buffer: AtomicHoveringBuffer;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {

    hovering_buffer.hovering_id = 0xFFFFFFFF;
    atomicStore(&hovering_buffer.hovering_max_z,0xFFFFFFFF);
    workgroupBarrier();

    if (gid.x >= card_data.total) {
        return;
    }

    let c = card_data.cards[gid.x];

    if (c.tableau == 0u //Mouse pile
        // c.tableau == 0xFFFFFFF0 //Burn pile
        ) {return;}

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

    let z = get_depth(c,card_data.total);

    atomicMin(&hovering_buffer.hovering_max_z, z);

    workgroupBarrier();

    if (z == atomicLoad(&hovering_buffer.hovering_max_z)) {
        hovering_buffer.hovering_id = c.id;
    }
    
    // card_data.hovering = 2u;
}