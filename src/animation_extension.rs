use bevy::gltf::extensions::GltfExtensionHandlers;
use bevy::{
    animation::AnimationEvent,
    asset::LoadContext,
    ecs::entity::EntityHashSet,
    gltf::extensions::GltfExtensionHandler,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    scene::SceneInstanceReady,
};

pub struct GltfExtensionHandlerAnimationPlugin;

impl Plugin for GltfExtensionHandlerAnimationPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_family = "wasm")]
        bevy::tasks::block_on(async {
            app.world_mut()
                .resource_mut::<GltfExtensionHandlers>()
                .0
                .write()
                .await
                .push(Box::new(
                    GltfExtensionHandlerAnimation::default(
                    ),
                ))
        });
        #[cfg(not(target_family = "wasm"))]
        app.world_mut()
            .resource_mut::<GltfExtensionHandlers>()
            .0
            .write_blocking()
            .push(Box::new(
                GltfExtensionHandlerAnimation::default(),
            ));

        app.add_observer(play_animations_when_ready);
    }
}

fn play_animations_when_ready(
    ready: On<SceneInstanceReady>,
    children: Query<&Children>,
    mut players: Query<(&mut AnimationPlayer, &Animations)>,
) {
    info!(player_count=?players.iter().len(),"play_when_ready");
    for child in children.iter_descendants(ready.entity) {
        let Ok((mut player, animation_to_play)) =
            players.get_mut(child)
        else {
            continue;
        };

        // happens to be hardcoded to whatever the first one is?
        // fine for jam, fix later.
        player
            .play(animation_to_play.by_name("idle"))
            .repeat();
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Animations {
    // graph_handle: Handle<AnimationGraph>,
    pub indices: Vec<AnimationNodeIndex>,
    pub player_animations: Vec<&'static str>,
}

impl Animations {
    pub fn by_name(&self, id: &str) -> AnimationNodeIndex {
        let position = self
            .player_animations
            .iter()
            .position(|key| key == &id)
            .expect("a valid animation id");
        self.indices[position]
    }
}

#[derive(Default, Clone)]
struct GltfExtensionHandlerAnimation {
    animation_root_indices: HashSet<usize>,
    animation_root_entities: EntityHashSet,
    named_animations:
        HashMap<Box<str>, Handle<AnimationClip>>,
}

impl GltfExtensionHandler
    for GltfExtensionHandlerAnimation
{
    fn dyn_clone(&self) -> Box<dyn GltfExtensionHandler> {
        Box::new((*self).clone())
    }

    fn on_animations_collected(
        &mut self,
        _load_context: &mut LoadContext<'_>,
        _animations: &[Handle<AnimationClip>],
        named_animations: &HashMap<
            Box<str>,
            Handle<AnimationClip>,
        >,
        animation_roots: &HashSet<usize>,
    ) {
        self.animation_root_indices =
            animation_roots.clone();
        self.named_animations = named_animations.clone();
        info!(
            animation_roots = ?animation_roots,
            ?named_animations,
            "animations_collected"
        );
    }

    fn on_gltf_node(
        &mut self,
        _load_context: &mut LoadContext<'_>,
        gltf_node: &gltf::Node,
        entity: &mut EntityWorldMut,
    ) {
        if self
            .animation_root_indices
            .contains(&gltf_node.index())
        {
            self.animation_root_entities
                .insert(entity.id());
        }
    }

    fn on_scene_completed(
        &mut self,
        load_context: &mut LoadContext<'_>,
        scene: &gltf::Scene,
        world_root_id: Entity,
        world: &mut World,
    ) {
        if !scene
            .name()
            .is_some_and(|name| name == "Player")
        {
            return;
        }

        // handle player animations
        let player_animations = vec!["idle", "hammer-slam"];
        // Create an AnimationGraph from the desired clips
        let (graph, indices) = AnimationGraph::from_clips(
            player_animations.iter().map(|id| {
                self.named_animations[*id].clone()
            }),
        );
        // Store the animation graph as an asset with an arbitrary label
        // We only have one graph, so this label will be unique
        let graph_handle = load_context.add_labeled_asset(
            "PlayerAnimationGraph".to_string(),
            graph,
        );

        // Create a component that stores a reference to our animation
        let animations = Animations {
            indices,
            player_animations,
        };

        info!(roots=?self.animation_root_entities,?world_root_id);
        assert!(
            self.animation_root_entities.len() <= 1,
            "Assuming there is only one graph per scene as a shortcut"
        );

        let Some(root_entity) =
            self.animation_root_entities.iter().next()
        else {
            return;
        };

        world.entity_mut(*root_entity).insert((
            AnimationGraphHandle(graph_handle.clone()),
            animations,
        ));

        self.animation_root_entities.clear();
    }
}
