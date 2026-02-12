use bevy::{
    diagnostic::{
        DiagnosticsStore, FrameTimeDiagnosticsPlugin,
    },
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use iyes_progress::{
    Progress, ProgressPlugin, ProgressReturningSystem,
    ProgressTracker,
};

// Time in seconds to complete a custom
// long-running task. If assets are loaded
// earlier, the current state will not be changed
// until the 'fake long task' is completed (thanks
// to 'iyes_progress')
// const DURATION_LONG_TASK_IN_SECS: f64 = 4.0;
const DURATION_LONG_TASK_IN_SECS: f64 = 0.3;

pub struct JamAssetsPlugin;

impl Plugin for JamAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MyStates>()
            .add_plugins((
                ProgressPlugin::<MyStates>::new()
                    .with_state_transition(
                        MyStates::AssetLoading,
                        MyStates::Next,
                    ),
                FrameTimeDiagnosticsPlugin::default(),
            ))
            .add_loading_state(
                LoadingState::new(MyStates::AssetLoading)
                    .load_collection::<GltfAssets>(),
            )
            .add_systems(
                Update,
                (
                    track_fake_long_task
                        .track_progress::<MyStates>(),
                    print_progress,
                )
                    .chain()
                    .run_if(in_state(
                        MyStates::AssetLoading,
                    ))
                    .after(
                        LoadingStateSet(
                            MyStates::AssetLoading,
                        ),
                    ),
            );
    }
}

// #[derive(AssetCollection, Resource)]
// struct AudioAssets {
//     #[asset(path = "audio/background.ogg")]
//     background: Handle<AudioSource>,
//     #[asset(path = "audio/plop.ogg")]
//     plop: Handle<AudioSource>,
// }

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
    #[asset(path = "001/misc.gltf")]
    pub misc: Handle<Gltf>,
}

fn track_fake_long_task(time: Res<Time>) -> Progress {
    if time.elapsed_secs_f64() > DURATION_LONG_TASK_IN_SECS
    {
        info!("Long fake task is completed");
        true.into()
    } else {
        false.into()
    }
}

fn print_progress(
    progress: Res<ProgressTracker<MyStates>>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_done: Local<u32>,
) {
    let progress = progress.get_global_progress();
    if progress.done > *last_done {
        *last_done = progress.done;
        info!(
            "[Frame {}] Changed progress: {:?}",
            diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                .map(|diagnostic| diagnostic.value().unwrap_or(0.))
                .unwrap_or(0.),
            progress
        );
    }
}

#[derive(
    Clone, Eq, PartialEq, Debug, Hash, Default, States,
)]
pub enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
