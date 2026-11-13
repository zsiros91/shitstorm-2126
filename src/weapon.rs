use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::arena::Z_BRIGHT;
use crate::assets::GameAssets;
use crate::combat::{Bullet, Lifetime, Velocity};
use crate::fx::{FadeOut, ScreenShake, spawn_casing, spawn_tracer};
use crate::player::{Aim, Player};
use crate::states::{GameSet, GameState, gameplay_active};
use crate::ui::cursor_over_hud;

const MAG_SIZE: u32 = 24;

#[derive(Component)]
pub struct Weapon {
    pub damage: f32,
    pub bullet_radius: f32,
    pub bullet_speed: f32,
    pub fire_interval: f32,
    pub cooldown: f32,
    pub mag: u32,
    pub mag_size: u32,
    pub reload_time: f32,
    pub reload_left: f32,
    pub reloading: bool,
    pub spread: f32,
}

impl Weapon {
    pub fn rifle() -> Self {
        Self {
            damage: 22.0,
            bullet_radius: 4.0,
            bullet_speed: 940.0,
            fire_interval: 0.11,
            cooldown: 0.0,
            mag: MAG_SIZE,
            mag_size: MAG_SIZE,
            reload_time: 1.5,
            reload_left: 0.0,
            reloading: false,
            spread: 0.035,
        }
    }

    /// 0.0..=1.0 progress of the current reload, for the HUD gauge.
    pub fn reload_fraction(&self) -> f32 {
        if self.reloading && self.reload_time > 0.0 {
            1.0 - (self.reload_left / self.reload_time).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (player_reload, player_fire)
                .chain()
                .in_set(GameSet::Combat)
                // spawn rounds before this frame's swept collision test, or
                // they take a full movement step un-checked and tunnel past
                // targets at point-blank range
                .before(crate::combat::bullet_collision)
                .run_if(gameplay_active),
        );
    }
}

fn player_reload(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut Weapon, With<Player>>,
) {
    let Ok(mut weapon) = player.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    if weapon.reloading {
        weapon.reload_left -= dt;
        if weapon.reload_left <= 0.0 {
            weapon.reloading = false;
            weapon.reload_left = 0.0;
            weapon.mag = weapon.mag_size;
        }
        return;
    }

    let wants_reload = keys.just_pressed(KeyCode::KeyR) || weapon.mag == 0;
    if wants_reload && weapon.mag < weapon.mag_size {
        weapon.reloading = true;
        weapon.reload_left = weapon.reload_time;
    }
}

fn player_fire(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    assets: Res<GameAssets>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    mut shake: ResMut<ScreenShake>,
    mut player: Query<(&Transform, &Aim, &mut Weapon), With<Player>>,
) {
    let Ok((transform, aim, mut weapon)) = player.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    weapon.cooldown = (weapon.cooldown - dt).max(0.0);
    if weapon.reloading || weapon.cooldown > 0.0 {
        return;
    }
    if !mouse.pressed(MouseButton::Left) || cursor_over_hud(&window) {
        return;
    }
    if weapon.mag == 0 {
        return;
    }

    weapon.cooldown = weapon.fire_interval;
    weapon.mag -= 1;

    let spread = (fastrand::f32() - 0.5) * 2.0 * weapon.spread;
    let angle = aim.angle + spread;
    let direction = Vec2::from_angle(angle);

    let origin = transform.translation.xy();
    let muzzle = origin + direction * 14.0;

    // the projectile
    commands.spawn((
        Sprite {
            image: assets.bullet.clone(),
            custom_size: Some(Vec2::new(12.0, 5.0)),
            color: Color::srgb(1.0, 0.94, 0.72),
            ..default()
        },
        Transform::from_xyz(muzzle.x, muzzle.y, Z_BRIGHT)
            .with_rotation(Quat::from_rotation_z(angle)),
        Velocity(direction * weapon.bullet_speed),
        Bullet {
            damage: weapon.damage,
            radius: weapon.bullet_radius,
            friendly: true,
        },
        Lifetime(1.4),
        DespawnOnExit(GameState::Playing),
    ));

    spawn_tracer(
        &mut commands,
        &mut images,
        &assets,
        muzzle,
        muzzle + direction * 46.0,
        Color::srgba(1.0, 0.9, 0.6, 0.7),
        2.0,
        0.05,
    );

    // muzzle flash sprite
    commands.spawn((
        Sprite {
            image: assets.muzzle_flash.clone(),
            custom_size: Some(Vec2::splat(26.0)),
            color: Color::srgb(1.0, 0.86, 0.5),
            ..default()
        },
        Transform::from_xyz(muzzle.x, muzzle.y, Z_BRIGHT + 1.0)
            .with_rotation(Quat::from_rotation_z(angle)),
        FadeOut {
            timer: 0.05,
            duration: 0.05,
        },
        DespawnOnExit(GameState::Playing),
    ));
    // ejected casing
    let eject = Vec2::from_angle(angle + std::f32::consts::FRAC_PI_2);
    let speed = 40.0 + fastrand::f32() * 45.0;
    spawn_casing(
        &mut commands,
        &mut images,
        assets.casing.clone(),
        origin,
        fastrand::f32() * std::f32::consts::TAU,
        eject * speed,
        0.5,
    );

    shake.add(0.035);
}
