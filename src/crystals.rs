use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
use noiz::prelude::*;

use crate::{
    assets::GltfAssets,
    spawn_circle::spawn_systems::{
        AppSpawnExt, ScaleIn, TranslateUpIn,
    },
};

pub struct CrystalPlugin;

impl Plugin for CrystalPlugin {
    fn build(&self, app: &mut App) {
        app.register_spawn_system(
            "gem-rock".to_string(),
            one_shot_spawn_gem_rock,
        )
        .add_plugins((
            MaterialPlugin::<
                ExtendedMaterial<
                    StandardMaterial,
                    CrystalExt,
                >,
            >::default(),
            MaterialPlugin::<
                ExtendedMaterial<
                    StandardMaterial,
                    EnergyExt,
                >,
            >::default(),
        ))
        .add_observer(on_add_crystal_material)
        .add_systems(FixedUpdate, rotate_material);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
struct CrystalMaterial;

fn on_add_crystal_material(
    added: On<Add, CrystalMaterial>,
    std_materials: Res<Assets<StandardMaterial>>,
    mut materials: ResMut<
        Assets<
            ExtendedMaterial<StandardMaterial, CrystalExt>,
        >,
    >,
    mut commands: Commands,
    query: Query<&MeshMaterial3d<StandardMaterial>>,
    time: Res<Time>,
) {
    let mat = std_materials
        .get(&query.get(added.entity).unwrap().0)
        .unwrap();
    commands
        .entity(added.entity)
        .remove::<MeshMaterial3d<StandardMaterial>>()
        .insert(MeshMaterial3d(materials.add(
            ExtendedMaterial {
                base: mat.clone(),
                extension: CrystalExt {
                    spawn_time: time.elapsed_secs(),
                    ..default()
                },
            },
        )));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
struct EnergyMaterial;

#[derive(
    Asset, AsBindGroup, Reflect, Debug, Clone, Default,
)]
pub struct CrystalExt {
    // We need to ensure that the bindings of the base
    // material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots
    // 0-99 for the base material.
    #[uniform(100)]
    pub(crate) spawn_time: f32,
    // Web examples WebGL2 support: structs must be 16 byte
    // aligned.
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_8b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_12b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_16b: u32,
    #[uniform(100)]
    pub(crate) spawn_color: LinearRgba,
}

impl MaterialExtension for CrystalExt {
    fn fragment_shader() -> ShaderRef {
        "shaders/crystal.wgsl".into()
    }
    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Add)
    }
}

#[derive(
    Asset, AsBindGroup, Reflect, Debug, Clone, Default,
)]
pub struct EnergyExt {
    // We need to ensure that the bindings of the base
    // material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots
    // 0-99 for the base material.
    #[uniform(100)]
    pub(crate) spawn_time: f32,
    // Web examples WebGL2 support: structs must be 16 byte
    // aligned.
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_8b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_12b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_16b: u32,
    #[uniform(100)]
    pub(crate) spawn_color: LinearRgba,
}

impl MaterialExtension for EnergyExt {
    fn fragment_shader() -> ShaderRef {
        "shaders/energy.wgsl".into()
    }
    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Add)
    }
}

#[derive(Component)]
pub struct CrystalPylon;

fn one_shot_spawn_gem_rock(
    transform: In<Transform>,
    mut commands: Commands,
    gltf: ResMut<GltfAssets>,
    gltfs: Res<Assets<Gltf>>,
) {
    commands.spawn(
        (
            Name::new("CrystalPylon"),
            CrystalPylon,
            SceneRoot(
                gltfs.get(&gltf.misc).unwrap().named_scenes
                    ["gem-rock"]
                    .clone(),
            ),
            *transform,
            ScaleIn(Timer::new(
                Duration::from_millis(100),
                TimerMode::Once,
            )),
            TranslateUpIn {
                timer: Timer::new(
                    Duration::from_millis(250),
                    TimerMode::Once,
                ),
                target: transform.translation,
            },
        ),
    );
}

fn rotate_material(
    mut query: Query<
        (&mut Transform, &GlobalTransform),
        With<CrystalMaterial>,
    >,
    time: Res<Time>,
) {
    for (mut transform, global) in &mut query {
        let noise = Noise::<
            MixCellGradients<
                OrthoGrid,
                Smoothstep,
                QuickGradients,
            >,
        >::default();
        let value: f32 =
            noise.sample(global.translation().xz());

        transform.rotation = Quat::from_rotation_y(
            FRAC_PI_2
                * (time.elapsed_secs() + value * 100.),
        );
    }
}
