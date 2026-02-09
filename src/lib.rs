use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use bevy::{
    color::palettes::tailwind::*,
    gltf::{GltfMaterialName, GltfMeshName},
    input::common_conditions::input_toggle_active,
    light::{VolumetricLight, light_consts::lux},
    prelude::*,
    scene::SceneInstanceReady,
};
use bevy_blockout::{BlockoutPlugin, UseBlockoutMaterial};
use bevy_enhanced_input::prelude::*;
#[cfg(feature = "inspector_egui")]
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin, quick::WorldInspectorPlugin,
};
use bevy_seedling::prelude::*;
use bevy_skein::SkeinPlugin;

use crate::atmosphere::DefaultAtmosphere;

pub mod atmosphere;
pub mod controls;

pub fn app() -> App {
    let mut app = App::new();

    app.insert_resource(ClearColor(SKY_800.into()))
        .add_plugins(DefaultPlugins)
        .add_plugins((
            #[cfg(feature = "inspector_egui")]
            EguiPlugin::default(),
            #[cfg(feature = "inspector_egui")]
            WorldInspectorPlugin::default().run_if(
                input_toggle_active(true, KeyCode::Escape),
            ),
            BlockoutPlugin,
            EnhancedInputPlugin,
            SeedlingPlugin::default(),
            SkeinPlugin::default(),
        ))
        .add_plugins((
            controls::ControlsPlugin,
            atmosphere::AtmospherePlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            |mut transforms: Query<
                &mut Transform,
                With<Eyeball>,
            >,
             time: Res<Time>| {
                for mut transform in &mut transforms {
                    transform.translation.x =
                        time.elapsed_secs().sin() * 2.;
                    transform.translation.z =
                        time.elapsed_secs().cos() * 2. + 2.;

                    transform.rotation =
                        Quat::from_rotation_y(
                            time.elapsed_secs() - FRAC_PI_2,
                        );
                }
            },
        );

    app
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
struct Eyeball;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
#[require(controls::ControlledByPlayer)]
struct PlayerCharacter;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(-5., 2., 5.)
    //         .looking_at(Vec3::ZERO, Vec3::Y),
    //     DefaultAtmosphere,
    // ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-1., 1., 1.)
            .looking_at(Vec3::ZERO, Vec3::Y),
        DefaultAtmosphere,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.02,
            near: -50.,
            ..OrthographicProjection::default_3d()
        }),
    ));
    // Sun
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            // lux::RAW_SUNLIGHT is recommended for use with this feature, since
            // other values approximate sunlight *post-scattering* in various
            // conditions. RAW_SUNLIGHT in comparison is the illuminance of the
            // sun unfiltered by the atmosphere, so it is the proper input for
            // sunlight to be filtered by the atmosphere.
            illuminance: lux::FULL_DAYLIGHT,
            //             FULL_DAYLIGHT
            // DIRECT_SUNLIGHT
            // RAW_SUNLIGHT
            ..default()
        },
        Transform::from_xyz(1.0, 4., 1.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        VolumetricLight,
    ));
    // commands.spawn((
    //     FogVolume::default(),
    //     Transform::from_scale(Vec3::new(10.0, 1.0, 10.0))
    //         .with_translation(Vec3::Y * 0.5),
    // ));
    commands
        .spawn(SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(2)
                    .from_asset("001/misc.gltf"),
            ),
        ))
        .observe(
            |ready: On<SceneInstanceReady>,
             children: Query<&Children>,
             query: Query<(
                &GltfMaterialName,
                &MeshMaterial3d<StandardMaterial>,
            )>,
             mut commands: Commands| {
                for child in
                    children.iter_descendants(ready.entity)
                {
                    if let Ok((name, material)) =
                        query.get(child)
                    {
                        match name.0.as_str() {
                            "Floor" => {
                                commands
                                    .entity(child)
                                    .insert(
                                        UseBlockoutMaterial,
                                    );
                            }
                            name => {
                                info!(?name);
                            }
                        };
                    };
                }
            },
        );
}
