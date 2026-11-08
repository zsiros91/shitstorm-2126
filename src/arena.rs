use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::{Collider, StaticCollider};
use crate::states::GameState;

pub const TILE_SIZE: f32 = 64.0;
pub const PLAYER_SPAWN: Vec2 = Vec2::new(0.0, -300.0);

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct GroundDecal;

#[derive(Resource)]
pub struct Arena {
    pub half: Vec2,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            half: Vec2::new(1280.0, 800.0),
        }
    }
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Arena>()
            .add_systems(OnEnter(GameState::Playing), spawn_arena);
    }
}

pub fn random_edge_point(arena: &Arena, margin: f32) -> Vec2 {
    let half = arena.half - Vec2::splat(margin);
    match fastrand::u32(0..4) {
        0 => Vec2::new(fastrand::f32() * half.x * 2.0 - half.x, half.y),
        1 => Vec2::new(fastrand::f32() * half.x * 2.0 - half.x, -half.y),
        2 => Vec2::new(-half.x, fastrand::f32() * half.y * 2.0 - half.y),
        _ => Vec2::new(half.x, fastrand::f32() * half.y * 2.0 - half.y),
    }
}

fn spawn_shadow(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec2,
    size: Vec2,
    z: f32,
) {
    commands.spawn((
        Sprite {
            image: assets.shadow.clone(),
            custom_size: Some(size),
            color: Color::srgba(1.0, 1.0, 1.0, 0.55),
            ..default()
        },
        Transform::from_xyz(position.x + 8.0, position.y - 10.0, z),
        DespawnOnExit(GameState::Playing),
    ));
}

fn spawn_arena(mut commands: Commands, assets: Res<GameAssets>, arena: Res<Arena>) {
    let columns = (arena.half.x * 2.0 / TILE_SIZE).round() as i32;
    let rows = (arena.half.y * 2.0 / TILE_SIZE).round() as i32;

    // Landing pad under the command centre.
    commands.spawn((
        Sprite {
            image: assets.base_pad.clone(),
            custom_size: Some(Vec2::splat(620.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -12.0),
        GroundDecal,
        DespawnOnExit(GameState::Playing),
    ));

    for ix in 0..columns {
        for iy in 0..rows {
            let position = Vec2::new(
                -arena.half.x + TILE_SIZE * 0.5 + ix as f32 * TILE_SIZE,
                -arena.half.y + TILE_SIZE * 0.5 + iy as f32 * TILE_SIZE,
            );

            let dist = position.length() / arena.half.length();
            let edge = ix.min(iy).min(columns - 1 - ix).min(rows - 1 - iy);
            let noise = fastrand::f32();

            let (image, tint) = if edge <= 1 {
                let rock = fastrand::u32(0..2) as usize;
                (
                    assets.tiles_rock[rock].clone(),
                    Color::srgb(0.72, 0.72, 0.74),
                )
            } else if edge == 2 && noise < 0.55 {
                let creep = fastrand::u32(0..2) as usize;
                (
                    assets.tiles_creep[creep].clone(),
                    Color::srgb(0.78, 0.68, 0.9),
                )
            } else if dist < 0.22 {
                let pick = fastrand::f32();
                if pick < 0.12 {
                    (assets.tile_grate.clone(), Color::WHITE)
                } else if pick < 0.18 {
                    (assets.tile_rust.clone(), Color::srgb(0.9, 0.82, 0.75))
                } else {
                    let idx = fastrand::u32(0..3) as usize;
                    (assets.tiles_metal[idx].clone(), Color::srgb(0.92, 0.96, 1.0))
                }
            } else if noise < 0.18 + dist * 0.25 {
                (
                    assets.tile_rust.clone(),
                    Color::srgb(0.85, 0.78, 0.72),
                )
            } else {
                let idx = fastrand::u32(0..3) as usize;
                (
                    assets.tiles_metal[idx].clone(),
                    Color::srgb(0.72, 0.78, 0.84),
                )
            };

            commands.spawn((
                Sprite {
                    image,
                    custom_size: Some(Vec2::splat(TILE_SIZE + 1.0)),
                    color: tint,
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, -10.0),
                Tile,
                DespawnOnExit(GameState::Playing),
            ));
        }
    }

    for &(kind, x, y) in PROPS {
        let (image, size, collider, shadow_size) = match kind {
            0 => (
                assets.crate_metal.clone(),
                Vec2::splat(52.0),
                Collider::aabb(Vec2::splat(23.0)),
                Vec2::splat(54.0),
            ),
            1 => (
                assets.crate_wood.clone(),
                Vec2::splat(52.0),
                Collider::aabb(Vec2::splat(23.0)),
                Vec2::splat(54.0),
            ),
            2 => (
                assets.barrel_green.clone(),
                Vec2::splat(38.0),
                Collider::circle(15.0),
                Vec2::splat(42.0),
            ),
            3 => (
                assets.barrel_red.clone(),
                Vec2::splat(38.0),
                Collider::circle(15.0),
                Vec2::splat(42.0),
            ),
            4 => (
                assets.rock.clone(),
                Vec2::splat(86.0),
                Collider::circle(27.0),
                Vec2::splat(82.0),
            ),
            5 => (
                assets.sandbag.clone(),
                Vec2::new(72.0, 32.0),
                Collider::aabb(Vec2::new(32.0, 13.0)),
                Vec2::new(76.0, 34.0),
            ),
            6 => (
                assets.pipe.clone(),
                Vec2::new(150.0, 56.0),
                Collider::aabb(Vec2::new(70.0, 18.0)),
                Vec2::new(150.0, 36.0),
            ),
            _ => (
                assets.crate_metal.clone(),
                Vec2::splat(52.0),
                Collider::aabb(Vec2::splat(23.0)),
                Vec2::splat(54.0),
            ),
        };

        spawn_shadow(&mut commands, &assets, Vec2::new(x, y), shadow_size, 1.0);
        commands.spawn((
            Sprite {
                image,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(x, y, 5.0),
            StaticCollider(collider),
            DespawnOnExit(GameState::Playing),
        ));
    }

    // Scorch marks and zerg growth around the perimeter.
    for _ in 0..18 {
        let position = Vec2::new(
            fastrand::f32() * arena.half.x * 2.0 - arena.half.x,
            fastrand::f32() * arena.half.y * 2.0 - arena.half.y,
        );
        if position.length() < 520.0 {
            continue;
        }
        let creep = fastrand::f32() < 0.35;
        commands.spawn((
            Sprite {
                image: if creep {
                    assets.creep_decal.clone()
                } else {
                    assets.scorch.clone()
                },
                custom_size: Some(Vec2::splat(if creep { 230.0 } else { 120.0 })),
                color: Color::srgba(1.0, 1.0, 1.0, if creep { 0.55 } else { 0.8 }),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, -8.5)
                .with_rotation(Quat::from_rotation_z(fastrand::f32() * std::f32::consts::TAU)),
            GroundDecal,
            DespawnOnExit(GameState::Playing),
        ));
    }
}

const PROPS: &[(u8, f32, f32)] = &[
    (0, -880.0, 470.0),
    (0, -810.0, 520.0),
    (1, 760.0, -430.0),
    (1, 820.0, -370.0),
    (2, -480.0, -610.0),
    (3, -420.0, -650.0),
    (2, 930.0, 500.0),
    (3, 990.0, 450.0),
    (4, -1110.0, -180.0),
    (4, 1090.0, 280.0),
    (4, 180.0, 700.0),
    (5, -340.0, -700.0),
    (5, 440.0, 600.0),
    (6, -1050.0, 260.0),
    (6, 700.0, 120.0),
    (4, -700.0, 700.0),
    (4, 1050.0, -650.0),
];
