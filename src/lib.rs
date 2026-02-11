use std::f32::consts::FRAC_PI_2;

use bevy::{
    color::palettes::tailwind::*,
    gltf::GltfMaterialName,
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
use bevy_rand::{
    global::GlobalRng, plugin::EntropyPlugin,
    prelude::WyRand,
};
use bevy_seedling::prelude::*;
use bevy_shader_utils::ShaderUtilsPlugin;
use bevy_skein::SkeinPlugin;

use crate::{
    atmosphere::DefaultAtmosphere,
    spawn_circle::InitSpawnCircle,
};

pub mod atmosphere;
pub mod awareness;
pub mod controls;
pub mod laser;
pub mod spawn_circle;

#[cfg(feature = "free_camera")]
pub mod debug_free_cam;

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
            MeshPickingPlugin,
            ShaderUtilsPlugin,
            EntropyPlugin::<WyRand>::default(),
        ))
        .add_plugins((
            controls::ControlsPlugin,
            atmosphere::AtmospherePlugin,
            awareness::AwarenessPlugin,
            laser::LaserPlugin,
            #[cfg(feature = "free_camera")]
            debug_free_cam::DebugCamPlugin,
            spawn_circle::SpawnCirclePlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, |mut gizmos: Gizmos| {
            gizmos.circle(
                Isometry3d::new(
                    Vec3::new(0., 0.5, 0.),
                    Quat::from_rotation_x(FRAC_PI_2),
                ),
                2.,
                Color::WHITE,
            );
        })
        .add_observer(
            |mut picked: On<Pointer<Click>>,
             mut commands: Commands| {
                picked.propagate(false);
                if let Some(position) = picked.hit.position
                {
                    commands.queue(InitSpawnCircle {
                        position: position.xz(),
                    });
                }
            },
        )
        .add_systems(
            FixedUpdate,
            |mut commands: Commands,
             mut rng: Single<
                &mut WyRand,
                With<GlobalRng>,
            >,
             mut timer: Local<TestSpawnTimer>,
             time: Res<Time>| {
                if timer
                    .0
                    .tick(time.delta())
                    .just_finished()
                {
                    let plane = Rectangle::from_size(
                        Vec2::new(20., 20.),
                    );

                    commands.queue(InitSpawnCircle {
                        position: plane
                            .sample_interior(&mut rng),
                    });
                }
            },
        )
        .add_systems(
            FixedUpdate,
            |mut transforms: Query<
                &mut Transform,
                With<Eyeball>,
            >,
             time: Res<Time>| {
                // for mut transform in &mut
                // transforms {
                //     transform.translation.x =
                //         time.elapsed_secs().
                // sin() * 2.;
                //     transform.translation.z =
                //         time.elapsed_secs().
                // cos() * 2. + 2.;

                //     transform.rotation =
                //         Quat::from_rotation_y(
                //             time.elapsed_secs()
                // - FRAC_PI_2,
                //         );
                // }
            },
        );

    app
}

#[derive(Component)]
struct TestSpawnTimer(Timer);
impl Default for TestSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            5.,
            TimerMode::Repeating,
        ))
    }
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
            // lux::RAW_SUNLIGHT is recommended for use with
            // this feature, since other values
            // approximate sunlight *post-scattering* in
            // various conditions. RAW_SUNLIGHT
            // in comparison is the illuminance of the
            // sun unfiltered by the atmosphere, so it is
            // the proper input for sunlight to
            // be filtered by the atmosphere.
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
    //     Transform::from_scale(Vec3::new(10.0, 1.0,
    // 10.0))         .with_translation(Vec3::Y *
    // 0.5), ));
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
