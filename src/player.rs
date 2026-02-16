use std::time::Duration;

use bevy::prelude::*;

use crate::{
    assets::GltfAssets,
    controls::ControlledByPlayer,
    health::Health,
    spawn_circle::spawn_systems::{ScaleIn, TranslateUpIn},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
#[require(ControlledByPlayer)]
pub struct PlayerCharacter;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
pub struct PlayerSpawnLocation;

pub struct SpawnPlayer {
    pub position: Transform,
    pub remaining_health: Option<Health>,
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) -> () {
        let scene = world
            .resource::<Assets<Gltf>>()
            .get(&world.resource::<GltfAssets>().misc)
            .unwrap()
            .named_scenes["Player"]
            .clone();
        let transform = self.position;

        world.spawn((
            PlayerCharacter,
            self.position,
            SceneRoot(scene),
            self.remaining_health
                .unwrap_or(Health::new(50.)),
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
        ));
    }
}
