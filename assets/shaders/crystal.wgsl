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
    // rotation for r channel
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    // globals.time;
    let noise = perlin_noise_2d((in.uv + in.world_position.xz / 10. + vec2(globals.time / 10.)) * vec2(10., 5.0)) + 1.;
    let a = pbr_input.material.base_color.a;
    pbr_input.material.base_color = pbr_input.material.base_color * noise * 100;
    pbr_input.material.base_color.a = a;

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

// out.color.r = uv2.x;
// out.color.g = uv2.y;
    return out;
}