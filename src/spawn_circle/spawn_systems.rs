use std::time::Duration;

use bevy::prelude::*;

use crate::{Eyeball, assets::GltfAssets};

trait AppSpawnExt {
    fn register_spawn_system<M>(
        &mut self,
        id: String,
        system: impl IntoSystem<In<Transform>, (), M> + 'static,
    ) -> &mut App;
}

impl AppSpawnExt for App {
    fn register_spawn_system<M>(
        &mut self,
        id: String,
        system: impl IntoSystem<In<Transform>, (), M> + 'static,
    ) -> &mut App {
        let system_id =
            self.world_mut().register_system(system);
        self.world_mut()
            .resource_mut::<super::SpawnSystems>()
            .0
            .insert(id, system_id);
        self
    }
}

pub struct SpawnSystemsPlugin;
impl Plugin for SpawnSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.register_spawn_system(
            "eye".to_string(),
            one_shot_spawn_eye,
        );

        app.add_systems(
            Update,
            (scale_in, translate_up_in),
        );
    }
}

#[derive(Component)]
pub struct ScaleIn(Timer);

#[derive(Component)]
pub struct TranslateUpIn {
    timer: Timer,
    target: Vec3,
}

fn scale_in(
    mut query: Query<(
        Entity,
        &mut ScaleIn,
        &mut Transform,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut timer, mut transform) in &mut query {
        if timer.0.tick(time.delta()).just_finished() {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<ScaleIn>();
        } else {
            transform.scale = Vec3::splat(0.5)
                .lerp(Vec3::ONE, timer.0.fraction());
        }
    }
}
fn translate_up_in(
    mut query: Query<(
        Entity,
        &mut TranslateUpIn,
        &mut Transform,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut modifier, mut transform) in &mut query
    {
        if modifier.timer.tick(time.delta()).just_finished()
        {
            transform.translation = modifier.target;
            commands
                .entity(entity)
                .remove::<TranslateUpIn>();
        } else {
            let translation_curve = EasingCurve::new(
                modifier.target.with_y(-1.),
                modifier.target,
                EaseFunction::ElasticOut,
            );
            transform.translation = translation_curve
                .sample_clamped(modifier.timer.fraction())
        }
    }
}

fn one_shot_spawn_eye(
    transform: In<Transform>,
    mut commands: Commands,
    gltf: ResMut<GltfAssets>,
    gltfs: Res<Assets<Gltf>>,
) {
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
