use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::{Collider, StaticCollider};
use crate::states::GameState;

pub const TILE: f32 = 32.0;
pub const MAP_W: i32 = 64;
pub const MAP_H: i32 = 48;
pub const HALF: Vec2 = Vec2::new(1024.0, 768.0);
pub const PLAYER_SPAWN: Vec2 = Vec2::new(-16.0, -204.0);

// z layers ---------------------------------------------------------------
pub const Z_DECAL: f32 = 0.5;
pub const Z_SHADOW: f32 = 1.0;
pub const Z_DARK: f32 = 100.0;
pub const Z_BRIGHT: f32 = 150.0;
pub const ACTOR_SORT_OFFSET: f32 = 18.0;

/// Actors and props sort slightly by y so southern things draw on top.
pub fn y_sort(y: f32) -> f32 {
    2.0 + (HALF.y - y) * 0.0005
}

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
        Self { half: HALF }
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

/// Deterministic per-cell hash so the map looks identical every run.
fn hash(a: i32, b: i32) -> u32 {
    let mut x = (a as u32).wrapping_mul(0x9E37_79B9) ^ (b as u32).wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^ (x >> 16)
}

fn wp(col: i32, row: i32) -> Vec2 {
    Vec2::new(
        -HALF.x + col as f32 * TILE + TILE * 0.5,
        -HALF.y + row as f32 * TILE + TILE * 0.5,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ground {
    Sand,
    Dirt,
    Gravel,
    Drygrass,
    Asphalt,
    DoubleH,
    DoubleV,
    CrossH,
    CrossV,
    Sidewalk,
    Checker,
    Wood,
    Concrete,
}

fn in_rect(
    col: i32,
    row: i32,
    cols: std::ops::RangeInclusive<i32>,
    rows: std::ops::RangeInclusive<i32>,
) -> bool {
    cols.contains(&col) && rows.contains(&row)
}

const STORE_COLS: std::ops::RangeInclusive<i32> = 6..=23;
const STORE_ROWS: std::ops::RangeInclusive<i32> = 30..=44;
const SHOP_COLS: std::ops::RangeInclusive<i32> = 46..=58;
const SHOP_ROWS: std::ops::RangeInclusive<i32> = 35..=44;

fn ground_at(col: i32, row: i32) -> Ground {
    let on_hw = (21..=27).contains(&row);
    let on_vt = (28..=34).contains(&col);

    // ruined store interior (NW block)
    if in_rect(col, row, 7..=22, 31..=43) {
        return if col <= 11 && row >= 40 {
            Ground::Wood
        } else {
            Ground::Checker
        };
    }
    // gas station shop interior (NE block)
    if in_rect(col, row, 47..=57, 36..=43) {
        return Ground::Checker;
    }
    // gas station forecourt
    if in_rect(col, row, 36..=61, 30..=44) {
        return Ground::Concrete;
    }

    if on_hw || on_vt {
        if on_hw && (col == 26 || col == 27 || col == 35 || col == 36) {
            return Ground::CrossH;
        }
        if on_vt && (row == 19 || row == 20 || row == 28 || row == 29) {
            return Ground::CrossV;
        }
        if on_hw && on_vt {
            return Ground::Asphalt;
        }
        if on_hw {
            return if row == 24 {
                Ground::DoubleH
            } else {
                Ground::Asphalt
            };
        }
        return if col == 31 {
            Ground::DoubleV
        } else {
            Ground::Asphalt
        };
    }

    if (row == 20 || row == 28) && !on_vt {
        return Ground::Sidewalk;
    }
    if (col == 27 || col == 35) && !on_hw {
        return Ground::Sidewalk;
    }

    // patchy desert terrain: region noise plus per-cell speckle
    let region = hash(col / 3, row / 3) % 24;
    let base = match region {
        0..=11 => Ground::Sand,
        12..=16 => Ground::Dirt,
        17..=19 => Ground::Gravel,
        _ => Ground::Drygrass,
    };
    let speck = hash(col + 71, row * 3 + 5) % 32;
    if speck == 0 {
        Ground::Dirt
    } else if speck == 1 {
        Ground::Gravel
    } else {
        base
    }
}

fn is_wall_cell(col: i32, row: i32) -> Option<&'static str> {
    // ruined brick store: perimeter with a collapsed south entrance
    if in_rect(col, row, STORE_COLS, STORE_ROWS) {
        let perimeter = col == 6 || col == 23 || row == 30 || row == 44;
        if perimeter {
            if row == 30 && (13..=15).contains(&col) {
                return None;
            }
            return Some(if hash(col, row) % 10 < 3 {
                "brick_broken"
            } else {
                "brick"
            });
        }
        return None;
    }
    // gas station shop: perimeter with a west doorway
    if in_rect(col, row, SHOP_COLS, SHOP_ROWS) {
        let perimeter = col == 46 || col == 58 || row == 35 || row == 44;
        if perimeter {
            if col == 46 && (39..=41).contains(&row) {
                return None;
            }
            let h = hash(col + 9, row + 4) % 10;
            return Some(if h < 2 {
                "metal"
            } else if h < 5 {
                "concrete_broken"
            } else {
                "concrete"
            });
        }
    }
    None
}

fn spawn_shadow(commands: &mut Commands, assets: &GameAssets, position: Vec2, size: Vec2) {
    commands.spawn((
        Sprite {
            image: assets.shadow.clone(),
            custom_size: Some(size),
            color: Color::srgba(1.0, 1.0, 1.0, 0.55),
            ..default()
        },
        Transform::from_xyz(position.x + 4.0, position.y - 5.0, Z_SHADOW),
        DespawnOnExit(GameState::Playing),
    ));
}

#[allow(clippy::too_many_arguments)]
fn spawn_prop(
    commands: &mut Commands,
    assets: &GameAssets,
    image: Handle<Image>,
    position: Vec2,
    size: Vec2,
    collider: Option<Collider>,
    rotation: f32,
    shadow: Vec2,
) {
    spawn_shadow(commands, assets, position, shadow);
    let sort_extent_y = rotation.sin().abs() * size.x * 0.5 + rotation.cos().abs() * size.y * 0.5;
    let mut entity = commands.spawn((
        Sprite {
            image,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, y_sort(position.y - sort_extent_y))
            .with_rotation(Quat::from_rotation_z(rotation)),
        DespawnOnExit(GameState::Playing),
    ));
    if let Some(collider) = collider {
        entity.insert(StaticCollider(collider));
    }
}

fn ground_image(assets: &GameAssets, ground: Ground, col: i32, row: i32) -> Handle<Image> {
    let h = hash(col + 13, row + 29);
    match ground {
        Ground::Sand => assets.tiles_sand[(h % 3) as usize].clone(),
        Ground::Dirt => assets.tiles_dirt[(h % 3) as usize].clone(),
        Ground::Gravel => assets.tiles_gravel[(h % 2) as usize].clone(),
        Ground::Drygrass => assets.tiles_drygrass[(h % 2) as usize].clone(),
        Ground::Asphalt => assets.tiles_asphalt[(h % 3) as usize].clone(),
        Ground::DoubleH => assets.road_double_h.clone(),
        Ground::DoubleV => assets.road_double_v.clone(),
        Ground::CrossH => assets.road_cross_h.clone(),
        Ground::CrossV => assets.road_cross_v.clone(),
        Ground::Sidewalk => assets.tiles_sidewalk[(h % 2) as usize].clone(),
        Ground::Checker => assets.tiles_checker[(h % 2) as usize].clone(),
        Ground::Wood => assets.tiles_wood[(h % 2) as usize].clone(),
        Ground::Concrete => assets.tiles_concrete[(h % 2) as usize].clone(),
    }
}

fn wall_image(assets: &GameAssets, kind: &str, col: i32, row: i32) -> Handle<Image> {
    let h = hash(col + 5, row + 17);
    match kind {
        "brick" => assets.walls_brick[(h % 2) as usize].clone(),
        "brick_broken" => assets.walls_brick_broken[(h % 2) as usize].clone(),
        "concrete" => assets.walls_concrete[(h % 2) as usize].clone(),
        "concrete_broken" => assets.wall_concrete_broken.clone(),
        _ => assets.walls_metal[(h % 2) as usize].clone(),
    }
}

fn spawn_decal(
    commands: &mut Commands,
    image: Handle<Image>,
    position: Vec2,
    size: f32,
    rotation: f32,
    alpha: f32,
) {
    commands.spawn((
        Sprite {
            image,
            custom_size: Some(Vec2::splat(size)),
            color: Color::srgba(1.0, 1.0, 1.0, alpha),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, Z_DECAL)
            .with_rotation(Quat::from_rotation_z(rotation)),
        GroundDecal,
        DespawnOnExit(GameState::Playing),
    ));
}

fn spawn_arena(mut commands: Commands, assets: Res<GameAssets>) {
    let half_turn = std::f32::consts::FRAC_PI_2;

    // -----------------------------------------------------------------
    // ground tiles
    // -----------------------------------------------------------------
    for col in 0..MAP_W {
        for row in 0..MAP_H {
            let ground = ground_at(col, row);
            let position = wp(col, row);
            commands.spawn((
                Sprite {
                    image: ground_image(&assets, ground, col, row),
                    custom_size: Some(Vec2::splat(TILE + 1.0)),
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, 0.0),
                Tile,
                DespawnOnExit(GameState::Playing),
            ));
        }
    }

    // -----------------------------------------------------------------
    // ruined building walls
    // -----------------------------------------------------------------
    for (cols, rows) in [(STORE_COLS, STORE_ROWS), (SHOP_COLS, SHOP_ROWS)] {
        for col in *cols.start()..=*cols.end() {
            for row in *rows.start()..=*rows.end() {
                let Some(kind) = is_wall_cell(col, row) else {
                    continue;
                };
                let position = wp(col, row);
                let broken = kind.contains("broken");
                commands.spawn((
                    Sprite {
                        image: wall_image(&assets, kind, col, row),
                        custom_size: Some(Vec2::splat(TILE + 1.0)),
                        ..default()
                    },
                    Transform::from_xyz(
                        position.x,
                        position.y,
                        y_sort(position.y - (TILE + 1.0) * 0.5),
                    ),
                    StaticCollider(Collider::aabb(Vec2::splat(16.0))),
                    DespawnOnExit(GameState::Playing),
                ));
                if broken && hash(col + 3, row + 8) % 2 == 0 {
                    spawn_decal(
                        &mut commands,
                        assets.decal_rubble[(hash(col, row) % 2) as usize].clone(),
                        position + Vec2::new(0.0, -14.0),
                        34.0,
                        0.0,
                        0.95,
                    );
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // lot props
    // -----------------------------------------------------------------
    // south-west lot: the squatter camp
    let p = |c: i32, r: i32| wp(c, r);
    spawn_prop(
        &mut commands,
        &assets,
        assets.debris_piles[0].clone(),
        p(12, 5),
        Vec2::new(34.0, 28.0),
        Some(Collider::circle(12.0)),
        0.0,
        Vec2::new(38.0, 30.0),
    );
    spawn_prop(
        &mut commands,
        &assets,
        assets.dead_trees[0].clone(),
        p(3, 7),
        Vec2::splat(46.0),
        Some(Collider::circle(9.0)),
        0.0,
        Vec2::splat(34.0),
    );
    spawn_prop(
        &mut commands,
        &assets,
        assets.dead_trees[1].clone(),
        p(21, 13),
        Vec2::splat(46.0),
        Some(Collider::circle(9.0)),
        0.0,
        Vec2::splat(34.0),
    );
    // north-east lot: gas station
    spawn_prop(
        &mut commands,
        &assets,
        assets.debris_piles[1].clone(),
        p(61, 41),
        Vec2::new(34.0, 28.0),
        Some(Collider::circle(12.0)),
        0.0,
        Vec2::new(38.0, 30.0),
    );

    // shop fittings inside the gas station
    spawn_prop(
        &mut commands,
        &assets,
        assets.counter.clone(),
        p(49, 37),
        Vec2::new(48.0, 16.0),
        Some(Collider::aabb(Vec2::new(24.0, 8.0))),
        0.0,
        Vec2::new(52.0, 18.0),
    );
    // store fittings
    spawn_prop(
        &mut commands,
        &assets,
        assets.counter.clone(),
        p(9, 32),
        Vec2::new(48.0, 16.0),
        Some(Collider::aabb(Vec2::new(24.0, 8.0))),
        0.0,
        Vec2::new(52.0, 18.0),
    );
    // south-east wasteland lot
    spawn_prop(
        &mut commands,
        &assets,
        assets.dead_trees[0].clone(),
        p(60, 14),
        Vec2::splat(46.0),
        Some(Collider::circle(9.0)),
        0.0,
        Vec2::splat(34.0),
    );
    spawn_prop(
        &mut commands,
        &assets,
        assets.debris_piles[0].clone(),
        p(44, 4),
        Vec2::new(34.0, 28.0),
        Some(Collider::circle(12.0)),
        0.0,
        Vec2::new(38.0, 30.0),
    );
    spawn_prop(
        &mut commands,
        &assets,
        assets.dead_trees[1].clone(),
        p(25, 46),
        Vec2::splat(46.0),
        Some(Collider::circle(9.0)),
        0.0,
        Vec2::splat(34.0),
    );

    // -----------------------------------------------------------------
    // scattered ground decals
    // -----------------------------------------------------------------
    let near_wall = |col: i32, row: i32| -> bool {
        (STORE_COLS.start() - 2..=STORE_COLS.end() + 2).contains(&col)
            && (STORE_ROWS.start() - 2..=STORE_ROWS.end() + 2).contains(&row)
            || (SHOP_COLS.start() - 2..=SHOP_COLS.end() + 2).contains(&col)
                && (SHOP_ROWS.start() - 2..=SHOP_ROWS.end() + 2).contains(&row)
    };

    for col in 0..MAP_W {
        for row in 0..MAP_H {
            let h1 = hash(col, row);
            let h2 = hash(col + 401, row + 77);
            let h3 = hash(col.wrapping_sub(133), row * 5 + 3);
            let ground = ground_at(col, row);
            let position = wp(col, row) + Vec2::new((h2 % 13) as f32 - 6.0, (h3 % 13) as f32 - 6.0);
            let rotation = ((h1 >> 4) % 4) as f32 * half_turn;

            let is_road = matches!(ground, Ground::Asphalt | Ground::DoubleH | Ground::DoubleV);
            let is_open = matches!(ground, Ground::Sand | Ground::Drygrass | Ground::Dirt);

            if h1 % 100 < 11 {
                spawn_decal(
                    &mut commands,
                    assets.decal_leaf[(h2 % 2) as usize].clone(),
                    position,
                    32.0,
                    rotation,
                    0.9,
                );
            }
            if is_open && h2 % 100 < 17 {
                spawn_decal(
                    &mut commands,
                    assets.decal_grass[(h3 % 2) as usize].clone(),
                    position,
                    32.0,
                    rotation,
                    0.95,
                );
            }
            if near_wall(col, row) && h3 % 100 < 34 {
                spawn_decal(
                    &mut commands,
                    assets.decal_rubble[(h1 % 2) as usize].clone(),
                    position,
                    32.0,
                    rotation,
                    0.95,
                );
            } else if h1 % 100 < 3 {
                spawn_decal(
                    &mut commands,
                    assets.decal_rubble[(h2 % 2) as usize].clone(),
                    position,
                    32.0,
                    rotation,
                    0.85,
                );
            }
            if is_road && h2 % 100 < 9 {
                spawn_decal(
                    &mut commands,
                    assets.decal_crack[(h3 % 2) as usize].clone(),
                    position,
                    32.0,
                    rotation,
                    0.9,
                );
            }
            if is_road && h1 % 100 < 3 {
                spawn_decal(
                    &mut commands,
                    assets.decal_oil.clone(),
                    position,
                    32.0,
                    rotation,
                    0.9,
                );
            }
        }
    }

    // story decals: something went down here
    spawn_decal(
        &mut commands,
        assets.decal_blood[0].clone(),
        p(18, 14),
        36.0,
        0.0,
        0.95,
    );
    spawn_decal(
        &mut commands,
        assets.decal_blood[1].clone(),
        p(50, 10),
        36.0,
        1.7,
        0.95,
    );
    spawn_decal(
        &mut commands,
        assets.decal_blood[0].clone(),
        p(33, 38),
        32.0,
        3.1,
        0.9,
    );
    spawn_decal(
        &mut commands,
        assets.decal_scorch.clone(),
        p(47, 19),
        40.0,
        0.4,
        0.9,
    );
    spawn_decal(
        &mut commands,
        assets.decal_scorch.clone(),
        p(9, 19),
        40.0,
        2.2,
        0.9,
    );
}

#[cfg(test)]
mod tests {
    use super::y_sort;

    #[test]
    fn lower_world_y_sorts_in_front() {
        assert!(y_sort(-10.0) > y_sort(10.0));
    }
}
