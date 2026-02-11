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
    let uv_r_rotate = mat2x2(
        cos(globals.time), -sin(globals.time),
        sin(globals.time),  cos(globals.time)
    );
    var in_r = in;
    in_r.uv = uv_r_rotate * (in.uv - vec2(0.5)) + vec2(0.5);
    var pbr_input_r = pbr_input_from_standard_material(in_r, is_front);

    // we can optionally modify the input before lighting and alpha_discard is applied
    let r_value = clamp(
        pbr_input_r.material.base_color.r + pbr_input_r.material.base_color.g + pbr_input_r.material.base_color.b,
        0.,
        1.,
    );

    // rotation for g channel
    let uv_g_rotate = mat2x2(
        cos(-globals.time), -sin(-globals.time),
        sin(-globals.time),  cos(-globals.time)
    );
    var in_g = in;
    in_g.uv = uv_g_rotate * (in.uv - vec2(0.5)) + vec2(0.5);
    var pbr_input = pbr_input_from_standard_material(in_g, is_front);

    // we can optionally modify the input before lighting and alpha_discard is applied
    let value = clamp(
        pbr_input_r.material.base_color.r + pbr_input.material.base_color.g + pbr_input.material.base_color.b,
        0.,
        1.,
    );
    
    let elapsed_since_spawn = globals.time - ext.spawn_time;
    var color = vec3(value) * smoothstep(0., 2., elapsed_since_spawn) * 500.;
    if elapsed_since_spawn > 1. {
         color = color * ext.spawn_color.xyz;
    }
    pbr_input.material.base_color.r = color.r;
    pbr_input.material.base_color.g = color.g;
    pbr_input.material.base_color.b = color.b;
    if value < 0.001 {
        discard;
    }

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