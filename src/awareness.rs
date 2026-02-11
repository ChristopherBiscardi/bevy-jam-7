use bevy::{
    color::palettes::tailwind::*,
    ecs::entity::EntityHashSet, prelude::*,
};

pub struct AwarenessPlugin;

/// How entities detect other entities
impl Plugin for AwarenessPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, detect_in_range)
            .add_systems(FixedUpdate, detect_out_of_range);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
#[require(DetectedEntities)]
struct TrackEntities {
    in_range: Circle,
    max_range: Circle,
}

#[derive(Component, Default)]
struct DetectedEntities(EntityHashSet);

/// An entity that can be detected by this module
/// if it is in range of another entity
#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
pub struct Detectable;

fn detect_in_range(
    _entities: Query<&GlobalTransform, With<Detectable>>,
    mut query: Query<(
        &TrackEntities,
        &GlobalTransform,
        &mut DetectedEntities,
    )>,
    mut gizmos: Gizmos,
) {
    for (tracking_info, transform, _detected) in
        &mut query
    {
        gizmos.circle(
            transform.translation(),
            tracking_info.in_range.radius,
            RED_400,
        );
    }
}
fn detect_out_of_range(
    _entities: Query<&GlobalTransform, With<Detectable>>,
    mut query: Query<(
        &TrackEntities,
        &GlobalTransform,
        &mut DetectedEntities,
    )>,
    mut gizmos: Gizmos,
) {
    for (tracking_info, transform, _detected) in
        &mut query
    {
        gizmos.circle(
            transform.translation(),
            tracking_info.max_range.radius,
            GREEN_400,
        );
    }
}
