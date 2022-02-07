#import bevy_pbr::mesh_struct
#import bevy_pbr::mesh_view_bind_group

[[group(2), binding(0)]]
var<uniform> mesh: Mesh;

struct CurveWidth {
    curve_width: f32;
};

[[group(1), binding(0)]]
var<uniform> curve_width: CurveWidth;

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] curve_normal: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    let rh = view.projection[0];
    let n = sqrt(dot(rh, rh)) * 5.0;
    let scale = curve_width.curve_width / n;

    let offset_position = vec4<f32>(
        vertex.position.x + vertex.curve_normal.x * scale,
        vertex.position.y + vertex.curve_normal.y * scale,
        vertex.position.z,
        1.0
    );

    let world_position = mesh.model * offset_position;

    var out: VertexOutput;
    out.clip_position = view.view_proj * world_position;

    return out;
}

[[stage(fragment)]]
fn fragment() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
