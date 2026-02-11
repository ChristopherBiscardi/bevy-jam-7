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
) {
    let mut transform =
        transforms.get_mut(movement.context).unwrap();

    // rotation is -45deg so that it goes "north", as
    // per the camera TODO: is camera rotation
    // in-scope?
    let rotation = Quat::from_rotation_y(-FRAC_PI_4);

    let mut velocity = movement.value.extend(0.0).xzy();
    velocity.z = -velocity.z;

    let distance_to_move = rotation * velocity;

    transform.translation += distance_to_move;
    transform.rotation = Quat::from_rotation_y(
        movement.value.to_angle() + FRAC_PI_4,
    );
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
