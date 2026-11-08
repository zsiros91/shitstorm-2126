mod arena;
mod assets;
mod build;
mod camera;
mod collision;
mod combat;
mod debug;
mod economy;
mod enemy;
mod fx;
mod healthbar;
mod hq;
mod player;
mod power;
mod states;
mod tower;
mod ui;
mod waves;

use bevy::prelude::*;

use crate::states::{GameSet, GameState, Paused};

fn main() {
    #[cfg(target_os = "windows")]
    if std::env::var_os("WGPU_BACKEND").is_none() {
        unsafe {
            std::env::set_var("WGPU_BACKEND", "dx12");
        }
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.012, 0.02, 0.03)))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "SHITSTORM 2126".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .init_resource::<Paused>()
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Ai,
                GameSet::Movement,
                GameSet::Combat,
                GameSet::Effects,
                GameSet::Ui,
            )
                .chain(),
        )
        .add_plugins((
            assets::GameAssetsPlugin,
            fx::FxPlugin,
            camera::CameraPlugin,
            arena::ArenaPlugin,
            hq::HqPlugin,
            power::PowerPlugin,
            healthbar::HealthBarPlugin,
            economy::EconomyPlugin,
        ))
        .add_plugins((
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            tower::TowerPlugin,
            build::BuildPlugin,
            combat::CombatPlugin,
            waves::WavesPlugin,
            ui::UiPlugin,
            debug::DebugPlugin,
        ))
        .run();
}
