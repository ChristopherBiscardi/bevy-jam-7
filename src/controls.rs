use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::animation_extension::Animations;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<PlayerControls>() // All contexts should be registered.
            .add_observer(apply_movement)
            .add_observer(on_add_controls)
            .add_observer(on_hammer_slam);
    }
}

#[derive(Component)]
struct PlayerControls;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

#[derive(InputAction)]
#[action_output(bool)]
struct HammerSlam;

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

    let distance_to_move = rotation * velocity * 0.03;

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
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick(),
                )),
            ),(
                Action::<HammerSlam>::new(),
               bindings!(
                    MouseButton::Left
                )
            )]
        ),
    ));
}

#[derive(Component)]
struct Slamming;

fn on_hammer_slam(
    slam: On<Start<HammerSlam>>,
    mut transforms: Query<
        &mut Transform,
        With<ControlledByPlayer>,
    >,
    // hack for animation-having player
    animations: Single<(&mut AnimationPlayer, &Animations)>,
) {
    info!(?slam);
    let (mut player, animations) = animations.into_inner();
    player.stop_all();

    player.play(animations.by_name("hammer-slam"));
    // let mut transform =
    //     transforms.get_mut(slam.context).unwrap();
    // info!(?transform);
}
