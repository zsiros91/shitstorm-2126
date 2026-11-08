use bevy::prelude::*;
use std::collections::HashSet;

use crate::assets::GameAssets;
use crate::states::GameState;

pub const HQ_RADIUS: f32 = 560.0;
pub const HQ_LINK_RANGE: f32 = 300.0;
pub const TOWER_LINK_RANGE: f32 = 280.0;
pub const RELAY_LINK_RANGE: f32 = 380.0;

#[derive(Component, Clone, Copy, Debug)]
pub struct PowerNode {
    pub link_range: f32,
    pub powered: bool,
    pub source: bool,
}

impl PowerNode {
    pub fn tower(range: f32) -> Self {
        Self {
            link_range: range,
            powered: false,
            source: false,
        }
    }
}

#[derive(Component)]
pub struct PowerSource;

#[derive(Component)]
pub struct PowerLinkVisual;

#[derive(Resource, Default)]
pub struct PowerGrid {
    pub links: Vec<(Entity, Entity)>,
    pub changed: bool,
}

pub struct PowerPlugin;

impl Plugin for PowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PowerGrid>()
            .add_systems(Update, (recompute_power, sync_power_links).chain());
    }
}

pub fn is_connected(position: Vec2, hq: Vec2, nodes: &[(Vec2, f32, bool)]) -> bool {
    if position.distance(hq) > HQ_RADIUS {
        return false;
    }
    if position.distance(hq) <= HQ_LINK_RANGE {
        return true;
    }
    nodes
        .iter()
        .any(|&(pos, range, powered)| powered && position.distance(pos) <= range)
}

fn recompute_power(
    hq: Query<(Entity, &Transform, &PowerNode), With<PowerSource>>,
    mut nodes: Query<(Entity, &Transform, &mut PowerNode), Without<PowerSource>>,
    mut grid: ResMut<PowerGrid>,
) {
    let Ok((hq_entity, hq_transform, hq_node)) = hq.single() else {
        return;
    };
    let hq_position = hq_transform.translation.xy();
    let mut data: Vec<(Entity, Vec2, f32)> = nodes
        .iter()
        .map(|(entity, transform, node)| (entity, transform.translation.xy(), node.link_range))
        .collect();
    data.push((hq_entity, hq_position, hq_node.link_range));

    let mut powered: HashSet<Entity> = HashSet::new();
    powered.insert(hq_entity);
    let mut links: Vec<(Entity, Entity)> = Vec::new();

    loop {
        let mut best: Option<(Entity, Entity, f32)> = None;
        for &(entity, position, _) in &data {
            if powered.contains(&entity) {
                continue;
            }
            if position.distance(hq_position) > HQ_RADIUS {
                continue;
            }
            for &(other, other_position, other_range) in &data {
                if !powered.contains(&other) || other == entity {
                    continue;
                }
                let distance = position.distance(other_position);
                if distance <= other_range
                    && best
                        .as_ref()
                        .map(|(_, _, best_distance)| distance < *best_distance)
                        .unwrap_or(true)
                {
                    best = Some((other, entity, distance));
                }
            }
        }
        if let Some((parent, child, _)) = best {
            powered.insert(child);
            links.push((parent, child));
        } else {
            break;
        }
    }

    for (entity, _, mut node) in &mut nodes {
        node.powered = powered.contains(&entity);
    }

    links.sort_by_key(|(a, b)| (a.to_bits(), b.to_bits()));
    if links != grid.links {
        grid.links = links;
        grid.changed = true;
    }
}

fn sync_power_links(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut grid: ResMut<PowerGrid>,
    visuals: Query<Entity, With<PowerLinkVisual>>,
    transforms: Query<&Transform>,
) {
    if !grid.changed {
        return;
    }
    grid.changed = false;
    for entity in &visuals {
        commands.entity(entity).despawn();
    }

    for &(from, to) in &grid.links {
        let (Ok(from_transform), Ok(to_transform)) = (transforms.get(from), transforms.get(to)) else {
            continue;
        };
        let start = from_transform.translation.xy();
        let end = to_transform.translation.xy();
        let delta = end - start;
        let length = delta.length();
        if length < 1.0 {
            continue;
        }
        let mid = (start + end) * 0.5;
        let angle = delta.y.atan2(delta.x);
        let rotation = Quat::from_rotation_z(angle);

        commands.spawn((
            Sprite {
                image: assets.tracer.clone(),
                custom_size: Some(Vec2::new(length, 18.0)),
                color: Color::srgba(0.1, 0.7, 0.9, 0.11),
                ..default()
            },
            Transform::from_xyz(mid.x, mid.y, -7.0).with_rotation(rotation),
            PowerLinkVisual,
            DespawnOnExit(GameState::Playing),
        ));
        commands.spawn((
            Sprite {
                image: assets.tracer.clone(),
                custom_size: Some(Vec2::new(length, 3.0)),
                color: Color::srgba(0.35, 0.9, 1.0, 0.5),
                ..default()
            },
            Transform::from_xyz(mid.x, mid.y, -6.5).with_rotation(rotation),
            PowerLinkVisual,
            DespawnOnExit(GameState::Playing),
        ));
    }
}
