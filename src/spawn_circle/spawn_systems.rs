use bevy::prelude::*;

pub trait AppSpawnExt {
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
        app.add_systems(
            Update,
            (scale_in, translate_up_in),
        );
    }
}

#[derive(Component)]
pub struct ScaleIn(pub Timer);

#[derive(Component)]
pub struct TranslateUpIn {
    pub timer: Timer,
    pub target: Vec3,
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
