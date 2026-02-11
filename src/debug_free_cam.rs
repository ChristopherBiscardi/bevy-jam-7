use bevy::{
    camera_controller::free_camera::{
        FreeCamera, FreeCameraPlugin,
    },
    input::common_conditions::input_just_pressed,
    prelude::*,
};

// use crate::atmosphere::DefaultAtmosphere;

pub struct DebugCamPlugin;

impl Plugin for DebugCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FreeCameraPlugin)
            .add_systems(Startup, spawn_debug_cam)
            .add_systems(
                Update,
                swap_debug_cam.run_if(input_just_pressed(
                    KeyCode::Equal,
                )),
            );
    }
}

fn spawn_debug_cam(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            is_active: false,
            ..default()
        },
        Transform::from_xyz(-5., 3., 5.)
            .looking_at(Vec3::ZERO, Vec3::Y),
        // TODO: Why does having Atmosphere on two cameras crash the shader?
        // DefaultAtmosphere,
        FreeCamera {
            // sensitivity: todo!(),
            key_forward: KeyCode::ArrowUp,
            key_back: KeyCode::ArrowDown,
            key_left: KeyCode::ArrowLeft,
            key_right: KeyCode::ArrowRight,
            ..default()
        },
    ));
}

fn swap_debug_cam(
    mut query: Query<
        (&mut Camera, Has<FreeCamera>),
        With<Camera3d>,
    >,
    mut show_debug: Local<bool>,
) {
    *show_debug = !*show_debug;
    for (mut camera, is_debug) in &mut query {
        if is_debug {
            camera.is_active = *show_debug;
        } else {
            camera.is_active = !*show_debug;
        }
    }
}
