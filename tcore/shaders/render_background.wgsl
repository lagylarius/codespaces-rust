struct RenderUniforms {
  resolution: vec2<f32>
};
@group(0) @binding(0) var<uniform> render_u: RenderUniforms;

struct VSOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    let u = f * f * (3.0 - 2.0 * f);

    return mix(a, b, u.x)
         + (c - a) * u.y * (1.0 - u.x)
         + (d - b) * u.x * u.y;
}

const PIXEL_FACTOR = 4.0;
fn quantize(p: vec2<f32>) -> vec2<f32> {
    var p_px = p * render_u.resolution;

    p_px = floor(p_px / PIXEL_FACTOR) * PIXEL_FACTOR + 0.5 * PIXEL_FACTOR;

    return p_px / render_u.resolution;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    var uv = in.uv;


    var p = uv * 2.0 - 1.0;

    let aspect = render_u.resolution.x / render_u.resolution.y;


    uv = quantize(uv);
    p = quantize(p);

    uv.x *= aspect;
    p.x *= aspect;

    // -------------------------
    // 1. Base felt color
    // -------------------------
    var color = vec3<f32>(0.08, 0.35, 0.18);

    // -------------------------
    // 2. Fiber noise (subtle cloth texture)
    // -------------------------
    let n = noise(uv * 400.0);
    color += (n - 0.5) * 0.05;

    // -------------------------
    // 3. Radial lighting (table spotlight feel)
    // -------------------------


    let dist = length(p);
    let light = 1.0 - smoothstep(0.2, 1.2, dist);
    color *= mix(0.9, 1.2, light);

    // -------------------------
    // 4. Edge vignette (wood rim illusion)
    // -------------------------
    // let vignette = smoothstep(1.2, 0.3, dist);
    // color *= vignette;

    return vec4<f32>(color, 1.0);
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VSOut {

    var pos = array<vec2<f32>, 4>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>( 1.0, -1.0)
    );

    var uv = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0), // bottom-left
        vec2<f32>(0.0, 1.0), // top-left
        vec2<f32>(1.0, 0.0), // bottom-right
        vec2<f32>(1.0, 1.0)  // top-right
    );


    var out: VSOut;
    out.position = vec4<f32>(pos[i], 0.0, 1.0);
    out.uv = uv[i];
    return out;
}