use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::Despawnable;

pub struct HammerSmackPlugin;

impl Plugin for HammerSmackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            HammerSmackMaterial,
        >::default())
            .add_systems(FixedUpdate, update_smack);
    }
}

#[derive(Component)]
#[require(NotShadowCaster, NotShadowReceiver)]
pub struct HammerSmack(Timer);

impl Default for HammerSmack {
    fn default() -> Self {
        Self(Timer::from_seconds(
            0.1,
            TimerMode::Once,
        ))
    }
}
// This struct defines the data that will be passed to your shader
#[derive(
    Asset, TypePath, AsBindGroup, Debug, Clone, Default,
)]
pub struct HammerSmackMaterial {
    #[uniform(0)]
    pub smack_percent: f32,
}

impl Material for HammerSmackMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/hammer_smack.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }
}

fn update_smack(
    mut query: Query<(
        Entity,
        &mut HammerSmack,
        &MeshMaterial3d<HammerSmackMaterial>,
    )>,
    mut materials: ResMut<Assets<HammerSmackMaterial>>,
    time: Res<Time>,
    mut commands: Commands,
    mut despawnable: ResMut<Despawnable>,
) {
    for (entity, mut smack_timer, mat) in &mut query {
        if smack_timer.0.tick(time.delta()).just_finished()
        {
            despawnable.0.insert(entity);
            // commands.entity(entity).despawn();
        } else {
            //update material
            let mut material =
                materials.get_mut(&mat.0).unwrap();
            material.smack_percent =
                smack_timer.0.fraction();
        }
    }
}
