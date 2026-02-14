use std::{
    f32::consts::{FRAC_PI_4, TAU},
    time::Duration,
};

use bevy::{
    color::palettes::tailwind::RED_400,
    math::sampling::UniformMeshSampler, prelude::*,
};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    MoveRandomly, PlayerCharacter,
    assets::GltfAssets,
    navmesh::ProcessedNavMesh,
    spawn_circle::spawn_systems::{
        AppSpawnExt, ScaleIn, TranslateUpIn,
    },
};

pub struct EyeBallPlugin;

impl Plugin for EyeBallPlugin {
    fn build(&self, app: &mut App) {
        app.register_spawn_system(
            "eye".to_string(),
            one_shot_spawn_eye,
        )
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
                spin_laser,
            ),
        )
        .add_observer(on_add_spin_laser);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
struct Eyeball;

#[derive(Component)]
#[require(SpinLaserTimer(Timer::from_seconds(
    2.,
    TimerMode::Once
)))]
struct SpinLaser;

#[derive(Component)]
struct SpinLaserTimer(Timer);

fn one_shot_spawn_eye(
    mut transform: In<Transform>,
    mut commands: Commands,
    gltf: ResMut<GltfAssets>,
    gltfs: Res<Assets<Gltf>>,
) {
    transform.translation.y = 0.5;

    commands.spawn(
        (
            Name::new("Eye"),
            Eyeball,
            SceneRoot(
                gltfs.get(&gltf.misc).unwrap().named_scenes
                    ["Eye"]
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
fn trigger_move_eyes_temp(
    query: Query<
        (Entity, &Transform),
        (
            With<Eyeball>,
            Without<MoveRandomly>,
            Without<ScaleIn>,
            Without<TranslateUpIn>,
            Without<SpinLaser>,
        ),
    >,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut commands: Commands,
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
    player: Single<
        &GlobalTransform,
        (With<PlayerCharacter>, Without<Eyeball>),
    >,
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
            if player
                .translation()
                .xz()
                .distance(global.translation().xz())
                < 3.
            {
                commands
                    .entity(entity)
                    .remove::<MoveRandomly>()
                    .insert(SpinLaser);
            } else {
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
                if navmesh.transformed_is_in_mesh(
                    sample.with_y(0.),
                ) {
                    commands.entity(entity).insert(
                        MoveRandomly {
                            from: transform
                                .translation
                                .xz(),
                            to: sample.xz(),
                        },
                    );
                }
            }
        } else {
            let direction = (move_randomly.to
                - global.translation().xz())
            .normalize();
            let movement = direction * time.delta_secs();
            transform.translation +=
                movement.extend(0.).xzy();

            let target_rotation = transform.looking_at(
                move_randomly
                    .to
                    .extend(global.translation().y)
                    .xzy(),
                Vec3::Y,
            );
            transform.rotation.smooth_nudge(
                &target_rotation.rotation,
                5.,
                time.delta_secs(),
            );
        }
    }
}

fn on_add_spin_laser(
    added: On<Add, SpinLaser>,
    player: Single<
        &Transform,
        (With<PlayerCharacter>, Without<Eyeball>),
    >,
    mut query: Query<&mut Transform, With<Eyeball>>,
) {
    let Ok(mut transform) = query.get_mut(added.entity)
    else {
        return;
    };
    transform.look_at(player.translation, Vec3::Y);
}

fn spin_laser(
    mut query: Query<
        (
            Entity,
            &mut SpinLaserTimer,
            &mut Transform,
        ),
        With<Eyeball>,
    >,
    mut commands: Commands,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (entity, mut laser_timer, mut transform) in
        &mut query
    {
        gizmos.ray(
            transform.translation,
            transform.forward() * 5.,
            RED_400,
        );

        if laser_timer.0.tick(time.delta()).just_finished()
        {
            commands
                .entity(entity)
                .remove::<(SpinLaser, SpinLaserTimer)>();
        } else {
            transform
                .rotate_local_z(TAU * time.delta_secs());
        }
    }
}
