use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::assets::GameAssets;
use crate::collision::StaticCollider;
use crate::economy::Economy;
use crate::fx::spawn_materialize;
use crate::hq::{Hq, HQ_COMMAND_RADIUS};
use crate::power::{is_connected, PowerNode, HQ_LINK_RANGE};
use crate::states::{gameplay_active, GameSet, GameState};
use crate::tower::{spawn_tower, TowerKind};

pub const BUILD_GRID: f32 = 16.0;

#[derive(Resource)]
pub struct BuildMode {
    pub active: bool,
    pub selected: TowerKind,
    pub valid: bool,
    pub status: String,
    pub suppress_pause: bool,
    ghost_kind: Option<TowerKind>,
}

impl Default for BuildMode {
    fn default() -> Self {
        Self {
            active: false,
            selected: TowerKind::Gauss,
            valid: false,
            status: String::new(),
            suppress_pause: false,
            ghost_kind: None,
        }
    }
}

#[derive(Component)]
pub struct BuildGhost;

#[derive(Component)]
pub struct BuildGhostRange;

#[derive(Component)]
pub struct BuildCommandVisual;

pub struct BuildPlugin;

impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildMode>()
            .add_systems(OnEnter(GameState::Playing), reset_build_mode)
            .add_systems(
                Update,
                (build_input, sync_command_radius, update_ghost, place_tower)
                    .chain()
                    .in_set(GameSet::Input)
                    .run_if(gameplay_active),
            );
    }
}

fn reset_build_mode(mut mode: ResMut<BuildMode>) {
    *mode = BuildMode::default();
}

fn build_input(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<BuildMode>) {
    if keys.just_pressed(KeyCode::KeyB) {
        mode.active = !mode.active;
        mode.status.clear();
    }

    if mode.active && keys.just_pressed(KeyCode::Escape) {
        mode.active = false;
        mode.suppress_pause = true;
        mode.status.clear();
    }

    if keys.just_pressed(KeyCode::Digit1) {
        mode.selected = TowerKind::Gauss;
        mode.active = true;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        mode.selected = TowerKind::Missile;
        mode.active = true;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        mode.selected = TowerKind::Arc;
        mode.active = true;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        mode.selected = TowerKind::Relay;
        mode.active = true;
    }
}

fn cursor_world_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

fn sync_command_radius(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mode: Res<BuildMode>,
    hq: Query<&Transform, With<Hq>>,
    visuals: Query<Entity, With<BuildCommandVisual>>,
) {
    if !mode.active {
        for entity in &visuals {
            commands.entity(entity).despawn();
        }
        return;
    }
    if visuals.iter().next().is_some() {
        return;
    }
    let Ok(hq_transform) = hq.single() else {
        return;
    };
    let hq_position = hq_transform.translation.xy();

    commands.spawn((
        Sprite {
            image: assets.ring.clone(),
            custom_size: Some(Vec2::splat(HQ_COMMAND_RADIUS * 2.0)),
            color: Color::srgba(0.95, 0.72, 0.28, 0.14),
            ..default()
        },
        Transform::from_xyz(hq_position.x, hq_position.y, -6.0),
        BuildCommandVisual,
        DespawnOnExit(GameState::Playing),
    ));
    commands.spawn((
        Sprite {
            image: assets.ring.clone(),
            custom_size: Some(Vec2::splat(HQ_LINK_RANGE * 2.0)),
            color: Color::srgba(0.3, 0.9, 1.0, 0.22),
            ..default()
        },
        Transform::from_xyz(hq_position.x, hq_position.y, -5.9),
        BuildCommandVisual,
        DespawnOnExit(GameState::Playing),
    ));
}

fn update_ghost(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    mut mode: ResMut<BuildMode>,
    economy: Res<Economy>,
    hq: Query<&Transform, With<Hq>>,
    nodes: Query<(&Transform, &PowerNode), Without<Hq>>,
    statics: Query<(&Transform, &StaticCollider)>,
    ghosts: Query<Entity, With<BuildGhost>>,
    mut parts: Query<(&mut Transform, &mut Sprite), (With<BuildGhost>, Without<BuildGhostRange>)>,
    mut ranges: Query<(&mut Transform, &mut Sprite), With<BuildGhostRange>>,
) {
    if !mode.active {
        if mode.ghost_kind.is_some() {
            for entity in &ghosts {
                commands.entity(entity).despawn();
            }
            mode.ghost_kind = None;
        }
        return;
    }
    let Ok(hq_transform) = hq.single() else {
        return;
    };
    let hq_position = hq_transform.translation.xy();
    let (camera, camera_transform) = *camera;
    let Some(cursor) = cursor_world_position(&window, camera, camera_transform) else {
        return;
    };
    let position = (cursor / BUILD_GRID).round() * BUILD_GRID;

    let node_list: Vec<(Vec2, f32, bool)> = nodes
        .iter()
        .map(|(transform, node)| (transform.translation.xy(), node.link_range, node.powered))
        .collect();
    let connected = is_connected(position, hq_position, &node_list);
    let inside = position.distance(hq_position) <= HQ_COMMAND_RADIUS;
    let radius = 30.0;
    let blocked = statics.iter().any(|(transform, collider)| {
        position.distance(transform.translation.xy()) < radius + collider.0.radius()
    });

    let (minerals, gas) = mode.selected.cost();
    let affordable = economy.can_afford(minerals, gas);
    mode.valid = inside && connected && !blocked && affordable;
    mode.status = if !inside {
        "OUTSIDE COMMAND RADIUS".to_string()
    } else if !connected {
        "NO POWER LINK TO HQ".to_string()
    } else if blocked {
        "PLACEMENT BLOCKED".to_string()
    } else if !affordable {
        if economy.minerals < minerals {
            "INSUFFICIENT MINERALS".to_string()
        } else {
            "INSUFFICIENT GAS".to_string()
        }
    } else {
        "CLICK TO MANIFEST".to_string()
    };

    if mode.ghost_kind != Some(mode.selected) {
        for entity in &ghosts {
            commands.entity(entity).despawn();
        }
        mode.ghost_kind = Some(mode.selected);
        let color = Color::srgba(0.35, 1.0, 0.6, 0.55);
        let base = mode.selected.sprite(&assets);
        let head = mode.selected.head_sprite(&assets);
        commands.spawn((
            Sprite {
                image: base,
                custom_size: Some(Vec2::splat(mode.selected.size())),
                color,
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 14.0),
            BuildGhost,
            DespawnOnExit(GameState::Playing),
        ));
        if let Some(head) = head {
            commands.spawn((
                Sprite {
                    image: head,
                    custom_size: Some(Vec2::splat(58.0)),
                    color,
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, 15.0),
                BuildGhost,
                DespawnOnExit(GameState::Playing),
            ));
        }
        commands.spawn((
            Sprite {
                image: assets.ring.clone(),
                custom_size: Some(Vec2::splat(if mode.selected == TowerKind::Relay {
                    mode.selected.link_range() * 2.0
                } else {
                    mode.selected.range() * 2.0
                })),
                color: Color::srgba(0.35, 0.9, 1.0, 0.3),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, -5.0),
            BuildGhost,
            BuildGhostRange,
            DespawnOnExit(GameState::Playing),
        ));
    }

    let color = if mode.valid {
        Color::srgba(0.35, 1.0, 0.6, 0.55)
    } else {
        Color::srgba(1.0, 0.35, 0.3, 0.55)
    };
    for (mut transform, mut sprite) in &mut parts {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        sprite.color = color;
    }
    for (mut transform, mut sprite) in &mut ranges {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        sprite.color = if mode.valid {
            Color::srgba(0.35, 0.9, 1.0, 0.34)
        } else {
            Color::srgba(1.0, 0.35, 0.3, 0.3)
        };
    }
}

fn place_tower(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut mode: ResMut<BuildMode>,
    mut economy: ResMut<Economy>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    ghosts: Query<Entity, With<BuildGhost>>,
) {
    if !mode.active {
        return;
    }
    if mouse.just_pressed(MouseButton::Right) {
        mode.active = false;
        mode.status.clear();
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) || !mode.valid {
        return;
    }
    if crate::ui::cursor_over_hud(&window) {
        return;
    }
    let (camera, camera_transform) = *camera;
    let Some(cursor) = cursor_world_position(&window, camera, camera_transform) else {
        return;
    };
    let position = (cursor / BUILD_GRID).round() * BUILD_GRID;
    let (minerals, gas) = mode.selected.cost();
    if !economy.spend(minerals, gas) {
        return;
    }
    spawn_tower(&mut commands, &assets, mode.selected, position);
    spawn_materialize(&mut commands, &assets, position);
    for entity in &ghosts {
        commands.entity(entity).despawn();
    }
    mode.ghost_kind = None;
    mode.status = "MANIFESTING".to_string();
}
