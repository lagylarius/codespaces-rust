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
@group(0) @binding(1) var<uniform> render_u: RenderUniforms;
@group(0) @binding(2) var<uniform> input_u: InputUniforms;
@group(0) @binding(3) var<storage,read_write> hovering_buffer: AtomicHoveringBuffer;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {

    hovering_buffer.hovering_id = 0xFFFFFFFF;
    atomicStore(&hovering_buffer.hovering_max_z,0xFFFFFFFF);
    


    let idx = gid.x;

    workgroupBarrier();

    if (idx >= card_data.total) {return;}

    let c = card_data.cards[idx];

    if (c.tableau == 0u) {return;} //Mouse pile
 
    let zoom = compute_zoom_to_fit(card_data.max_cards_on_one_tableau, render_u.resolution.y);
    let mouse = unzoom_point(input_u.mouse_pos, vec2<f32>(0.0), zoom);

    let aabb = get_world_position_and_size(c, mouse,render_u.resolution / zoom);
    let origin = aabb.xy;
    let size = aabb.zw;

    var hovering =
        mouse.x >= origin.x &&
        mouse.x <= origin.x + size.x &&
        mouse.y >= origin.y &&
        mouse.y <= origin.y + size.y;
    if (!hovering) {return;}

    let z = get_depth(c,card_data.total);

    //Save z
    atomicMin(&hovering_buffer.hovering_max_z, z);
}