//====================================================================
// Uniforms

struct Camera {
    projection: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

//====================================================================

struct VertexIn {
    // Vertex
    @location(0) vertex_pos: vec2<f32>,
    // Instance
    @location(1) pos: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) radius: f32,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos: vec2<f32>,
    @location(1) center: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) radius: f32,
}

//====================================================================

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    var vertex_pos = in.vertex_pos * in.radius + in.pos;

    out.clip_position = 
        camera.projection * 
        vec4<f32>(vertex_pos, 0., 1.);

    out.pos = vertex_pos;
    out.center = in.pos;
    out.color = in.color;
    out.radius = in.radius / 2.;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let distance = distance(in.pos, in.center);

    if distance > in.radius {
        discard;
    }

    return in.color;
}

//====================================================================

