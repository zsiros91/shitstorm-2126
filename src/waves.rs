use bevy::prelude::*;

use crate::arena::{random_edge_point, Arena};
use crate::assets::GameAssets;
use crate::combat::EnemyKilled;
use crate::economy::Economy;
use crate::enemy::{spawn_enemy, Enemy, EnemyKind};
use crate::player::{Player, Repair};
use crate::states::{gameplay_active, GameSet, GameState};

const MAX_ALIVE: usize = 120;

#[derive(Clone, Copy, Debug)]
pub enum WavePhase {
    Countdown(f32),
    Spawning,
    Clearing,
}

#[derive(Resource)]
pub struct WaveManager {
    pub wave: u32,
    pub phase: WavePhase,
    pub remaining_to_spawn: u32,
    pub spawn_timer: f32,
    pub total_in_wave: u32,
}

#[derive(Resource, Default)]
pub struct Score {
    pub value: u32,
}

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_systems(
                OnEnter(GameState::Playing),
                (start_waves, reset_score).chain(),
            )
            .add_systems(
                Update,
                (track_score, wave_director)
                    .chain()
                    .in_set(GameSet::Ai)
                    .run_if(gameplay_active),
            );
    }
}

fn start_waves(mut commands: Commands) {
    commands.insert_resource(WaveManager {
        wave: 1,
        phase: WavePhase::Countdown(5.0),
        remaining_to_spawn: 0,
        spawn_timer: 0.0,
        total_in_wave: 0,
    });
}

fn reset_score(mut score: ResMut<Score>) {
    score.value = 0;
}

fn track_score(
    mut kills: MessageReader<EnemyKilled>,
    mut score: ResMut<Score>,
    mut economy: ResMut<Economy>,
) {
    for kill in kills.read() {
        score.value += kill.score;
        economy.minerals += kill.minerals;
        economy.gas += kill.gas;
    }
}

fn wave_director(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    arena: Res<Arena>,
    mut waves: ResMut<WaveManager>,
    mut economy: ResMut<Economy>,
    enemies: Query<&Enemy>,
    mut player: Query<&mut Repair, With<Player>>,
) {
    let dt = time.delta_secs();
    let alive = enemies.iter().count();
    let was_clearing = matches!(waves.phase, WavePhase::Clearing);

    match waves.phase {
        WavePhase::Countdown(timer) => {
            let timer = timer - dt;
            if timer <= 0.0 {
                waves.remaining_to_spawn = 10 + waves.wave * 5;
                waves.total_in_wave = waves.remaining_to_spawn;
                waves.spawn_timer = 0.0;
                waves.phase = WavePhase::Spawning;
            } else {
                waves.phase = WavePhase::Countdown(timer);
            }
        }
        WavePhase::Spawning => {
            waves.spawn_timer -= dt;
            let mut projected = alive;

            while waves.spawn_timer <= 0.0
                && waves.remaining_to_spawn > 0
                && projected < MAX_ALIVE
            {
                let position = random_edge_point(&arena, 60.0);
                let kind = pick_kind(waves.wave);
                spawn_enemy(&mut commands, &assets, kind, position, waves.wave);
                waves.remaining_to_spawn -= 1;
                projected += 1;
                let interval = (0.85 - 0.045 * waves.wave as f32).max(0.18);
                waves.spawn_timer += interval;
            }
        }
        WavePhase::Clearing => {}
    }

    if matches!(waves.phase, WavePhase::Spawning) && waves.remaining_to_spawn == 0 {
        waves.phase = WavePhase::Clearing;
    }

    if was_clearing && alive == 0 {
        if let Ok(mut repair) = player.single_mut() {
            repair.charges = repair.max_charges;
        }
        economy.minerals += 45 + waves.wave * 8;
        economy.gas += 8 + waves.wave / 2;
        waves.wave += 1;
        waves.phase = WavePhase::Countdown(6.0);
    }
}

fn pick_kind(wave: u32) -> EnemyKind {
    let roll = fastrand::f32();
    if wave >= 4 && roll < (0.07 + wave as f32 * 0.006).min(0.18) {
        EnemyKind::Ultralisk
    } else if wave >= 2 && roll < 0.38 {
        EnemyKind::Hydralisk
    } else {
        EnemyKind::Zergling
    }
}
