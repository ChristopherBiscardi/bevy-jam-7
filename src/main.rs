use bevy::{
    color::palettes::tailwind::*,
    input::common_conditions::input_toggle_active,
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin, quick::WorldInspectorPlugin,
};
use bevy_seedling::prelude::*;
use bevy_skein::SkeinPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(SKY_800.into()))
        .add_plugins(DefaultPlugins)
        .add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(
                input_toggle_active(true, KeyCode::Escape),
            ),
            EnhancedInputPlugin,
            SeedlingPlugin::default(),
            SkeinPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5., 2., 5.)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
