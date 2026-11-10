#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
use bevy::window::WindowResolution;

use crate::states::{GameSet, GameState, Paused};

/// Resolve the asset directory. A plain `cargo run` puts the binary in `target/debug`,
/// which makes Bevy look for `target/debug/assets`. Prefer the project assets directory
/// that was baked in at compile time (development), then an `assets` folder next to the
/// executable (packaged builds, any platform), and finally a relative `assets` folder.
fn asset_path() -> String {
    if let Ok(root) = std::env::var("BEVY_ASSET_ROOT") {
        return format!("{root}/assets");
    }
    let manifest_assets = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    if std::path::Path::new(&manifest_assets).is_dir() {
        return manifest_assets;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside_exe = dir.join("assets");
            if beside_exe.is_dir() {
                return beside_exe.to_string_lossy().into_owned();
            }
        }
    }
    "assets".to_string()
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
                        // Lock the window to 1:1 pixels so OS display scaling
                        // (125%/150% on Windows, fractional scaling on Linux)
                        // can never blur the pixel art.
                        resolution: WindowResolution::new(1280, 720)
                            .with_scale_factor_override(1.0),
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
