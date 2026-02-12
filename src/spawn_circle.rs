use bevy::{
    color::palettes::tailwind::RED_400,
    light::{NotShadowCaster, NotShadowReceiver},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::assets::{GltfAssets, MyStates};

pub struct SpawnCirclePlugin;

impl Plugin for SpawnCirclePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MaterialPlugin::<
            ExtendedMaterial<
                StandardMaterial,
                SpawnCircleExt,
            >,
        >::default(),
        MaterialPlugin::<
            ExtendedMaterial<
                StandardMaterial,
                SpawnColumnExt,
            >,
        >::default()))
            .add_systems(
                Update,
                (scale_base, spawn_cylinder, spawn_circle_spawn).run_if(in_state(MyStates::Next)),
            )
            .add_observer(
                |added: On<Add, CylinderMaterial>,
                 std_materials: Res<
                    Assets<StandardMaterial>,
                >,
                 mut materials: ResMut<
                    Assets<
                        ExtendedMaterial<
                            StandardMaterial,
                            SpawnColumnExt,
                        >,
                    >,
                >,
                mut commands: Commands,
                query: Query<&MeshMaterial3d<StandardMaterial>>,
                time: Res<Time>,
                | {
                    let mat = std_materials.get(&query.get(added.entity).unwrap().0).unwrap();
                    commands.entity(added.entity).remove::<MeshMaterial3d<StandardMaterial>>().insert(
                         MeshMaterial3d(materials.add(
                                ExtendedMaterial {
                                    base: mat.clone(),
                                    extension: SpawnColumnExt {
                                        spawn_time: time.elapsed_secs(),
                                        spawn_color: RED_400.into(),
                                        ..default()
                                    },
                                },
                            )),
                    );
                },
            );
    }
    fn finish(&self, app: &mut App) {
        let handle = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Plane3d::new(Vec3::Y, Vec2::ONE));
        app.world_mut().insert_resource(
            SpawnCircleMeshPlane { default: handle },
        );
    }
}

#[derive(Resource)]
struct SpawnCircleMeshPlane {
    default: Handle<Mesh>,
}

#[derive(Component)]
pub struct SpawnCircleBase;

#[derive(Component)]
struct CylinderTimer(Timer);

#[derive(Component)]
struct SpawnCircleSpawnTimer(Timer);

#[derive(Component)]
#[require(CylinderTimer(Timer::from_seconds(
    1.,
    TimerMode::Once
)))]
#[require(SpawnCircleSpawnTimer(Timer::from_seconds(
    1.5,
    TimerMode::Once
)))]
pub struct SpawnCircle;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
struct CylinderMaterial;

#[derive(
    Asset, AsBindGroup, Reflect, Debug, Clone, Default,
)]
pub struct SpawnCircleExt {
    // We need to ensure that the bindings of the base
    // material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots
    // 0-99 for the base material.
    #[uniform(100)]
    pub(crate) spawn_time: f32,
    // Web examples WebGL2 support: structs must be 16 byte
    // aligned.
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_8b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_12b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_16b: u32,
    #[uniform(100)]
    pub(crate) spawn_color: LinearRgba,
}

impl MaterialExtension for SpawnCircleExt {
    fn fragment_shader() -> ShaderRef {
        "shaders/spawn_circle.wgsl".into()
    }
    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Add)
    }
}

#[derive(
    Asset, AsBindGroup, Reflect, Debug, Clone, Default,
)]
pub struct SpawnColumnExt {
    // We need to ensure that the bindings of the base
    // material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots
    // 0-99 for the base material.
    #[uniform(100)]
    pub(crate) spawn_time: f32,
    // Web examples WebGL2 support: structs must be 16 byte
    // aligned.
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_8b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_12b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_16b: u32,
    #[uniform(100)]
    pub(crate) spawn_color: LinearRgba,
}

impl MaterialExtension for SpawnColumnExt {
    fn fragment_shader() -> ShaderRef {
        "shaders/spawn_circle_column.wgsl".into()
    }
    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Add)
    }
    fn enable_shadows() -> bool {
        false
    }
}

fn spawn_cylinder(
    mut query: Query<(
        Entity,
        &mut CylinderTimer,
        &Transform,
    )>,
    time: Res<Time>,
    mut commands: Commands,
    gltf: ResMut<GltfAssets>,
    gltfs: Res<Assets<Gltf>>,
) {
    for (entity, mut timer, _transform) in &mut query {
        if timer.0.tick(time.delta()).just_finished() {
            commands
                .entity(entity)
                .remove::<CylinderTimer>();

            let child = commands
                .spawn((
                    Name::new("CylinderScene"),
                    SceneRoot(
                        gltfs
                            .get(&gltf.misc)
                            .unwrap()
                            .named_scenes["SpawnCircleColumn"].clone(),
                    ),
                    Transform::default()
                        .with_scale(Vec3::splat(0.4)),
                ))
                .id();
            commands.entity(entity).add_child(child);
        };
    }
}
fn scale_base(
    mut query: Query<
        (
            &mut Transform,
            &MeshMaterial3d<
                ExtendedMaterial<
                    StandardMaterial,
                    SpawnCircleExt,
                >,
            >,
        ),
        With<SpawnCircleBase>,
    >,
    materials: Res<
        Assets<
            ExtendedMaterial<
                StandardMaterial,
                SpawnCircleExt,
            >,
        >,
    >,
    time: Res<Time>,
) {
    for (mut transform, material_handle) in &mut query {
        let Some(material) = materials.get(material_handle)
        else {
            continue;
        };
        let time_since_start = time.elapsed_secs()
            - material.extension.spawn_time;

        transform.scale = transform
            .scale
            .lerp(Vec3::ONE, time_since_start / 2.);
    }
}

fn spawn_circle_spawn(
    mut query: Query<(
        Entity,
        &mut SpawnCircleSpawnTimer,
        &Transform,
    )>,
    time: Res<Time>,
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    gltf: Res<GltfAssets>,
) {
    for (entity, mut timer, transform) in &mut query {
        if timer.0.tick(time.delta()).just_finished() {
            commands
                .entity(entity)
                .remove::<SpawnCircleSpawnTimer>();

            let mut new_transform = *transform;
            new_transform.translation.y = 0.5;
            commands.spawn((
                Name::new("Eye"),
                SceneRoot(
                    gltfs
                        .get(&gltf.misc)
                        .unwrap()
                        .named_scenes["Eye"]
                        .clone(),
                ),
                new_transform, // transform.clone(), // .with_scale(Vec3::splat(0.4)),
            ));
            commands.entity(entity).despawn();
        };
    }
}

pub struct InitSpawnCircle {
    pub position: Vec2,
}

impl Command for InitSpawnCircle {
    fn apply(self, world: &mut World) {
        let spawn_circle_texture = world
            .resource_mut::<AssetServer>()
            .load("spawn-circle-001.png");
        let spawn_circle_mesh_plane = world
            .resource::<SpawnCircleMeshPlane>()
            .default
            .clone();
        let time = world.resource::<Time>().elapsed_secs();

        let material = world
            .resource_mut::<Assets<
                ExtendedMaterial<
                    StandardMaterial,
                    SpawnCircleExt,
                >,
            >>()
            .add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color_texture: Some(
                        spawn_circle_texture,
                    ),
                    ..Default::default()
                },
                extension: SpawnCircleExt {
                    spawn_time: time,
                    spawn_color: RED_400.into(),
                    ..default()
                },
            });
        world.spawn((
            Name::new("SpawnCircle"),
            SpawnCircle,
            Visibility::Visible,
            Transform::from_xyz(
                self.position.x,
                0.001,
                self.position.y,
            ),
            children![(
                SpawnCircleBase,
                NotShadowCaster,
                NotShadowReceiver,
                Mesh3d(spawn_circle_mesh_plane),
                MeshMaterial3d(material),
                Transform::default()
                    .with_scale(Vec3::new(0.8, 1., 0.8))
            )],
        ));
    }
}
