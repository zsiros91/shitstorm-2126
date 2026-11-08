use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GameAssets {
    pub player_hull: Handle<Image>,
    pub player_turret: Handle<Image>,
    pub enemy_walker: Handle<Image>,
    pub enemy_runner: Handle<Image>,
    pub enemy_brute: Handle<Image>,
    pub bullet: Handle<Image>,
    pub muzzle_flash: Handle<Image>,
    pub tiles: [Handle<Image>; 4],
    pub crate_wood: Handle<Image>,
    pub crate_metal: Handle<Image>,
    pub barrel_green: Handle<Image>,
    pub barrel_red: Handle<Image>,
    pub tree: Handle<Image>,
    pub sandbag: Handle<Image>,
    pub explosions: [Handle<Image>; 5],
    pub font: Handle<Font>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            player_hull: assets.load("sprites/player_hull.png"),
            player_turret: assets.load("sprites/player_turret.png"),
            enemy_walker: assets.load("sprites/enemy_walker.png"),
            enemy_runner: assets.load("sprites/enemy_runner.png"),
            enemy_brute: assets.load("sprites/enemy_brute.png"),
            bullet: assets.load("sprites/bullet.png"),
            muzzle_flash: assets.load("sprites/muzzle_flash.png"),
            tiles: [
                assets.load("sprites/tile_grass_1.png"),
                assets.load("sprites/tile_grass_2.png"),
                assets.load("sprites/tile_sand_1.png"),
                assets.load("sprites/tile_sand_2.png"),
            ],
            crate_wood: assets.load("sprites/crate.png"),
            crate_metal: assets.load("sprites/crate_metal.png"),
            barrel_green: assets.load("sprites/barrel_green.png"),
            barrel_red: assets.load("sprites/barrel_red.png"),
            tree: assets.load("sprites/tree.png"),
            sandbag: assets.load("sprites/sandbag.png"),
            explosions: [
                assets.load("sprites/explosion_1.png"),
                assets.load("sprites/explosion_2.png"),
                assets.load("sprites/explosion_3.png"),
                assets.load("sprites/explosion_4.png"),
                assets.load("sprites/explosion_5.png"),
            ],
            font: assets.load("fonts/KenneyPixel.ttf"),
        }
    }
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>();
    }
}
