use std::f32::consts::{FRAC_PI_4, PI};

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<PlayerControls>() // All contexts should be registered.
            .add_observer(apply_movement)
            .add_observer(on_add_controls);
    }
}

#[derive(Component)]
struct PlayerControls;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ControlledByPlayer;

fn apply_movement(
    movement: On<Fire<Movement>>,
    mut transforms: Query<
        &mut Transform,
        With<ControlledByPlayer>,
    >,
    mut gizmos: Gizmos,
) {
    let mut transform =
        transforms.get_mut(movement.context).unwrap();

    // rotation is -45deg so that it goes "north", as per the camera
    // TODO: is camera rotation in-scope?
    let rotation = Quat::from_rotation_y(-FRAC_PI_4);

    let mut velocity = movement.value.extend(0.0).xzy();
    velocity.z = -velocity.z;

    let distance_to_move = rotation * velocity;
    // info!(?distance_to_move, angle=?distance_to_move.xz().normalize().to_angle());
    let t = *transform;
    gizmos.axes(t, 3.);
    // gizmos.line(
    //     transform.translation,
    //     (transform.translation + distance_to_move) * 200.,
    //     Color::WHITE,
    // );

    transform.translation += distance_to_move;
    info!(movement=?movement.value);
    transform.rotation = Quat::from_rotation_y(
        movement.value.to_angle() + FRAC_PI_4,
    );
    // transform.translation += Vec3::NEG_Z * 0.01;
}

fn on_add_controls(
    controlled: On<Add, ControlledByPlayer>,
    mut commands: Commands,
) {
    commands.entity(controlled.entity).insert((
        PlayerControls,
        actions!(PlayerControls[
            (
                Action::<Movement>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
                Scale::splat(0.10),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick(),
                )),
            ),]
        ),
    ));
}
