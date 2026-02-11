#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

#import bevy_pbr::mesh_view_bindings::globals;
#import bevy_shader_utils::perlin_noise_2d::perlin_noise_2d

struct Extension {
    spawn_time: f32,
    #ifdef SIXTEEN_BYTE_ALIGNMENT
        // Web examples WebGL2 support: structs must be 16 byte aligned.
        _webgl2_padding_8b: u32,
        _webgl2_padding_12b: u32,
        _webgl2_padding_16b: u32,
    #endif
    spawn_color: vec4f,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> ext: Extension;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {

    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // let elapsed_since_spawn = globals.time - ext.spawn_time;
    // var color = vec3(value) * smoothstep(0., 2., elapsed_since_spawn) * 500.;
    // if elapsed_since_spawn > 1. {
    //      color = color * ext.spawn_color.xyz;
    // }
    // pbr_input.material.base_color.r = color.r;
    // pbr_input.material.base_color.g = color.g;
    // pbr_input.material.base_color.b = color.b;
    // if value < 0.001 {
    //     discard;
    // }
    
    // pbr_input.material.base_color.a = 0.3;//in.uv.y;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

#endif

// let uv2 = uv_rotate * (in2.uv - vec2(0.5));
let noise = perlin_noise_2d(
    (in.uv + vec2(globals.time + ext.spawn_time))
     * vec2(10., 5.0));
// out.color.r = uv2.x;
// out.color.g = uv2.y;
out.color.r = 100.;
// out.color.g = 1.;
// out.color.b = 1.;
if noise < 1. - in.uv.y {
    discard;
}
// out.color.a = (1.0 - in.uv.y) * 0.;
    return out;
}