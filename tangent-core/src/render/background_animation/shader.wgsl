struct BackgroundUniform {
    top: vec4<f32>,
    bottom: vec4<f32>,
}
@group(0) @binding(0) var<uniform> background: BackgroundUniform;
struct Vertex { @location(0) position: vec2<f32>, }
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
@vertex
fn vs_main(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = (vertex.position + vec2<f32>(1.0)) * 0.5;
    return out;
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Static, low-contrast diagonal wash: no motion behind the notation.
    let t = smoothstep(0.0, 1.0, (1.0 - in.uv.y) * 0.65 + in.uv.x * 0.35);
    return mix(background.top, background.bottom, t);
}
