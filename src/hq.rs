use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::{Collider, StaticCollider};
use crate::combat::{Health, HitFlash};
use crate::healthbar::spawn_health_bar;
use crate::power::{HQ_LINK_RANGE, HQ_RADIUS, PowerNode, PowerSource};
use crate::states::GameState;

#[derive(Component)]
pub struct Hq;

pub struct HqPlugin;

impl Plugin for HqPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_hq);
    }
}

fn spawn_hq(mut commands: Commands, assets: Res<GameAssets>) {
    let position = Vec2::ZERO;

    let hq = commands
        .spawn((
            Sprite {
                image: assets.hq.clone(),
                custom_size: Some(Vec2::splat(240.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 6.0),
            Hq,
            Health::new(2500.0),
            HitFlash::default(),
            Collider::circle(86.0),
            StaticCollider(Collider::circle(92.0)),
            PowerNode {
                link_range: HQ_LINK_RANGE,
                powered: true,
                source: true,
            },
            PowerSource,
            DespawnOnExit(GameState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    image: assets.shadow.clone(),
                    custom_size: Some(Vec2::splat(300.0)),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.7),
                    ..default()
                },
                Transform::from_xyz(16.0, -18.0, -4.0),
            ));
        })
        .id();

    spawn_health_bar(&mut commands, hq, 170.0, 132.0, false);
}

pub const HQ_COMMAND_RADIUS: f32 = HQ_RADIUS;
