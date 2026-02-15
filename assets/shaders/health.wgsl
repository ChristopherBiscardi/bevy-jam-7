#import bevy_pbr::forward_io::VertexOutput

struct HealthData {
    health_color: vec4f,
    last_color: vec4f,
    total: f32,
    last: f32,
    current: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> health: HealthData;


@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let current = health.current / health.total;
    let last = health.last / health.total;
    if mesh.uv.x <= current {
        return health.health_color;
    } else if mesh.uv.x <= last {
        return health.last_color;
    } else {
        // return vec4(0.2,0.2,0.2,1.);
        discard;
    }
}