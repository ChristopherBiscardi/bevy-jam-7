use bevy::{
    anti_alias::{smaa::Smaa, taa::TemporalAntiAliasing},
    camera::Exposure,
    light::{AtmosphereEnvironmentMapLight, VolumetricFog},
    pbr::{
        Atmosphere, AtmosphereSettings, ScatteringMedium,
        ScreenSpaceReflections,
    },
    post_process::bloom::Bloom,
    prelude::*,
};

pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalAmbientLight::NONE)
            .add_observer(on_add_default_atmosphere);
    }
}

#[derive(Component)]
pub struct DefaultAtmosphere;

fn on_add_default_atmosphere(
    added: On<Add, DefaultAtmosphere>,
    mut commands: Commands,
    mut scattering_mediums: ResMut<
        Assets<ScatteringMedium>,
    >,
) {
    commands.entity(added.entity).insert((
        Atmosphere::earthlike(
            scattering_mediums
                .add(ScatteringMedium::default()),
        ),
        // Can be adjusted to change the scene scale and rendering quality
        AtmosphereSettings::default(),
        // The directional light illuminance used in this scene
        // (the one recommended for use with this feature) is
        // quite bright, so raising the exposure compensation helps
        // bring the scene to a nicer brightness range.
        Exposure { ev100: 13.0 },
        // Tonemapper chosen just because it looked good with the scene, any
        // tonemapper would be fine :)
        // Tonemapping::AcesFitted,
        // Bloom gives the sun a much more natural look.
        Bloom::NATURAL,
        // Enables the atmosphere to drive reflections and ambient lighting (IBL) for this view
        AtmosphereEnvironmentMapLight::default(),
        #[cfg(feature = "free_camera")]
        FreeCamera::default(),
        VolumetricFog {
            ambient_intensity: 0.0,
            ..default()
        },
        Msaa::Off,
        // TemporalAntiAliasing::default(),
        Smaa::default(),
        ScreenSpaceReflections::default(),
    ));
}
