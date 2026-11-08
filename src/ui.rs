use bevy::prelude::*;
use bevy::text::FontSmoothing;

use crate::assets::GameAssets;
use crate::combat::Health;
use crate::player::{Player, Repair};
use crate::states::{gameplay_active, GameSet, GameState, Paused};
use crate::waves::{Score, WaveManager, WavePhase};

#[derive(Component, Clone, Copy)]
enum HudText {
    Health,
    Repair,
    Wave,
    Score,
}

#[derive(Component)]
struct PauseOverlay;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), spawn_menu)
            .add_systems(
                Update,
                menu_input.run_if(in_state(GameState::Menu)),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                (spawn_hud, reset_paused),
            )
            .add_systems(
                Update,
                (
                    toggle_pause.run_if(in_state(GameState::Playing)),
                    sync_pause_overlay.run_if(in_state(GameState::Playing)),
                    update_hud
                        .in_set(GameSet::Ui)
                        .run_if(gameplay_active),
                ),
            )
            .add_systems(OnEnter(GameState::GameOver), (spawn_game_over, reset_paused))
            .add_systems(
                Update,
                game_over_input.run_if(in_state(GameState::GameOver)),
            );
    }
}

fn ui_font(assets: &GameAssets, size: f32) -> TextFont {
    TextFont::from_font_size(size)
        .with_font(assets.font.clone())
        .with_font_smoothing(FontSmoothing::None)
}

fn spawn_menu(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.05, 0.96)),
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SHITSTORM 2126"),
                ui_font(&assets, 78.0),
                TextColor(Color::srgb(1.0, 0.58, 0.16)),
            ));
            parent.spawn((
                Text::new("A tank. A storm of the dead. No backup."),
                ui_font(&assets, 24.0),
                TextColor(Color::srgb(0.82, 0.86, 0.82)),
            ));
            parent.spawn((
                Text::new("WASD / ARROWS  MOVE      MOUSE  AIM + FIRE"),
                ui_font(&assets, 20.0),
                TextColor(Color::srgb(0.65, 0.72, 0.66)),
            ));
            parent.spawn((
                Text::new("R  HOLD TO REPAIR (STATIONARY)      ESC  PAUSE"),
                ui_font(&assets, 20.0),
                TextColor(Color::srgb(0.65, 0.72, 0.66)),
            ));
            parent.spawn((
                Text::new("PRESS ENTER TO DEPLOY"),
                ui_font(&assets, 30.0),
                TextColor(Color::srgb(1.0, 0.86, 0.4)),
            ));
        });
}

fn menu_input(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

fn reset_paused(mut paused: ResMut<Paused>) {
    paused.0 = false;
}

fn spawn_hud(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            DespawnOnExit(GameState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(14.0),
                    left: Val::Px(16.0),
                    ..default()
                },
                Text::new("HULL 100/100"),
                ui_font(&assets, 22.0),
                TextColor(Color::srgb(0.92, 0.96, 0.9)),
                HudText::Health,
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(42.0),
                    left: Val::Px(16.0),
                    ..default()
                },
                Text::new("REPAIR 3/3"),
                ui_font(&assets, 22.0),
                TextColor(Color::srgb(0.45, 0.95, 0.55)),
                HudText::Repair,
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(14.0),
                    right: Val::Px(16.0),
                    ..default()
                },
                Text::new("WAVE 1"),
                ui_font(&assets, 22.0),
                TextColor(Color::srgb(1.0, 0.82, 0.4)),
                HudText::Wave,
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(42.0),
                    right: Val::Px(16.0),
                    ..default()
                },
                Text::new("SCORE 0"),
                ui_font(&assets, 22.0),
                TextColor(Color::srgb(0.92, 0.96, 0.9)),
                HudText::Score,
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(12.0),
                    left: Val::Px(16.0),
                    ..default()
                },
                Text::new("WASD MOVE   LMB FIRE   R REPAIR   ESC PAUSE"),
                ui_font(&assets, 16.0),
                TextColor(Color::srgba(0.8, 0.85, 0.8, 0.6)),
            ));
        });
}

fn update_hud(
    player: Query<(&Health, &Repair), With<Player>>,
    score: Res<Score>,
    waves: Option<Res<WaveManager>>,
    mut texts: Query<(&mut Text, &HudText)>,
) {
    let Ok((health, repair)) = player.single() else {
        return;
    };

    let wave_text = match waves.as_deref() {
        Some(waves) => match waves.phase {
            WavePhase::Countdown(timer) => {
                format!("WAVE {}   IN {:.0}", waves.wave, timer.ceil())
            }
            WavePhase::Spawning => format!("WAVE {}", waves.wave),
            WavePhase::Clearing => format!("WAVE {}   CLEAR", waves.wave),
        },
        None => "WAVE --".to_string(),
    };

    let repair_text = if repair.channel > 0.0 {
        "REPAIRING...".to_string()
    } else {
        format!("REPAIR {}/{}", repair.charges, repair.max_charges)
    };

    for (mut text, kind) in &mut texts {
        text.0 = match kind {
            HudText::Health => format!("HULL {:.0}/{:.0}", health.current, health.max),
            HudText::Repair => repair_text.clone(),
            HudText::Wave => wave_text.clone(),
            HudText::Score => format!("SCORE {}", score.value),
        };
    }
}

fn toggle_pause(keys: Res<ButtonInput<KeyCode>>, mut paused: ResMut<Paused>) {
    if keys.just_pressed(KeyCode::Escape) {
        paused.0 = !paused.0;
    }
}

fn sync_pause_overlay(
    mut commands: Commands,
    assets: Res<GameAssets>,
    paused: Res<Paused>,
    overlay: Query<Entity, With<PauseOverlay>>,
) {
    let existing = overlay.iter().next();

    if paused.0 && existing.is_none() {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.7)),
                PauseOverlay,
                DespawnOnExit(GameState::Playing),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("PAUSED"),
                    ui_font(&assets, 64.0),
                    TextColor(Color::srgb(1.0, 0.86, 0.4)),
                ));
            });
    } else if !paused.0 {
        if let Some(entity) = existing {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_game_over(
    mut commands: Commands,
    assets: Res<GameAssets>,
    score: Res<Score>,
    waves: Option<Res<WaveManager>>,
) {
    let wave = waves.map(|waves| waves.wave).unwrap_or(1);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.02, 0.02, 0.9)),
            DespawnOnExit(GameState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                ui_font(&assets, 78.0),
                TextColor(Color::srgb(0.95, 0.22, 0.15)),
            ));
            parent.spawn((
                Text::new(format!("SCORE {}      WAVE {}", score.value, wave)),
                ui_font(&assets, 30.0),
                TextColor(Color::srgb(0.92, 0.9, 0.86)),
            ));
            parent.spawn((
                Text::new("ENTER  FIGHT AGAIN      ESC  MENU"),
                ui_font(&assets, 24.0),
                TextColor(Color::srgb(1.0, 0.82, 0.4)),
            ));
        });
}

fn game_over_input(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    } else if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }
}
