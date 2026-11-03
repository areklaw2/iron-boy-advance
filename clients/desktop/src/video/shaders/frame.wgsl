struct Uniforms {
    scale: vec2<f32>,
    offset: vec2<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var t_frame: texture_2d<f32>;
@group(0) @binding(2) var s_frame: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// 4-vertex quad drawn as a triangle strip, sized to the letterbox area.
// vid maps to the four corners:
//   0: top-left, 1: top-right, 2: bottom-left, 3: bottom-right
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    let x = f32(vid & 1u);
    let y = f32((vid >> 1u) & 1u);

    let uv = vec2<f32>(x, y);
    let ndc = vec2<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0);
    let pos = ndc * u.scale + u.offset;

    var o: VsOut;
    o.pos = vec4<f32>(pos, 0.0, 1.0);
    o.uv = uv;
    return o;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(t_frame, s_frame, in.uv);
}
