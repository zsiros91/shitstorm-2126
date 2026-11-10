use bevy::prelude::*;

use crate::combat::Health;
use crate::states::GameState;

#[derive(Component)]
pub struct HealthBar {
    pub owner: Entity,
    pub width: f32,
    pub offset_y: f32,
    pub fill: bool,
    pub hostile: bool,
}

pub fn spawn_health_bar(
    commands: &mut Commands,
    owner: Entity,
    width: f32,
    offset_y: f32,
    hostile: bool,
) {
    commands.spawn((
        Sprite::from_color(
            Color::srgba(0.03, 0.04, 0.05, 0.85),
            Vec2::new(width + 4.0, 6.0),
        ),
        Transform::from_xyz(0.0, 0.0, 151.0),
        HealthBar {
            owner,
            width,
            offset_y,
            fill: false,
            hostile,
        },
        DespawnOnExit(GameState::Playing),
    ));

    let color = if hostile {
        Color::srgb(0.9, 0.18, 0.12)
    } else {
        Color::srgb(0.25, 0.9, 0.4)
    };
    commands.spawn((
        Sprite::from_color(color, Vec2::new(width, 3.0)),
        Transform::from_xyz(0.0, 0.0, 152.0),
        HealthBar {
            owner,
            width,
            offset_y,
            fill: true,
            hostile,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_health_bars);
    }
}

fn update_health_bars(
    mut commands: Commands,
    owners: Query<(&Transform, &Health), Without<HealthBar>>,
    mut bars: Query<(Entity, &HealthBar, &mut Transform, &mut Sprite), With<HealthBar>>,
) {
    for (entity, bar, mut transform, mut sprite) in &mut bars {
        let Ok((owner_transform, health)) = owners.get(bar.owner) else {
            commands.entity(entity).despawn();
            continue;
        };

        let fraction = health.fraction();
        let position = owner_transform.translation.xy();
        transform.translation.x = position.x;
        transform.translation.y = position.y + bar.offset_y;
        transform.translation.z = if bar.fill { 152.0 } else { 151.0 };

        if bar.fill {
            let width = bar.width * fraction.max(0.001);
            sprite.custom_size = Some(Vec2::new(width, 3.0));
            transform.translation.x = position.x - bar.width * 0.5 + width * 0.5;
            sprite.color = if bar.hostile {
                if fraction > 0.55 {
                    Color::srgb(0.94, 0.24, 0.17)
                } else if fraction > 0.25 {
                    Color::srgb(0.96, 0.5, 0.16)
                } else {
                    Color::srgb(0.66, 0.1, 0.08)
                }
            } else if fraction > 0.55 {
                Color::srgb(0.25, 0.9, 0.4)
            } else if fraction > 0.25 {
                Color::srgb(0.95, 0.72, 0.2)
            } else {
                Color::srgb(0.95, 0.2, 0.14)
            };
        } else {
            sprite.custom_size = Some(Vec2::new(bar.width + 4.0, 6.0));
            transform.translation.x = position.x;
        }
    }
}
