use bevy::prelude::*;
use bevy::sprite::{BorderRect, TextureSlicer};
use bevy::window::{CursorOptions, PrimaryWindow};

use crate::assets::GameAssets;
use crate::combat::{EnemyKilled, Health};
use crate::enemy::Enemy;
use crate::player::{Player, Stamina};
use crate::states::{GameSet, GameState, Paused, gameplay_active};
use crate::weapon::Weapon;

pub const ZONE_NAME: &str = "DUSTWIND JUNCTION";

#[derive(Component, Clone, Copy)]
enum HudText {
    Health,
    Stamina,
    Ammo,
    Hostiles,
    Kills,
    Hint,
}

#[derive(Component)]
struct PauseOverlay;

/// Vertical vitals bar; `kind` picks which stat drives the fill.
#[derive(Component, Clone, Copy)]
enum VitalBar {
    Health,
    Stamina,
}

/// One tick of the segmented magazine gauge.
#[derive(Component)]
struct MagTick(usize);

/// Thin reload progress strip under the magazine gauge.
#[derive(Component)]
struct ReloadBar;

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct DamageFlash {
    timer: f32,
}

#[derive(Resource, Default)]
struct RunStats {
    kills: u32,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunStats>()
            .add_systems(OnEnter(GameState::Menu), spawn_menu)
            .add_systems(Update, menu_input.run_if(in_state(GameState::Menu)))
            .add_systems(
                OnEnter(GameState::Playing),
                (spawn_hud, reset_paused, reset_stats).chain(),
            )
            .add_systems(
                Update,
                (
                    toggle_pause.run_if(in_state(GameState::Playing)),
                    sync_pause_overlay.run_if(in_state(GameState::Playing)),
                    update_hud.in_set(GameSet::Ui).run_if(gameplay_active),
                    move_crosshair.run_if(in_state(GameState::Playing)),
                    count_kills.in_set(GameSet::Ui).run_if(gameplay_active),
                ),
            )
            .add_systems(Update, sync_cursor)
            .add_systems(
                OnEnter(GameState::GameOver),
                (spawn_game_over, reset_paused),
            )
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

/// True when a viewport point sits on one of the fixed HUD panels
/// (zone block, vitals panel, bottom strip with hotbar / help / ammo).
pub fn hud_blocks(point: Vec2, size: Vec2) -> bool {
    point.y > size.y - 118.0
        || (point.x > size.x - 196.0 && point.y < 194.0)
        || (point.x < 330.0 && point.y < 86.0)
}

/// True when the mouse is over one of the fixed HUD panels. The weapon
/// system uses this to avoid firing "through" the interface.
pub fn cursor_over_hud(window: &Window) -> bool {
    window
        .cursor_position()
        .is_some_and(|cursor| hud_blocks(cursor, window.size()))
}

// ---------------------------------------------------------------------------
// menu
// ---------------------------------------------------------------------------

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
            BackgroundColor(Color::srgba(0.012, 0.016, 0.02, 0.98)),
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(720.0),
                    height: Val::Px(520.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(12.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                framed_image(assets.ui_panel.clone(), 5.0),
                BorderColor::all(Color::srgba(0.84, 0.69, 0.36, 0.7)),
                BackgroundColor(Color::srgba(0.02, 0.03, 0.03, 0.55)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    ImageNode {
                        image: assets.portrait.clone(),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.95),
                        ..default()
                    },
                    Node {
                        width: Val::Px(72.0),
                        height: Val::Px(72.0),
                        margin: UiRect::bottom(Val::Px(6.0)),
                        ..default()
                    },
                ));
                panel.spawn((
                    Text::new("SHITSTORM 2126"),
                    ui_font(&assets, 62.0, true),
                    TextColor(Color::srgb(0.95, 0.72, 0.28)),
                    TextShadow {
                        offset: Vec2::new(3.0, 4.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.85),
                    },
                ));
                panel.spawn((
                    Text::new(ZONE_NAME),
                    ui_font(&assets, 20.0, false),
                    TextColor(Color::srgb(0.58, 0.78, 0.82)),
                ));
                panel.spawn((
                    Node {
                        width: Val::Px(560.0),
                        height: Val::Px(2.0),
                        margin: UiRect::vertical(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.84, 0.69, 0.36, 0.45)),
                ));
                panel.spawn((
                    Text::new(
                        "THE GRID IS GONE. THE RAIDERS ARE NOT.\n\
                         PICK THROUGH THE RUINS AND KEEP BREATHING.",
                    ),
                    ui_font(&assets, 19.0, false),
                    TextColor(Color::srgb(0.8, 0.84, 0.8)),
                    TextLayout::justify(Justify::Center),
                ));
                panel.spawn((
                    Text::new(
                        "WASD / ARROWS  MOVE       SHIFT  SPRINT\n\
                         MOUSE  AIM + FIRE          R  RELOAD\n\
                         ESC  PAUSE",
                    ),
                    ui_font(&assets, 17.0, false),
                    TextColor(Color::srgb(0.6, 0.7, 0.7)),
                    TextLayout::justify(Justify::Center),
                ));
                panel.spawn((
                    Text::new("PRESS ENTER TO WALK OUT"),
                    ui_font(&assets, 26.0, true),
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

fn reset_stats(mut stats: ResMut<RunStats>) {
    stats.kills = 0;
}

// ---------------------------------------------------------------------------
// hud
// ---------------------------------------------------------------------------

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
            // -------------------------------------------------------------
            // zone name, top-left
            // -------------------------------------------------------------
            root.spawn((
                text_node(Val::Px(16.0), Val::Px(12.0)),
                Text::new(ZONE_NAME),
                ui_font(&assets, 19.0, true),
                TextColor(Color::srgb(0.92, 0.88, 0.78)),
                TextShadow {
                    offset: Vec2::new(2.0, 2.0),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.85),
                },
            ));
            root.spawn((
                text_node(Val::Px(17.0), Val::Px(38.0)),
                Text::new("SECTOR 7  //  LONG NIGHT"),
                ui_font(&assets, 11.0, false),
                TextColor(Color::srgba(0.62, 0.68, 0.66, 0.9)),
            ));
            root.spawn((
                text_node(Val::Px(17.0), Val::Px(56.0)),
                Text::new("KILLS 0"),
                ui_font(&assets, 12.0, true),
                TextColor(Color::srgb(0.9, 0.62, 0.3)),
                HudText::Kills,
            ));

            // -------------------------------------------------------------
            // vitals panel, top-right
            // -------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(14.0),
                    top: Val::Px(14.0),
                    width: Val::Px(176.0),
                    height: Val::Px(156.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                framed_image(assets.ui_panel.clone(), 5.0),
                BorderColor::all(Color::srgba(0.84, 0.69, 0.36, 0.55)),
                BackgroundColor(Color::srgba(0.015, 0.025, 0.03, 0.72)),
            ))
            .with_children(|panel| {
                spawn_vital_bar(panel, &assets, VitalBar::Health, 16.0);
                spawn_vital_bar(panel, &assets, VitalBar::Stamina, 100.0);
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    top: Val::Px(178.0),
                    ..default()
                },
                Text::new("HOSTILES 0"),
                ui_font(&assets, 13.0, true),
                TextColor(Color::srgb(0.95, 0.4, 0.3)),
                TextShadow {
                    offset: Vec2::new(1.0, 2.0),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.85),
                },
                HudText::Hostiles,
            ));

            // -------------------------------------------------------------
            // hotbar, bottom-left
            // -------------------------------------------------------------
            root.spawn((Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                bottom: Val::Px(14.0),
                height: Val::Px(48.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                ..default()
            },))
                .with_children(|bar| {
                    for slot in 1..=5 {
                        let selected = slot == 1;
                        bar.spawn((
                            Node {
                                width: Val::Px(48.0),
                                height: Val::Px(48.0),
                                border: UiRect::all(Val::Px(1.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            ImageNode::new(if selected {
                                assets.ui_slot_hover.clone()
                            } else {
                                assets.ui_slot.clone()
                            }),
                            BorderColor::all(if selected {
                                Color::srgba(1.0, 0.82, 0.35, 0.95)
                            } else {
                                Color::srgba(0.4, 0.45, 0.44, 0.75)
                            }),
                        ))
                        .with_children(|slot_node| {
                            if selected {
                                slot_node.spawn((
                                    ImageNode::new(assets.player_gun.clone()),
                                    Node {
                                        width: Val::Px(34.0),
                                        height: Val::Px(14.0),
                                        ..default()
                                    },
                                ));
                            }
                            slot_node.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(3.0),
                                    top: Val::Px(1.0),
                                    ..default()
                                },
                                Text::new(slot.to_string()),
                                ui_font(&assets, 11.0, true),
                                TextColor(if selected {
                                    Color::srgb(1.0, 0.86, 0.4)
                                } else {
                                    Color::srgba(0.6, 0.66, 0.64, 0.8)
                                }),
                            ));
                        });
                    }
                });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(14.0),
                    bottom: Val::Px(70.0),
                    ..default()
                },
                Text::new("WASD MOVE   SHIFT SPRINT   LMB FIRE   R RELOAD   T TURRET   ESC PAUSE"),
                ui_font(&assets, 12.0, false),
                TextColor(Color::srgba(0.66, 0.72, 0.7, 0.92)),
                TextShadow {
                    offset: Vec2::new(1.0, 1.0),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.9),
                },
                HudText::Hint,
            ));

            // -------------------------------------------------------------
            // ammo panel, bottom-right
            // -------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(14.0),
                    bottom: Val::Px(14.0),
                    width: Val::Px(176.0),
                    height: Val::Px(104.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                framed_image(assets.ui_panel.clone(), 5.0),
                BorderColor::all(Color::srgba(0.84, 0.69, 0.36, 0.55)),
                BackgroundColor(Color::srgba(0.015, 0.025, 0.03, 0.72)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(14.0),
                        top: Val::Px(8.0),
                        width: Val::Px(14.0),
                        height: Val::Px(14.0),
                        ..default()
                    },
                    ImageNode {
                        image: assets.icon_ammo.clone(),
                        color: Color::srgba(0.95, 0.86, 0.6, 0.9),
                        ..default()
                    },
                ));
                panel.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(14.0),
                        top: Val::Px(8.0),
                        ..default()
                    },
                    Text::new("5.56  //  RIFLE"),
                    ui_font(&assets, 11.0, false),
                    TextColor(Color::srgba(0.66, 0.72, 0.7, 0.95)),
                ));
                panel.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(14.0),
                        top: Val::Px(24.0),
                        width: Val::Px(148.0),
                        ..default()
                    },
                    Text::new("24/24"),
                    ui_font(&assets, 30.0, true),
                    TextColor(Color::srgb(0.95, 0.86, 0.6)),
                    TextShadow {
                        offset: Vec2::new(2.0, 2.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.85),
                    },
                    TextLayout::justify(Justify::Right),
                    HudText::Ammo,
                ));

                // segmented magazine gauge: one tick per round
                panel
                    .spawn((Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(14.0),
                        top: Val::Px(64.0),
                        width: Val::Px(148.0),
                        height: Val::Px(12.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(2.0),
                        ..default()
                    },))
                    .with_children(|gauge| {
                        for index in 0..24 {
                            gauge.spawn((
                                Node {
                                    width: Val::Px(4.0),
                                    height: Val::Px(12.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.1, 0.12, 0.12)),
                                BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                                MagTick(index),
                            ));
                        }
                    });

                panel.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(14.0),
                        top: Val::Px(84.0),
                        width: Val::Px(148.0),
                        height: Val::Px(6.0),
                        border: UiRect::all(Val::Px(1.0)),
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.07, 0.07, 0.9)),
                    BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                    ReloadBar,
                ));
            });

            // -------------------------------------------------------------
            // damage flash + crosshair
            // -------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.75, 0.05, 0.03, 0.0)),
                ZIndex(50),
                DamageFlash { timer: 0.0 },
            ));

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(15.0),
                    height: Val::Px(15.0),
                    ..default()
                },
                ImageNode::new(assets.crosshair.clone()),
                ZIndex(100),
                Crosshair,
            ));
        });
}

fn spawn_vital_bar(
    parent: &mut ChildSpawnerCommands,
    assets: &GameAssets,
    kind: VitalBar,
    left: f32,
) {
    let (icon, value_left) = match kind {
        VitalBar::Health => (assets.icon_health.clone(), 16.0),
        VitalBar::Stamina => (assets.icon_stamina.clone(), 100.0),
    };

    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left + 18.0),
                top: Val::Px(32.0),
                width: Val::Px(24.0),
                height: Val::Px(84.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.05, 0.92)),
            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        ))
        .with_children(|track| {
            track.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(match kind {
                    VitalBar::Health => Color::srgb(0.24, 0.86, 0.36),
                    VitalBar::Stamina => Color::srgb(0.94, 0.78, 0.24),
                }),
                kind,
            ));
        });

    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(value_left + 23.0),
            top: Val::Px(12.0),
            width: Val::Px(16.0),
            height: Val::Px(16.0),
            ..default()
        },
        ImageNode::new(icon),
    ));

    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(value_left),
            top: Val::Px(124.0),
            width: Val::Px(60.0),
            ..default()
        },
        Text::new("100"),
        ui_font(assets, 17.0, true),
        TextColor(match kind {
            VitalBar::Health => Color::srgb(0.5, 0.95, 0.55),
            VitalBar::Stamina => Color::srgb(0.97, 0.86, 0.45),
        }),
        TextLayout::justify(Justify::Center),
        match kind {
            VitalBar::Health => HudText::Health,
            VitalBar::Stamina => HudText::Stamina,
        },
    ));
}

fn text_node(left: Val, top: Val) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left,
        top,
        ..default()
    }
}

#[allow(clippy::too_many_arguments)]
fn update_hud(
    player: Query<(&Health, &Stamina, &Weapon), With<Player>>,
    enemies: Query<(), With<Enemy>>,
    stats: Res<RunStats>,
    mut texts: Query<(&mut Text, &HudText)>,
    mut widgets: Query<(
        &mut Node,
        &mut BackgroundColor,
        Option<&VitalBar>,
        Option<&MagTick>,
        Option<&ReloadBar>,
    )>,
    mut flashes: Query<&mut DamageFlash>,
    mut last_health: Local<f32>,
) {
    let Ok((health, stamina, weapon)) = player.single() else {
        return;
    };

    let mag = weapon.mag as usize;
    for (mut node, mut color, vital, tick, reload) in &mut widgets {
        if let Some(kind) = vital {
            let fraction = match kind {
                VitalBar::Health => health.fraction(),
                VitalBar::Stamina => stamina.current / stamina.max,
            };
            node.height = Val::Percent((fraction * 100.0).clamp(0.0, 100.0));
            *color = BackgroundColor(match kind {
                VitalBar::Health => {
                    if fraction > 0.55 {
                        Color::srgb(0.24, 0.86, 0.36)
                    } else if fraction > 0.25 {
                        Color::srgb(0.95, 0.72, 0.2)
                    } else {
                        Color::srgb(0.95, 0.2, 0.14)
                    }
                }
                VitalBar::Stamina => {
                    if fraction > 0.25 {
                        Color::srgb(0.94, 0.78, 0.24)
                    } else {
                        Color::srgb(0.85, 0.45, 0.18)
                    }
                }
            });
        } else if let Some(tick) = tick {
            *color = BackgroundColor(if tick.0 < mag {
                if weapon.reloading {
                    Color::srgb(0.55, 0.75, 0.8)
                } else {
                    Color::srgb(0.93, 0.8, 0.42)
                }
            } else {
                Color::srgb(0.1, 0.12, 0.12)
            });
        } else if reload.is_some() {
            node.display = if weapon.reloading {
                Display::Flex
            } else {
                Display::None
            };
            node.width = Val::Percent(weapon.reload_fraction() * 100.0);
        }
    }

    let hostiles = enemies.iter().count();
    for (mut text, kind) in &mut texts {
        match kind {
            HudText::Health => text.0 = format!("{:.0}", health.current),
            HudText::Stamina => text.0 = format!("{:.0}", stamina.current),
            HudText::Ammo => text.0 = format!("{}/{}", weapon.mag, weapon.mag_size),
            HudText::Hostiles => text.0 = format!("HOSTILES {}", hostiles),
            HudText::Kills => text.0 = format!("KILLS {}", stats.kills),
            HudText::Hint => {}
        }
    }

    // red flash when the player takes a hit
    let dropped = health.current < *last_health - 0.01;
    *last_health = health.current;
    for mut flash in &mut flashes {
        if dropped {
            flash.timer = 0.35;
        }
    }
}

/// Cached kill count so the HUD text pass can read it without a second query.
fn count_kills(mut kills: MessageReader<EnemyKilled>, mut stats: ResMut<RunStats>) {
    for _ in kills.read() {
        stats.kills += 1;
    }
}

fn move_crosshair(
    window: Single<&Window, With<PrimaryWindow>>,
    paused: Res<Paused>,
    mut crosshair: Query<(&mut Node, &mut Visibility), With<Crosshair>>,
    mut flashes: Query<(&mut BackgroundColor, &mut DamageFlash)>,
    time: Res<Time>,
) {
    let cursor = window.cursor_position();
    let visible = !paused.0 && cursor.is_some();

    for (mut node, mut visibility) in &mut crosshair {
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if let Some(cursor) = cursor {
            node.left = Val::Px(cursor.x - 7.0);
            node.top = Val::Px(cursor.y - 7.0);
        }
    }

    for (mut color, mut flash) in &mut flashes {
        let alpha = (flash.timer / 0.35) * 0.42;
        color.0 = Color::srgba(0.75, 0.05, 0.03, alpha.max(0.0));
        if flash.timer > 0.0 {
            flash.timer = (flash.timer - time.delta_secs()).max(0.0);
        }
    }
}

/// The OS cursor is hidden while playing; the sprite crosshair replaces it.
fn sync_cursor(
    state: Res<State<GameState>>,
    paused: Res<Paused>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let visible = *state.get() != GameState::Playing || paused.0;
    for mut cursor in &mut cursors {
        if cursor.visible != visible {
            cursor.visible = visible;
        }
    }
}

// ---------------------------------------------------------------------------
// pause + game over
// ---------------------------------------------------------------------------

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<Paused>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if paused.0 && keys.just_pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    } else if keys.just_pressed(KeyCode::Escape) {
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
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.015, 0.02, 0.8)),
                ZIndex(90),
                PauseOverlay,
                DespawnOnExit(GameState::Playing),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("PAUSED"),
                    ui_font(&assets, 56.0, true),
                    TextColor(Color::srgb(1.0, 0.82, 0.4)),
                    TextShadow {
                        offset: Vec2::new(3.0, 3.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                    },
                ));
                parent.spawn((
                    Text::new("ESC  RESUME     Q  QUIT"),
                    ui_font(&assets, 18.0, false),
                    TextColor(Color::srgb(0.75, 0.8, 0.8)),
                ));
            });
    } else if !paused.0 {
        if let Some(entity) = existing {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_game_over(mut commands: Commands, assets: Res<GameAssets>, stats: Res<RunStats>) {
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
            BackgroundColor(Color::srgba(0.03, 0.012, 0.01, 0.94)),
            DespawnOnExit(GameState::GameOver),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(640.0),
                    height: Val::Px(360.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(14.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                framed_image(assets.ui_panel.clone(), 5.0),
                BorderColor::all(Color::srgba(0.85, 0.25, 0.18, 0.8)),
                BackgroundColor(Color::srgba(0.07, 0.02, 0.02, 0.6)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("YOU DIED"),
                    ui_font(&assets, 64.0, true),
                    TextColor(Color::srgb(0.95, 0.25, 0.16)),
                    TextShadow {
                        offset: Vec2::new(3.0, 4.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.9),
                    },
                ));
                panel.spawn((
                    Text::new(ZONE_NAME),
                    ui_font(&assets, 19.0, false),
                    TextColor(Color::srgb(0.6, 0.7, 0.7)),
                ));
                panel.spawn((
                    Node {
                        width: Val::Px(480.0),
                        height: Val::Px(2.0),
                        margin: UiRect::vertical(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.85, 0.25, 0.18, 0.5)),
                ));
                panel.spawn((
                    Text::new(format!("KILLS {}", stats.kills)),
                    ui_font(&assets, 26.0, true),
                    TextColor(Color::srgb(0.92, 0.86, 0.76)),
                ));
                panel.spawn((
                    Text::new("ENTER  WALK OUT AGAIN      ESC  TITLE"),
                    ui_font(&assets, 19.0, false),
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
