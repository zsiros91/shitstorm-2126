mod arena;
mod assets;
mod camera;
mod collision;
mod combat;
mod debug;
mod enemy;
mod fx;
mod healthbar;
mod player;
mod states;
mod ui;
mod weapon;

use bevy::prelude::*;

use crate::states::{GameSet, GameState, Paused};

/// Resolve the asset directory. A plain `cargo run` puts the binary in `target/debug`,
/// which makes Bevy look for `target/debug/assets`. Prefer the project assets directory
/// that was baked in at compile time, and fall back to a co-located `assets` folder.
fn asset_path() -> String {
    if let Ok(root) = std::env::var("BEVY_ASSET_ROOT") {
        return format!("{root}/assets");
    }
    let manifest_assets = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    if std::path::Path::new(&manifest_assets).is_dir() {
        manifest_assets
    } else {
        "assets".to_string()
    }
}

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
                .set(AssetPlugin {
                    file_path: asset_path(),
                    ..default()
                })
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
            healthbar::HealthBarPlugin,
        ))
        .add_plugins((
            player::PlayerPlugin,
            weapon::WeaponPlugin,
            enemy::EnemyPlugin,
            combat::CombatPlugin,
            ui::UiPlugin,
            debug::DebugPlugin,
        ))
        .run();
}
