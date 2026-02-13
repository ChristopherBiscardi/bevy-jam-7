use bevy::prelude::*;
use vleue_navigator::VleueNavigatorPlugin;

pub struct NavMeshPlugin;

impl Plugin for NavMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VleueNavigatorPlugin)
            .add_observer(on_add_navmesh);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[type_path = "api"]
struct NavMesh;

#[derive(Component)]
pub struct ProcessedNavMesh(
    pub Handle<vleue_navigator::NavMesh>,
);

fn on_add_navmesh(
    added: On<Add, NavMesh>,
    query: Query<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
    mut navmeshes: ResMut<Assets<vleue_navigator::NavMesh>>,
    mut commands: Commands,
) {
    let mesh = meshes.get(query.get(added.entity).expect(
        "NavMesh should be on an Entity with a Mesh3d",
    )).expect("A Mesh3d component should point to a valid Mesh");

    let Some(navmesh) =
        vleue_navigator::NavMesh::from_bevy_mesh(mesh)
    else {
        error!(
            "failed to create vleue_navigator::NavMesh::from_bevy_mesh"
        );
        return;
    };

    let handle = navmeshes.add(navmesh);
    commands
        .entity(added.entity)
        .insert(ProcessedNavMesh(handle));
}
