use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use bevy::{
    animation::AnimationEvent,
    color::palettes::tailwind::*,
    ecs::entity::EntityHashSet,
    gltf::GltfMaterialName,
    input::common_conditions::input_toggle_active,
    light::{VolumetricLight, light_consts::lux},
    math::{
        bounding::{BoundingCircle, IntersectsVolume},
        sampling::UniformMeshSampler,
    },
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
use rand::{
    Rng, prelude::Distribution, seq::IndexedRandom,
};

use crate::{
    animation_extension::GltfExtensionHandlerAnimationPlugin,
    assets::{GltfAssets, JamAssetsPlugin, MyStates},
    atmosphere::DefaultAtmosphere,
    crystals::CrystalPlugin,
    eyes::EyeBallPlugin,
    flock_sphere::FlockSpherePlugin,
    hammer_smack::{
        HammerSmack, HammerSmackMaterial, HammerSmackPlugin,
    },
    health::{Attack, Health, HealthPlugin},
    navmesh::{NavMeshPlugin, ProcessedNavMesh},
    player::{
        PlayerCharacter, PlayerPlugin, PlayerSpawnLocation,
        SpawnPlayer,
    },
    spawn_circle::{
        InitSpawnCircle, SpawnSystems,
        spawn_systems::{ScaleIn, TranslateUpIn},
    },
};

pub mod animation_extension;
pub mod assets;
pub mod atmosphere;
pub mod awareness;
pub mod controls;
pub mod crystals;
pub mod eyes;
pub mod flock_sphere;
pub mod hammer_smack;
pub mod health;
pub mod laser;
pub mod navmesh;
pub mod player;
pub mod spawn_circle;

#[cfg(feature = "free_camera")]
pub mod debug_free_cam;

#[derive(Resource, Default)]
pub struct Despawnable(EntityHashSet);

pub fn app() -> App {
    let mut app = App::new();

    app.init_resource::<Despawnable>()
        .insert_resource(ClearColor(SKY_800.into()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
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
            JamAssetsPlugin,
            NavMeshPlugin,
            FlockSpherePlugin,
            EyeBallPlugin,
            CrystalPlugin,
            HealthPlugin,
            PlayerPlugin,
            GltfExtensionHandlerAnimationPlugin,
            HammerSmackPlugin,
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
        .add_observer(pointer_click_spawn_eye)
        .add_systems(
            FixedUpdate,
            random_spawn_eyes
                .run_if(in_state(MyStates::Next)),
        )
        .add_systems(
            OnEnter(MyStates::Next),
            spawn_first_level,
        )
        .add_observer(on_scene_spawn_player)
        .add_systems(
            OnExit(MyStates::AssetLoading),
            on_exit_asset_loading,
        )
        .add_systems(
            Last,
            |mut despawnable: ResMut<Despawnable>,
             mut commands: Commands| {
                for entity in despawnable.0.drain() {
                    commands.entity(entity).try_despawn();
                }
            },
        )
        .add_observer(on_hammer_slam_finished)
        .add_observer(on_hammer_slam_hit);

    app
}

fn on_hammer_slam_finished(
    finished: On<HammerSlamFinished>,
) {
    info!("DONE");
    // play idle, remove spam prevention
}
fn on_hammer_slam_hit(
    finished: On<HammerSlamHit>,
    players: Query<
        (Entity, &GlobalTransform),
        With<PlayerCharacter>,
    >,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<HammerSmackMaterial>>,
    enemies: Query<
        (Entity, &Transform),
        (With<Health>, Without<PlayerCharacter>),
    >,
) {
    let Ok((player_entity, player)) = players.single()
    else {
        warn!("non-single player!");
        return;
    };
    let translation = player.forward().as_vec3().xz() * 2.;
    let mut new_transform = player.compute_transform();
    new_transform.translation.x -= translation.x;
    new_transform.translation.z -= translation.y;
    new_transform.translation.y = 0.1;

    commands.spawn((
        Name::new("hammer_hit_effect"),
        Mesh3d(meshes.add(
            Circle::new(1.5).mesh().build().rotated_by(
                Quat::from_rotation_x(-FRAC_PI_2),
            ),
        )),
        MeshMaterial3d(materials.add(
            HammerSmackMaterial { smack_percent: 0. },
        )),
        HammerSmack::default(),
        new_transform,
    ));
    let hit_circle = BoundingCircle {
        center: new_transform.translation.xz(),
        circle: Circle { radius: 1.5 },
    };
    for (entity, enemy) in enemies {
        let enemy = BoundingCircle {
            center: enemy.translation.xz(),
            circle: Circle::new(0.5),
        };

        if enemy.intersects(&hit_circle) {
            commands.trigger(Attack {
                attacker: player_entity,
                receiver: entity,
                strength: 20.,
            });
        }
    }
}
#[derive(AnimationEvent, Clone)]
pub struct HammerSlamFinished;

#[derive(AnimationEvent, Clone)]
pub struct HammerSlamHit;

fn on_exit_asset_loading(
    gltf: Res<GltfAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut clips: ResMut<Assets<AnimationClip>>,
) {
    let animations =
        &gltfs.get(&gltf.misc).unwrap().named_animations;
    let mut clip =
        clips.get_mut(&animations["hammer-slam"]).unwrap();

    // 24 fps?
    let duration = clip.duration();
    info!(?duration);
    clip.add_event(0.375, HammerSlamHit);
    clip.add_event(duration, HammerSlamFinished);
}
fn spawn_first_level(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    gltf: Res<GltfAssets>,
) {
    commands
        .spawn(
            SceneRoot(
                gltfs.get(&gltf.misc).unwrap().named_scenes
                    ["level-001"]
                    .clone(),
            ),
        )
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
                    if let Ok((name, _material)) =
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

fn on_scene_spawn_player(
    ready: On<SceneInstanceReady>,
    query: Query<
        &GlobalTransform,
        With<PlayerSpawnLocation>,
    >,
    children: Query<&Children>,
    mut commands: Commands,
    transform_helper: TransformHelper,
) {
    let Some(entity) = children
        .iter_descendants(ready.entity)
        .find(|entity| query.get(*entity).is_ok())
    else {
        return;
    };
    let player_spawn = transform_helper
        .compute_global_transform(entity)
        .unwrap();
    commands.queue(SpawnPlayer {
        position: player_spawn.compute_transform(),
        remaining_health: None,
    });
}
fn pointer_click_spawn_eye(
    mut picked: On<Pointer<Click>>,
    mut commands: Commands,
    spawn_systems: Res<SpawnSystems>,
    player_spawn: Single<
        &GlobalTransform,
        With<PlayerSpawnLocation>,
    >,
) {
    picked.propagate(false);
    // if let Some(position) = picked.hit.position {
    //     // let id = spawn_systems.0.get("gem-rock").unwrap();

    //     // commands.queue(InitSpawnCircle {
    //     //     position: position.xz(),
    //     //     event: *id,
    //     //     spawn_color: SKY_400.into(),
    //     // });
    // } else {
    //     warn!("spawn attempt without a hit position");
    // }
    // commands.queue(SpawnPlayer {
    //     position: player_spawn.compute_transform(),
    //     remaining_health: None,
    // });
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

#[derive(Component)]
pub struct ActivePlayerCamera;

fn startup(mut commands: Commands) {
    commands.spawn((
        ActivePlayerCamera,
        Camera3d::default(),
        Transform::from_xyz(-10., 10., 10.)
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
    //     Transform::from_scale(Vec3::new(10.0,
    // 1.0, 10.0))
    // .with_translation(Vec3::Y * 0.5), ));
}

fn random_spawn_eyes(
    mut commands: Commands,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut timer: Local<TestSpawnTimer>,
    time: Res<Time>,
    spawn_systems: Res<SpawnSystems>,
    current_navmesh: Query<(&ProcessedNavMesh, &Mesh3d)>,
    meshes: Res<Assets<Mesh>>,
    navmeshes: Res<Assets<vleue_navigator::NavMesh>>,
) {
    let Ok((navmesh, mesh)) = current_navmesh.single()
    else {
        return;
    };

    let navmesh = navmeshes.get(&navmesh.0).expect("a valid ProcessedNavMesh should fetch a valid NavMesh");
    let mesh = meshes
        .get(&mesh.0)
        .expect("a valid Mesh3d should fetch a valid Mesh");

    let sampler = UniformMeshSampler::try_new(
        mesh.triangles().unwrap(),
    )
    .unwrap();

    if timer.0.tick(time.delta()).just_finished() {
        let sample = rng.sample(&sampler);
        // TODO: loop until finding a valid position in the navmesh.
        // but for now we're using the mesh to sample so it *should* always
        // find a valid location
        if navmesh.transformed_is_in_mesh(sample.with_y(0.))
        {
            let enemy_to_spawn = ["eye", "flock-sphere"]
                .choose(&mut rng)
                .unwrap();
            let id = spawn_systems
                .0
                .get(*enemy_to_spawn)
                .expect("enemy {enemy_to_spawn} should have a valid spawn system registered");

            commands.queue(InitSpawnCircle {
                position: sample.xz(),
                event: *id,
                spawn_color: RED_400.into(),
            });
        }
    }
}

#[derive(Component)]
struct MoveRandomly {
    from: Vec2,
    to: Vec2,
}
