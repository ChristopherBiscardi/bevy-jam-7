use std::f32::consts::FRAC_PI_4;

use bevy::{
    color::palettes::tailwind::*,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::{ActivePlayerCamera, PlayerCharacter};

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            MaterialPlugin::<HealthBarMaterial>::default(),
        )
        .add_systems(FixedUpdate, lerp_health)
        .add_systems(
            Update,
            (align_healthbars, sync_health),
        )
        .add_observer(on_attack)
        .add_observer(on_add_health);
    }
}

#[derive(EntityEvent)]
pub struct Attack {
    pub attacker: Entity,
    #[event_target]
    pub receiver: Entity,
    pub strength: f32,
}

#[derive(Component)]
pub struct Health {
    /// the max total health an entity can have
    pub total: f32,
    /// the "last" health value, before damage was applied.
    /// used for showing damage chunks in a healthbar
    pub last: f32,
    /// the current health value
    pub current: f32,
}

impl Health {
    pub fn new(total: f32) -> Self {
        Self {
            total,
            last: 15.,   //total,
            current: 5., //total,
        }
    }
}

#[derive(Component)]
#[relationship(relationship_target = HealthBarDisplay)]
pub struct HealthBarOf {
    #[relationship]
    pub owner: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = HealthBarOf)]
pub struct HealthBarDisplay(Vec<Entity>);

fn on_attack(
    attack: On<Attack>,
    mut health_counts: Query<&mut Health>,
) {
    info!("process attack");
    let Ok(mut health) =
        health_counts.get_mut(attack.receiver)
    else {
        return;
    };

    health.current -= attack.strength;
}

fn lerp_health(
    mut health_counts: Query<&mut Health>,
    time: Res<Time>,
) {
    for mut health in &mut health_counts {
        let current = health.current;
        health.last.smooth_nudge(
            &current,
            1.,
            time.delta_secs(),
        );
    }
}

fn on_add_health(
    added: On<Add, Health>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<HealthBarMaterial>>,
    mut commands: Commands,
) {
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::from_size(
            Vec2::new(0.6, 0.1),
        ))),
        MeshMaterial3d(materials.add(HealthBarMaterial {
            health_color: RED_400.into(),
            last_color: YELLOW_400.into(),
            total: 0.,
            last: 0.,
            current: 0.,
        })),
        // Visibility::Hidden,
        Transform::default().with_rotation(
            Quat::from_rotation_y(-FRAC_PI_4),
        ),
        HealthBarOf {
            owner: added.entity,
        },
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

fn align_healthbars(
    mut query: Query<
        (&HealthBarOf, &mut Transform),
        Without<ActivePlayerCamera>,
    >,
    transforms: Query<&Transform, Without<HealthBarOf>>,
) {
    for (health_bar_of, mut transform) in &mut query {
        let owner_transform =
            transforms.get(health_bar_of.owner).expect("a health bar must have an owner with a transform");
        // This system is a shortcut that only syncs translation instead of
        // also syncing rotation, as would happen if this was a child transform.
        // In the future this whole system can be gpu instead of just the vertical offset
        transform.translation = owner_transform.translation;
    }
}

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct HealthBarMaterial {
    #[uniform(0)]
    health_color: LinearRgba,
    #[uniform(0)]
    last_color: LinearRgba,
    #[uniform(0)]
    total: f32,
    #[uniform(0)]
    last: f32,
    #[uniform(0)]
    current: f32,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for HealthBarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/health.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/health_vertex.wgsl".into()
    }
}

fn sync_health(
    query: Query<
        (&Health, &HealthBarDisplay),
        Changed<Health>,
    >,
    handles: Query<&MeshMaterial3d<HealthBarMaterial>>,
    mut materials: ResMut<Assets<HealthBarMaterial>>,
) {
    for (health, health_bars) in &query {
        for health_entity in &health_bars.0 {
            let mat = materials
                .get_mut(
                    &handles.get(*health_entity).unwrap().0,
                )
                .unwrap();
            mat.total = health.total;
            mat.current = health.current;
            mat.last = health.last;
        }
    }
}
