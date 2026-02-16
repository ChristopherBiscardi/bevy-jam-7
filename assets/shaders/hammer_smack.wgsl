#import bevy_pbr::forward_io::VertexOutput

struct Material {
    smack_percent: f32,
    #ifdef SIXTEEN_BYTE_ALIGNMENT
        // Web examples WebGL2 support: structs must be 16 byte aligned.
        _webgl2_padding_8b: u32,
        _webgl2_padding_12b: u32,
        _webgl2_padding_16b: u32,
    #endif
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> ext: Material;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
// let distance = length(   mesh.uv);
// let angle = atan2(mesh.uv.x, mesh.uv.y);
// let result = vec2(angle / 3.14159 * 2, distance);
let uv = mesh.uv - vec2(0.5);
if abs(ext.smack_percent - abs(uv.x + uv.y)) < 0.2 {
    return vec4(10.,10.,10.,1.);
} else {
    // return vec4(uv.x, uv.y, 0.,1.);
    discard;
}
return vec4(0.,1.,1.,0.);
// return 
}
// float2 toPolar(float2 cartesian){
// 	float distance = length(cartesian);
// 	float angle = atan2(cartesian.y, cartesian.x);
// 	return float2(angle / UNITY_TWO_PI, distance);
// }