use bevy::prelude::*;

use crate::states::GameState;

#[derive(Resource, Debug)]
pub struct Economy {
    pub minerals: u32,
    pub gas: u32,
}

impl Default for Economy {
    fn default() -> Self {
        Self {
            minerals: 160,
            gas: 35,
        }
    }
}

impl Economy {
    pub fn can_afford(&self, minerals: u32, gas: u32) -> bool {
        self.minerals >= minerals && self.gas >= gas
    }

    pub fn spend(&mut self, minerals: u32, gas: u32) -> bool {
        if !self.can_afford(minerals, gas) {
            return false;
        }
        self.minerals -= minerals;
        self.gas -= gas;
        true
    }
}

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Economy>()
            .add_systems(OnEnter(GameState::Playing), reset_economy);
    }
}

fn reset_economy(mut economy: ResMut<Economy>) {
    *economy = Economy::default();
}
