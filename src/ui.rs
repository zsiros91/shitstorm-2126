use bevy::prelude::*;
use bevy::sprite::{BorderRect, TextureSlicer};
use bevy::ui_widgets::{Activate, ActivateOnPress, Button as UiButton};

use crate::arena::Arena;
use crate::assets::GameAssets;
use crate::build::BuildMode;
use crate::combat::Health;
use crate::economy::Economy;
use crate::enemy::Enemy;
use crate::hq::Hq;
use crate::player::{Player, Repair};
use crate::power::PowerNode;
use crate::states::{gameplay_active, GameSet, GameState, Paused};
use crate::tower::{Tower, TowerKind};
use crate::waves::{Score, WaveManager, WavePhase};

#[derive(Component, Clone, Copy)]
enum HudText {
    Armor,
    Repair,
    Wave,
    Score,
    Minerals,
    Gas,
    Hq,
    BuildStatus,
    Alert,
}

#[derive(Component)]
struct PauseOverlay;

#[derive(Component)]
struct CommandButton {
    kind: TowerKind,
}

#[derive(Component)]
struct MinimapDot;

#[derive(Resource, Default)]
struct AlertState {
    text: String,
    timer: f32,
}

impl AlertState {
    fn push(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.timer = 3.2;
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AlertState>()
            .add_systems(OnEnter(GameState::Menu), spawn_menu)
            .add_systems(Update, menu_input.run_if(in_state(GameState::Menu)))
            .add_systems(
                OnEnter(GameState::Playing),
                (spawn_hud, reset_paused).chain(),
            )
            .add_systems(
                Update,
                (
                    toggle_pause.run_if(in_state(GameState::Playing)),
                    tick_alert,
                    sync_pause_overlay.run_if(in_state(GameState::Playing)),
                    update_hud
                        .in_set(GameSet::Ui)
                        .run_if(gameplay_active),
                    update_minimap
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

fn framed_image(image: Handle<Image>, border: f32) -> ImageNode {
    ImageNode {
        image,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(border),
            ..default()
        }),
        ..default()
    }
}

fn ui_font(assets: &GameAssets, size: f32, bold: bool) -> TextFont {
    TextFont::from_font_size(size).with_font(if bold {
        assets.font_bold.clone()
    } else {
        assets.font.clone()
    })
}

/// True when the mouse is over one of the fixed HUD panels. Gameplay input
/// systems use this to avoid firing or manifesting towers "through" the HUD.
pub fn cursor_over_hud(window: &Window) -> bool {
    let Some(cursor) = window.cursor_position() else {
        return false;
    };
    let size = window.size();
    cursor.y > size.y - 150.0
        || (cursor.x < 350.0 && cursor.y < 122.0)
        || (cursor.x > size.x - 330.0 && cursor.y < 96.0)
}

fn panel_node(left: Val, top: Val, width: Val, height: Val) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left,
        top,
        width,
        height,
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn spawn_menu(mut commands: Commands, assets: Res<GameAssets>) {
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
            BackgroundColor(Color::srgba(0.015, 0.025, 0.035, 0.97)),
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(760.0),
                    height: Val::Px(540.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(12.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                framed_image(assets.ui_panel.clone(), 10.0),
                BorderColor::all(Color::srgba(0.84, 0.69, 0.36, 0.7)),
                BackgroundColor(Color::srgba(0.02, 0.055, 0.08, 0.5)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("SHITSTORM 2126"),
                    ui_font(&assets, 66.0, true),
                    TextColor(Color::srgb(0.95, 0.72, 0.28)),
                    TextShadow {
                        offset: Vec2::new(3.0, 4.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                    },
                ));
                panel.spawn((
                    Text::new("TERRAN DOMINION  //  FORWARD OUTPOST GX-2126"),
                    ui_font(&assets, 19.0, false),
                    TextColor(Color::srgb(0.55, 0.82, 0.88)),
                ));
                panel.spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(2.0),
                        margin: UiRect::vertical(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.84, 0.69, 0.36, 0.5)),
                ));
                panel.spawn((
                    Text::new("A GOLIATH WALKER. A SWARM AT THE GATE.\nHOLD THE COMMAND CENTRE AND KEEP THE POWER CHAIN ALIVE."),
                    ui_font(&assets, 20.0, false),
                    TextColor(Color::srgb(0.82, 0.86, 0.82)),
                ));
                panel.spawn((
                    Text::new("WASD / ARROWS  MOVE      MOUSE  AIM + FIRE\nB OR 1-4  BUILD MENU      LMB  MANIFEST TOWER\nR  HOLD TO REPAIR          ESC  PAUSE"),
                    ui_font(&assets, 18.0, false),
                    TextColor(Color::srgb(0.62, 0.72, 0.72)),
                ));
                panel.spawn((
                    Text::new("PRESS ENTER TO DEPLOY"),
                    ui_font(&assets, 28.0, true),
                    TextColor(Color::srgb(1.0, 0.86, 0.4)),
                    TextShadow {
                        offset: Vec2::new(2.0, 3.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                    },
                ));
            });
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

fn text_node(left: Val, top: Val) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left,
        top,
        ..default()
    }
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
        .with_children(|root| {
            // -----------------------------------------------------------------
            // top left: command centre status
            // -----------------------------------------------------------------
            root.spawn((
                panel_node(Val::Px(18.0), Val::Px(16.0), Val::Px(330.0), Val::Px(100.0)),
                framed_image(assets.ui_panel.clone(), 10.0),
                BorderColor::all(Color::srgba(0.84, 0.69, 0.36, 0.45)),
                BackgroundColor(Color::srgba(0.015, 0.04, 0.06, 0.72)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    text_node(Val::Px(14.0), Val::Px(8.0)),
                    Text::new("COMMAND CENTRE"),
                    ui_font(&assets, 14.0, true),
                    TextColor(Color::srgb(0.9, 0.72, 0.34)),
                ));
                panel.spawn((
                    text_node(Val::Px(14.0), Val::Px(28.0)),
                    Text::new("INTEGRITY 2500/2500"),
                    ui_font(&assets, 21.0, true),
                    TextColor(Color::srgb(0.9, 0.95, 0.92)),
                    HudText::Hq,
                ));
                panel.spawn((
                    text_node(Val::Px(14.0), Val::Px(58.0)),
                    Text::new("WAVE 1"),
                    ui_font(&assets, 15.0, false),
                    TextColor(Color::srgb(0.62, 0.76, 0.78)),
                    HudText::Wave,
                ));
                panel.spawn((
                    text_node(Val::Px(14.0), Val::Px(78.0)),
                    Text::new("SCORE 0"),
                    ui_font(&assets, 14.0, false),
                    TextColor(Color::srgb(0.72, 0.74, 0.7)),
                    HudText::Score,
                ));
            });

            // -----------------------------------------------------------------
            // top right: resources
            // -----------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(18.0),
                    top: Val::Px(16.0),
                    width: Val::Px(300.0),
                    height: Val::Px(74.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                framed_image(assets.ui_panel.clone(), 10.0),
                BorderColor::all(Color::srgba(0.84, 0.69, 0.36, 0.45)),
                BackgroundColor(Color::srgba(0.015, 0.04, 0.06, 0.72)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    ImageNode::new(assets.icon_mineral.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(22.0),
                        top: Val::Px(22.0),
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        ..default()
                    },
                ));
                panel.spawn((
                    text_node(Val::Px(58.0), Val::Px(18.0)),
                    Text::new("160"),
                    ui_font(&assets, 27.0, true),
                    TextColor(Color::srgb(0.66, 0.9, 1.0)),
                    HudText::Minerals,
                ));
                panel.spawn((
                    ImageNode::new(assets.icon_gas.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(166.0),
                        top: Val::Px(22.0),
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        ..default()
                    },
                ));
                panel.spawn((
                    text_node(Val::Px(202.0), Val::Px(18.0)),
                    Text::new("35"),
                    ui_font(&assets, 27.0, true),
                    TextColor(Color::srgb(0.6, 1.0, 0.65)),
                    HudText::Gas,
                ));
                panel.spawn((
                    text_node(Val::Px(20.0), Val::Px(52.0)),
                    Text::new("MINERALS           VESPENE"),
                    ui_font(&assets, 10.0, false),
                    TextColor(Color::srgb(0.5, 0.62, 0.64)),
                ));
            });

            // -----------------------------------------------------------------
            // alert banner
            // -----------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    top: Val::Px(126.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(34.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|row| {
                row.spawn((
                    Text::new(""),
                    ui_font(&assets, 20.0, true),
                    TextColor(Color::srgb(1.0, 0.55, 0.25)),
                    TextShadow {
                        offset: Vec2::new(2.0, 2.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.9),
                    },
                    HudText::Alert,
                ));
            });

            // -----------------------------------------------------------------
            // bottom command bar
            // -----------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(144.0),
                    border: UiRect::top(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.012, 0.03, 0.045, 0.93)),
                BorderColor::all(Color::srgba(0.84, 0.69, 0.36, 0.55)),
            ))
            .with_children(|bar| {
                // portrait
                bar.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(18.0),
                        top: Val::Px(12.0),
                        width: Val::Px(100.0),
                        height: Val::Px(100.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    ImageNode::new(assets.portrait.clone()),
                    BorderColor::all(Color::srgba(0.84, 0.69, 0.36, 0.75)),
                ));

                bar.spawn((
                    text_node(Val::Px(132.0), Val::Px(16.0)),
                    Text::new("GX-9 GOLIATH"),
                    ui_font(&assets, 17.0, true),
                    TextColor(Color::srgb(0.9, 0.8, 0.55)),
                ));
                bar.spawn((
                    text_node(Val::Px(132.0), Val::Px(40.0)),
                    Text::new("ARMOR 150/150"),
                    ui_font(&assets, 19.0, true),
                    TextColor(Color::srgb(0.35, 0.95, 0.55)),
                    HudText::Armor,
                ));
                bar.spawn((
                    text_node(Val::Px(132.0), Val::Px(66.0)),
                    Text::new("REPAIR 3/3"),
                    ui_font(&assets, 15.0, false),
                    TextColor(Color::srgb(0.5, 0.85, 0.6)),
                    HudText::Repair,
                ));
                bar.spawn((
                    text_node(Val::Px(132.0), Val::Px(90.0)),
                    Text::new("WASD MOVE   LMB FIRE   R REPAIR"),
                    ui_font(&assets, 11.0, false),
                    TextColor(Color::srgba(0.52, 0.65, 0.66, 0.85)),
                ));

                // command card
                bar.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(360.0),
                        top: Val::Px(9.0),
                        width: Val::Px(500.0),
                        height: Val::Px(126.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                ))
                .with_children(|card| {
                    for kind in TowerKind::ALL {
                        let (minerals, gas) = kind.cost();
                        card.spawn((
                            Node {
                                width: Val::Px(119.0),
                                height: Val::Px(118.0),
                                border: UiRect::all(Val::Px(1.0)),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                row_gap: Val::Px(2.0),
                                ..default()
                            },
                            framed_image(assets.ui_button.clone(), 10.0),
                            BorderColor::all(Color::srgba(0.35, 0.55, 0.62, 0.7)),
                            BackgroundColor(Color::srgba(0.02, 0.04, 0.06, 0.7)),
                            CommandButton { kind },
                            UiButton,
                            ActivateOnPress,
                        ))
                        .observe(select_tower_from_click)
                        .with_children(|button| {
                            button.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(5.0),
                                    top: Val::Px(2.0),
                                    ..default()
                                },
                                Text::new(kind.hotkey()),
                                ui_font(&assets, 12.0, true),
                                TextColor(Color::srgb(0.95, 0.8, 0.4)),
                            ));
                            button.spawn((
                                ImageNode::new(kind.icon(&assets)),
                                Node {
                                    width: Val::Px(42.0),
                                    height: Val::Px(42.0),
                                    ..default()
                                },
                            ));
                            button.spawn((
                                Text::new(kind.short()),
                                ui_font(&assets, 12.0, true),
                                TextColor(Color::srgb(0.82, 0.86, 0.82)),
                            ));
                            let cost = if gas > 0 {
                                format!("{}m {}g", minerals, gas)
                            } else {
                                format!("{}m", minerals)
                            };
                            button.spawn((
                                Text::new(cost),
                                ui_font(&assets, 10.0, false),
                                TextColor(Color::srgb(0.58, 0.76, 0.76)),
                            ));
                        });
                    }
                });

                // minimap
                bar.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(18.0),
                        top: Val::Px(8.0),
                        width: Val::Px(200.0),
                        height: Val::Px(128.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    ImageNode::new(assets.ui_minimap.clone()),
                    BorderColor::all(Color::srgba(0.35, 0.65, 0.72, 0.8)),
                    BackgroundColor(Color::srgba(0.01, 0.03, 0.04, 0.8)),
                ))
                .with_children(|map| {
                    for _ in 0..64 {
                        map.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                width: Val::Px(2.0),
                                height: Val::Px(2.0),
                                display: Display::None,
                                border_radius: BorderRadius::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.9, 0.2, 0.2)),
                            MinimapDot,
                        ));
                    }
                });
            });

            // build status / help line
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    bottom: Val::Px(150.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(28.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|row| {
                row.spawn((
                    Text::new("B BUILD  1 GAUSS  2 MISSILE  3 ARC  4 RELAY"),
                    ui_font(&assets, 15.0, false),
                    TextColor(Color::srgb(0.6, 0.75, 0.76)),
                    TextShadow {
                        offset: Vec2::new(1.0, 2.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                    },
                    HudText::BuildStatus,
                ));
            });
        });
}

fn select_tower_from_click(
    activate: On<Activate>,
    buttons: Query<&CommandButton>,
    mut build: ResMut<BuildMode>,
) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };
    build.selected = button.kind;
    build.active = true;
    build.suppress_pause = false;
    build.status = "CLICK IN THE FIELD TO MANIFEST".to_string();
}

fn update_hud(
    player: Query<(&Health, &Repair), With<Player>>,
    hq: Query<&Health, With<Hq>>,
    economy: Res<Economy>,
    waves: Res<WaveManager>,
    score: Res<Score>,
    build: Res<BuildMode>,
    enemies: Query<&Enemy>,
    towers: Query<&PowerNode, With<Tower>>,
    mut alert: ResMut<AlertState>,
    mut texts: Query<(&mut Text, &HudText)>,
    mut buttons: Query<(&CommandButton, &mut ImageNode, &mut BorderColor)>,
    mut last_powered: Local<usize>,
    mut last_wave: Local<u32>,
) {
    let (Ok((health, repair)), Ok(hq_health)) = (player.single(), hq.single()) else {
        return;
    };

    let powered = towers.iter().filter(|node| node.powered).count();
    let total_towers = towers.iter().count();
    if powered < *last_powered {
        alert.push("POWER LINK SEVERED // TURRET OFFLINE");
    } else if total_towers > *last_powered && powered >= *last_powered {
        if *last_powered > 0 {
            alert.push("POWER GRID EXPANDED");
        }
    }
    *last_powered = powered;

    if *last_wave != waves.wave {
        if *last_wave != 0 {
            alert.push(format!("WAVE {} INBOUND", waves.wave));
        }
        *last_wave = waves.wave;
    }

    let wave_text = match waves.phase {
        WavePhase::Countdown(timer) => {
            format!("WAVE {}  IN {:.0}   //  {} HOSTILES", waves.wave, timer.ceil(), enemies.iter().count())
        }
        WavePhase::Spawning => format!(
            "WAVE {}  //  {} HOSTILES  //  {} TO SPAWN",
            waves.wave,
            enemies.iter().count(),
            waves.remaining_to_spawn
        ),
        WavePhase::Clearing => format!(
            "WAVE {}  //  {} HOSTILES REMAIN",
            waves.wave,
            enemies.iter().count()
        ),
    };

    let repair_text = if repair.channel > 0.0 {
        format!("REPAIRING... {:.0}%", (repair.channel / repair.channel_time * 100.0).min(100.0))
    } else {
        format!("REPAIR {}/{}   [HOLD R]", repair.charges, repair.max_charges)
    };

    let build_text = if build.active {
        let (minerals, gas) = build.selected.cost();
        let cost = if gas > 0 {
            format!("{}m {}g", minerals, gas)
        } else {
            format!("{}m", minerals)
        };
        format!(
            "{} // {} // {}  //  ESC CANCEL",
            build.selected.title(),
            cost,
            build.status
        )
    } else {
        "B BUILD  1 GAUSS  2 MISSILE  3 ARC  4 RELAY".to_string()
    };

    let alert_text = if alert.timer > 0.0 {
        alert.text.clone()
    } else {
        String::new()
    };

    for (mut text, kind) in &mut texts {
        text.0 = match kind {
            HudText::Armor => format!("ARMOR {:.0}/{:.0}", health.current, health.max),
            HudText::Repair => repair_text.clone(),
            HudText::Wave => wave_text.clone(),
            HudText::Score => format!("SCORE {}", score.value),
            HudText::Minerals => format!("{}", economy.minerals),
            HudText::Gas => format!("{}", economy.gas),
            HudText::Hq => format!("INTEGRITY {:.0}/{:.0}", hq_health.current, hq_health.max),
            HudText::BuildStatus => build_text.clone(),
            HudText::Alert => alert_text.clone(),
        };
    }

    for (button, mut image, mut border) in &mut buttons {
        let (minerals, gas) = button.kind.cost();
        let affordable = economy.can_afford(minerals, gas);
        image.color = if affordable {
            Color::WHITE
        } else {
            Color::srgba(0.45, 0.45, 0.45, 0.55)
        };
        *border = BorderColor::all(if build.active && build.selected == button.kind {
            Color::srgba(1.0, 0.82, 0.35, 0.95)
        } else if affordable {
            Color::srgba(0.3, 0.75, 0.82, 0.8)
        } else {
            Color::srgba(0.6, 0.22, 0.18, 0.75)
        });
    }
}

fn update_minimap(
    arena: Res<Arena>,
    mut dots: Query<(&mut Node, &mut BackgroundColor), With<MinimapDot>>,
    hq: Query<&Transform, With<Hq>>,
    player: Query<&Transform, (With<Player>, Without<Hq>)>,
    towers: Query<(&Transform, &PowerNode), (With<Tower>, Without<Player>, Without<Hq>)>,
    enemies: Query<&Transform, (With<Enemy>, Without<Tower>, Without<Player>, Without<Hq>)>,
) {
    let mut points: Vec<(Vec2, Color, f32)> = Vec::new();
    if let Ok(transform) = hq.single() {
        points.push((transform.translation.xy(), Color::srgb(1.0, 0.8, 0.3), 6.0));
    }
    if let Ok(transform) = player.single() {
        points.push((transform.translation.xy(), Color::srgb(0.35, 1.0, 0.55), 5.0));
    }
    for (transform, node) in &towers {
        let color = if node.powered {
            Color::srgb(0.35, 0.9, 1.0)
        } else {
            Color::srgb(1.0, 0.28, 0.2)
        };
        points.push((transform.translation.xy(), color, 3.5));
    }
    for transform in &enemies {
        if points.len() >= 64 {
            break;
        }
        points.push((transform.translation.xy(), Color::srgb(0.95, 0.22, 0.16), 2.5));
    }

    let width = 184.0;
    let height = 112.0;
    let half = arena.half;
    for (i, (mut node, mut color)) in dots.iter_mut().enumerate() {
        if i >= points.len() {
            node.display = Display::None;
            continue;
        }
        let (world, dot_color, size) = points[i];
        let x = 8.0 + (world.x + half.x) / (half.x * 2.0) * width;
        let y = 8.0 + (half.y - world.y) / (half.y * 2.0) * height;
        node.display = Display::Flex;
        node.left = Val::Px(x - size * 0.5);
        node.top = Val::Px(y - size * 0.5);
        node.width = Val::Px(size);
        node.height = Val::Px(size);
        color.0 = dot_color;
    }
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut build: ResMut<BuildMode>,
    mut paused: ResMut<Paused>,
) {
    if build.suppress_pause {
        build.suppress_pause = false;
        return;
    }
    if keys.just_pressed(KeyCode::Escape) && !build.active {
        paused.0 = !paused.0;
    }
}

fn tick_alert(time: Res<Time>, mut alert: ResMut<AlertState>) {
    if alert.timer > 0.0 {
        alert.timer -= time.delta_secs();
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
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.02, 0.03, 0.78)),
                PauseOverlay,
                DespawnOnExit(GameState::Playing),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("TACTICAL PAUSE"),
                    ui_font(&assets, 58.0, true),
                    TextColor(Color::srgb(1.0, 0.82, 0.4)),
                    TextShadow {
                        offset: Vec2::new(3.0, 3.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                    },
                ));
                parent.spawn((
                    Text::new("ESC  RESUME"),
                    ui_font(&assets, 18.0, false),
                    TextColor(Color::srgb(0.75, 0.82, 0.82)),
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
    economy: Res<Economy>,
    waves: Option<Res<WaveManager>>,
) {
    let wave = waves.map(|waves| waves.wave).unwrap_or(1);

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
            BackgroundColor(Color::srgba(0.02, 0.01, 0.01, 0.92)),
            DespawnOnExit(GameState::GameOver),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(680.0),
                    height: Val::Px(390.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(14.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                framed_image(assets.ui_panel.clone(), 10.0),
                BorderColor::all(Color::srgba(0.85, 0.25, 0.18, 0.8)),
                BackgroundColor(Color::srgba(0.08, 0.02, 0.02, 0.65)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("OUTPOST LOST"),
                    ui_font(&assets, 64.0, true),
                    TextColor(Color::srgb(0.95, 0.25, 0.16)),
                    TextShadow {
                        offset: Vec2::new(3.0, 4.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.9),
                    },
                ));
                panel.spawn((
                    Text::new(format!("WAVE {}      SCORE {}", wave, score.value)),
                    ui_font(&assets, 26.0, true),
                    TextColor(Color::srgb(0.92, 0.9, 0.86)),
                ));
                panel.spawn((
                    Text::new(format!(
                        "REMAINING MINERALS {}   //   VESPENE {}",
                        economy.minerals, economy.gas
                    )),
                    ui_font(&assets, 15.0, false),
                    TextColor(Color::srgb(0.62, 0.7, 0.68)),
                ));
                panel.spawn((
                    Text::new("ENTER  REDEPLOY      ESC  RETURN TO MENU"),
                    ui_font(&assets, 20.0, false),
                    TextColor(Color::srgb(1.0, 0.82, 0.4)),
                ));
            });
        });
}

fn game_over_input(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    } else if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }
}
