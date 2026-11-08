use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GameAssets {
    pub player_legs: [Handle<Image>; 2],
    pub player_torso: Handle<Image>,
    pub enemy_zergling: Handle<Image>,
    pub enemy_hydralisk: Handle<Image>,
    pub enemy_ultralisk: Handle<Image>,

    pub bullet_gauss: Handle<Image>,
    pub bullet_missile: Handle<Image>,
    pub muzzle_flash: Handle<Image>,
    pub tracer: Handle<Image>,
    pub beam: Handle<Image>,
    pub explosions: [Handle<Image>; 6],
    pub smoke: Handle<Image>,
    pub spark: Handle<Image>,
    pub ring: Handle<Image>,
    pub selection_ring: Handle<Image>,
    pub shadow: Handle<Image>,
    pub scorch: Handle<Image>,
    pub creep_decal: Handle<Image>,

    pub tiles_metal: [Handle<Image>; 3],
    pub tile_grate: Handle<Image>,
    pub tile_rust: Handle<Image>,
    pub tiles_rock: [Handle<Image>; 2],
    pub tiles_creep: [Handle<Image>; 2],
    pub base_pad: Handle<Image>,

    pub hq: Handle<Image>,
    pub gauss_base: Handle<Image>,
    pub gauss_head: Handle<Image>,
    pub missile_base: Handle<Image>,
    pub missile_head: Handle<Image>,
    pub arc_base: Handle<Image>,
    pub arc_head: Handle<Image>,
    pub relay: Handle<Image>,

    pub crate_metal: Handle<Image>,
    pub crate_wood: Handle<Image>,
    pub barrel_green: Handle<Image>,
    pub barrel_red: Handle<Image>,
    pub pipe: Handle<Image>,
    pub sandbag: Handle<Image>,
    pub rock: Handle<Image>,

    pub ui_panel: Handle<Image>,
    pub ui_button: Handle<Image>,
    pub ui_button_hover: Handle<Image>,
    pub ui_minimap: Handle<Image>,
    pub icon_mineral: Handle<Image>,
    pub icon_gas: Handle<Image>,
    pub icon_gauss: Handle<Image>,
    pub icon_missile: Handle<Image>,
    pub icon_arc: Handle<Image>,
    pub icon_relay: Handle<Image>,
    pub icon_repair: Handle<Image>,
    pub portrait: Handle<Image>,

    pub font: Handle<Font>,
    pub font_bold: Handle<Font>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let load = |path: &'static str| assets.load::<Image>(path);
        Self {
            player_legs: [
                load("sprites/goliath_legs_1.png"),
                load("sprites/goliath_legs_2.png"),
            ],
            player_torso: load("sprites/goliath_torso.png"),
            enemy_zergling: load("sprites/enemy_zergling.png"),
            enemy_hydralisk: load("sprites/enemy_hydralisk.png"),
            enemy_ultralisk: load("sprites/enemy_ultralisk.png"),

            bullet_gauss: load("sprites/bullet_gauss.png"),
            bullet_missile: load("sprites/bullet_missile.png"),
            muzzle_flash: load("sprites/muzzle_flash.png"),
            tracer: load("sprites/tracer.png"),
            beam: load("sprites/beam.png"),
            explosions: [
                load("sprites/explosion_1.png"),
                load("sprites/explosion_2.png"),
                load("sprites/explosion_3.png"),
                load("sprites/explosion_4.png"),
                load("sprites/explosion_5.png"),
                load("sprites/explosion_6.png"),
            ],
            smoke: load("sprites/smoke.png"),
            spark: load("sprites/spark.png"),
            ring: load("sprites/ring_cyan.png"),
            selection_ring: load("sprites/selection_ring.png"),
            shadow: load("sprites/shadow.png"),
            scorch: load("sprites/decal_scorch.png"),
            creep_decal: load("sprites/decal_creep.png"),

            tiles_metal: [
                load("sprites/tile_metal_1.png"),
                load("sprites/tile_metal_2.png"),
                load("sprites/tile_metal_3.png"),
            ],
            tile_grate: load("sprites/tile_grate.png"),
            tile_rust: load("sprites/tile_rust_1.png"),
            tiles_rock: [
                load("sprites/tile_rock_1.png"),
                load("sprites/tile_rock_2.png"),
            ],
            tiles_creep: [
                load("sprites/tile_creep_1.png"),
                load("sprites/tile_creep_2.png"),
            ],
            base_pad: load("sprites/base_pad.png"),

            hq: load("sprites/hq_command.png"),
            gauss_base: load("sprites/tower_gauss_base.png"),
            gauss_head: load("sprites/tower_gauss_head.png"),
            missile_base: load("sprites/tower_missile_base.png"),
            missile_head: load("sprites/tower_missile_head.png"),
            arc_base: load("sprites/tower_arc_base.png"),
            arc_head: load("sprites/tower_arc_head.png"),
            relay: load("sprites/relay.png"),

            crate_metal: load("sprites/crate_metal.png"),
            crate_wood: load("sprites/crate_wood.png"),
            barrel_green: load("sprites/barrel_green.png"),
            barrel_red: load("sprites/barrel_red.png"),
            pipe: load("sprites/pipe.png"),
            sandbag: load("sprites/sandbag.png"),
            rock: load("sprites/rock.png"),

            ui_panel: load("sprites/ui_panel.png"),
            ui_button: load("sprites/ui_button.png"),
            ui_button_hover: load("sprites/ui_button_hover.png"),
            ui_minimap: load("sprites/ui_minimap.png"),
            icon_mineral: load("sprites/icon_mineral.png"),
            icon_gas: load("sprites/icon_gas.png"),
            icon_gauss: load("sprites/icon_gauss.png"),
            icon_missile: load("sprites/icon_missile.png"),
            icon_arc: load("sprites/icon_arc.png"),
            icon_relay: load("sprites/icon_relay.png"),
            icon_repair: load("sprites/icon_repair.png"),
            portrait: load("sprites/portrait_goliath.png"),

            font: assets.load::<Font>("fonts/ChakraPetch-Regular.ttf"),
            font_bold: assets.load::<Font>("fonts/ChakraPetch-Bold.ttf"),
        }
    }
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>();
    }
}
