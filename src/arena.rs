use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::{Collider, StaticCollider};
use crate::states::GameState;

pub const TILE_SIZE: f32 = 64.0;

#[derive(Component)]
pub struct Tile;

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
        0 => Vec2::new(
            fastrand::f32() * half.x * 2.0 - half.x,
            half.y,
        ),
        1 => Vec2::new(
            fastrand::f32() * half.x * 2.0 - half.x,
            -half.y,
        ),
        2 => Vec2::new(
            -half.x,
            fastrand::f32() * half.y * 2.0 - half.y,
        ),
        _ => Vec2::new(
            half.x,
            fastrand::f32() * half.y * 2.0 - half.y,
        ),
    }
}

fn spawn_arena(mut commands: Commands, assets: Res<GameAssets>, arena: Res<Arena>) {
    let columns = (arena.half.x * 2.0 / TILE_SIZE) as i32;
    let rows = (arena.half.y * 2.0 / TILE_SIZE) as i32;

    for ix in 0..columns {
        for iy in 0..rows {
            let position = Vec2::new(
                -arena.half.x + TILE_SIZE * 0.5 + ix as f32 * TILE_SIZE,
                -arena.half.y + TILE_SIZE * 0.5 + iy as f32 * TILE_SIZE,
            );

            let sandy = (ix / 4 + iy / 3) % 7 == 0;
            let variant = if sandy {
                2 + fastrand::u32(0..2) as usize
            } else {
                fastrand::u32(0..2) as usize
            };

            let tint = match variant {
                0 => Color::srgb(1.0, 0.4, 0.4),
                1 => Color::srgb(0.4, 1.0, 0.4),
                2 => Color::srgb(0.4, 0.6, 1.0),
                _ => Color::srgb(1.0, 1.0, 0.4),
            };

            commands.spawn((
                Sprite {
                    image: assets.tiles[variant].clone(),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
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
        let (image, size, collider) = match kind {
            0 => (
                assets.crate_wood.clone(),
                Vec2::splat(56.0),
                Collider::aabb(Vec2::splat(26.0)),
            ),
            1 => (
                assets.crate_metal.clone(),
                Vec2::splat(56.0),
                Collider::aabb(Vec2::splat(26.0)),
            ),
            2 => (
                assets.barrel_green.clone(),
                Vec2::splat(40.0),
                Collider::circle(16.0),
            ),
            3 => (
                assets.barrel_red.clone(),
                Vec2::splat(40.0),
                Collider::circle(16.0),
            ),
            4 => (
                assets.tree.clone(),
                Vec2::splat(96.0),
                Collider::circle(20.0),
            ),
            _ => (
                assets.sandbag.clone(),
                Vec2::new(64.0, 28.0),
                Collider::aabb(Vec2::new(30.0, 12.0)),
            ),
        };

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
}

const PROPS: &[(u8, f32, f32)] = &[
    (0, -900.0, 500.0),
    (0, -820.0, 560.0),
    (1, 700.0, -450.0),
    (1, 760.0, -390.0),
    (2, -500.0, -600.0),
    (3, -440.0, -640.0),
    (2, 950.0, 520.0),
    (3, 1010.0, 470.0),
    (4, -1100.0, -200.0),
    (4, 1100.0, 250.0),
    (4, 150.0, 700.0),
    (5, -200.0, -700.0),
    (5, 400.0, 620.0),
];
