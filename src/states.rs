use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
pub struct Paused(pub bool);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Ai,
    Movement,
    Combat,
    Effects,
    Ui,
}

pub fn gameplay_active(state: Res<State<GameState>>, paused: Res<Paused>) -> bool {
    *state.get() == GameState::Playing && !paused.0
}
