use std::f32::consts::FRAC_PI_2;

use bevy::{
    color::palettes::tailwind::*,
    gltf::GltfMaterialName,
    input::common_conditions::input_toggle_active,
    light::{VolumetricLight, light_consts::lux},
    math::sampling::UniformMeshSampler,
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
use rand::{Rng, prelude::Distribution};

use crate::{
    assets::{GltfAssets, JamAssetsPlugin, MyStates},
    atmosphere::DefaultAtmosphere,
    navmesh::{NavMeshPlugin, ProcessedNavMesh},
    spawn_circle::{
        InitSpawnCircle, SpawnSystems,
        spawn_systems::{ScaleIn, TranslateUpIn},
    },
};

pub mod assets;
pub mod atmosphere;
pub mod awareness;
pub mod controls;
pub mod laser;
pub mod navmesh;
pub mod spawn_circle;

#[cfg(feature = "free_camera")]
pub mod debug_free_cam;

pub fn app() -> App {
    let mut app = App::new();

    app.insert_resource(ClearColor(SKY_800.into()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // resizable: (),
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
        ))
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (
                trigger_move_eyes_temp.run_if(
                    any_match_filter::<(
                        With<Eyeball>,
                        Without<MoveRandomly>,
                        Without<ScaleIn>,
                        Without<TranslateUpIn>,
                    )>,
                ),
                move_eyes_temp,
            ),
        )
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
        );

    app
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
                    ["Scene"]
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

fn pointer_click_spawn_eye(
    mut picked: On<Pointer<Click>>,
    mut commands: Commands,
    spawn_systems: Res<SpawnSystems>,
) {
    picked.propagate(false);
    if let Some(position) = picked.hit.position {
        let id = spawn_systems.0.get("eye").unwrap();

        for i in 0..100 {
            commands.queue(InitSpawnCircle {
                position: position.xz(),
                event: *id,
            });
        }
    } else {
        warn!("spawn attempt without a hit position");
    }
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

fn startup(mut commands: Commands) {
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
) {
    if timer.0.tick(time.delta()).just_finished() {
        let plane =
            Rectangle::from_size(Vec2::new(20., 20.));

        let id = spawn_systems.0.get("eye").unwrap();

        commands.queue(InitSpawnCircle {
            position: plane.sample_interior(&mut rng),
            event: *id,
        });
    }
}

#[derive(Component)]
struct MoveRandomly {
    from: Vec2,
    to: Vec2,
}

fn trigger_move_eyes_temp(
    query: Query<
        (Entity, &Transform),
        (
            With<Eyeball>,
            Without<MoveRandomly>,
            Without<ScaleIn>,
            Without<TranslateUpIn>,
        ),
    >,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut commands: Commands,
    time: Res<Time>,
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

    for (entity, transform) in &query {
        let sample = rng.sample(&sampler);
        // TODO: loop until finding a valid position in the navmesh.
        // but for now we're using the mesh to sample so it *should* always
        // find a valid location
        if navmesh.transformed_is_in_mesh(sample.with_y(0.))
        {
            commands.entity(entity).insert(MoveRandomly {
                from: transform.translation.xz(),
                to: sample.xz(),
            });
        }
    }
}

fn move_eyes_temp(
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &GlobalTransform,
            &MoveRandomly,
        ),
        With<Eyeball>,
    >,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut commands: Commands,
    time: Res<Time>,
    // mut gizmos: Gizmos,
    current_navmesh: Query<(&ProcessedNavMesh, &Mesh3d)>,
    meshes: Res<Assets<Mesh>>,
    navmeshes: Res<Assets<vleue_navigator::NavMesh>>,
) {
    for (entity, mut transform, global, move_randomly) in
        &mut query
    {
        // gizmos.arrow(
        //     global.translation(),
        //     move_randomly
        //         .to
        //         .extend(global.translation().y)
        //         .xzy(),
        //     Color::WHITE,
        // );
        // gizmos.sphere(
        //     move_randomly.to.extend(0.).xzy(),
        //     0.5,
        //     GREEN_400,
        // );
        if global
            .translation()
            .xz()
            .distance(move_randomly.to)
            < 0.1
        {
            let Ok((navmesh, mesh)) =
                current_navmesh.single()
            else {
                return;
            };

            let navmesh = navmeshes.get(&navmesh.0).expect("a valid ProcessedNavMesh should fetch a valid NavMesh");
            let mesh = meshes.get(&mesh.0).expect(
                "a valid Mesh3d should fetch a valid Mesh",
            );

            let sampler = UniformMeshSampler::try_new(
                mesh.triangles().unwrap(),
            )
            .unwrap();

            let sample = rng.sample(&sampler);
            // TODO: loop until finding a valid position in the navmesh.
            // but for now we're using the mesh to sample so it *should* always
            // find a valid location
            if navmesh
                .transformed_is_in_mesh(sample.with_y(0.))
            {
                commands.entity(entity).insert(
                    MoveRandomly {
                        from: transform.translation.xz(),
                        to: sample.xz(),
                    },
                );
            }
        } else {
            let direction = (move_randomly.to
                - global.translation().xz())
            .normalize();
            let movement = direction * time.delta_secs();
            transform.translation +=
                movement.extend(0.).xzy();

            transform.look_at(
                move_randomly
                    .to
                    .extend(global.translation().y)
                    .xzy(),
                Vec3::Y,
            );
        }
    }
}
