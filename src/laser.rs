use bevy::prelude::*;

pub struct LaserPlugin;

/// Laser process
///
/// 1. `RunningLaserProcess` is added to an
///    entity, with a target?
/// 1. Show small laser as tracking indicator
///     - tracking indicator smooth_nudges
///       increasingly accurate until achieving
///       "lock on"
///     - after lock-on is achieved, fire laser
/// 3. firing laser is a distance-restricted beam
///     - inner color with rainbow fresnel oklch
///       hues
/// 4. time
impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

/// Add this component to cause the entity to fire
/// a laser at the target
#[derive(Component)]
struct RunningLaserProcess;

fn on_add_running_laser_process(
    added: On<Add, RunningLaserProcess>,
) {
}
