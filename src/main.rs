mod arena;
mod assets;
mod camera;
mod collision;
mod combat;
mod debug;
mod enemy;
mod fx;
mod player;
mod states;
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
        .insert_resource(ClearColor(Color::srgb(0.07, 0.08, 0.09)))
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
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            combat::CombatPlugin,
            waves::WavesPlugin,
            ui::UiPlugin,
            debug::DebugPlugin,
        ))
        .run();
}
