use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GameAssets {
    // terrain ----------------------------------------------------------
    pub tiles_sand: [Handle<Image>; 3],
    pub tiles_dirt: [Handle<Image>; 3],
    pub tiles_gravel: [Handle<Image>; 2],
    pub tiles_drygrass: [Handle<Image>; 2],
    pub tiles_asphalt: [Handle<Image>; 3],
    pub road_double_h: Handle<Image>,
    pub road_double_v: Handle<Image>,
    pub road_cross_h: Handle<Image>,
    pub road_cross_v: Handle<Image>,
    pub tiles_sidewalk: [Handle<Image>; 2],
    pub tiles_checker: [Handle<Image>; 2],
    pub tiles_wood: [Handle<Image>; 2],
    pub tiles_concrete: [Handle<Image>; 2],

    // walls ------------------------------------------------------------
    pub walls_brick: [Handle<Image>; 2],
    pub walls_brick_broken: [Handle<Image>; 2],
    pub walls_concrete: [Handle<Image>; 2],
    pub wall_concrete_broken: Handle<Image>,
    pub walls_metal: [Handle<Image>; 2],

    // decals -----------------------------------------------------------
    pub decal_leaf: [Handle<Image>; 2],
    pub decal_grass: [Handle<Image>; 2],
    pub decal_oil: Handle<Image>,
    pub decal_blood: [Handle<Image>; 2],
    pub decal_rubble: [Handle<Image>; 2],
    pub decal_crack: [Handle<Image>; 2],
    pub decal_scorch: Handle<Image>,

    // props ------------------------------------------------------------
    pub counter: Handle<Image>,
    pub dead_trees: [Handle<Image>; 2],
    pub debris_piles: [Handle<Image>; 2],

    // characters -------------------------------------------------------
    pub player_body: [[Handle<Image>; 2]; 4],
    pub player_gun: Handle<Image>,
    pub raider_body: [[Handle<Image>; 2]; 4],
    pub raider_gun: Handle<Image>,
    pub infected_body: [[Handle<Image>; 2]; 4],

    // fx ---------------------------------------------------------------
    pub bullet: Handle<Image>,
    pub tracer: Handle<Image>,
    pub muzzle_flash: Handle<Image>,
    pub casing: Handle<Image>,
    pub dust_puff: Handle<Image>,
    pub spark: Handle<Image>,
    pub shadow: Handle<Image>,
    pub shadow_small: Handle<Image>,
    pub blood_splat: Handle<Image>,
    pub crosshair: Handle<Image>,
    pub hit_marker: Handle<Image>,

    // hud --------------------------------------------------------------
    pub ui_panel: Handle<Image>,
    pub ui_slot: Handle<Image>,
    pub ui_slot_hover: Handle<Image>,
    pub icon_health: Handle<Image>,
    pub icon_stamina: Handle<Image>,
    pub icon_ammo: Handle<Image>,
    pub portrait: Handle<Image>,

    pub font: Handle<Font>,
    pub font_bold: Handle<Font>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let load = |path: &'static str| assets.load::<Image>(path);
        Self {
            tiles_sand: [
                load("sprites/tile_sand_1.png"),
                load("sprites/tile_sand_2.png"),
                load("sprites/tile_sand_3.png"),
            ],
            tiles_dirt: [
                load("sprites/tile_dirt_1.png"),
                load("sprites/tile_dirt_2.png"),
                load("sprites/tile_dirt_3.png"),
            ],
            tiles_gravel: [
                load("sprites/tile_gravel_1.png"),
                load("sprites/tile_gravel_2.png"),
            ],
            tiles_drygrass: [
                load("sprites/tile_drygrass_1.png"),
                load("sprites/tile_drygrass_2.png"),
            ],
            tiles_asphalt: [
                load("sprites/tile_asphalt_1.png"),
                load("sprites/tile_asphalt_2.png"),
                load("sprites/tile_asphalt_3.png"),
            ],
            road_double_h: load("sprites/tile_road_double_h.png"),
            road_double_v: load("sprites/tile_road_double_v.png"),
            road_cross_h: load("sprites/tile_road_cross_h.png"),
            road_cross_v: load("sprites/tile_road_cross_v.png"),
            tiles_sidewalk: [
                load("sprites/tile_sidewalk_1.png"),
                load("sprites/tile_sidewalk_2.png"),
            ],
            tiles_checker: [
                load("sprites/tile_checker_1.png"),
                load("sprites/tile_checker_2.png"),
            ],
            tiles_wood: [
                load("sprites/tile_wood_1.png"),
                load("sprites/tile_wood_2.png"),
            ],
            tiles_concrete: [
                load("sprites/tile_concrete_1.png"),
                load("sprites/tile_concrete_2.png"),
            ],

            walls_brick: [
                load("sprites/wall_brick_1.png"),
                load("sprites/wall_brick_2.png"),
            ],
            walls_brick_broken: [
                load("sprites/wall_brick_broken_1.png"),
                load("sprites/wall_brick_broken_2.png"),
            ],
            walls_concrete: [
                load("sprites/wall_concrete_1.png"),
                load("sprites/wall_concrete_2.png"),
            ],
            wall_concrete_broken: load("sprites/wall_concrete_broken.png"),
            walls_metal: [
                load("sprites/wall_metal_1.png"),
                load("sprites/wall_metal_2.png"),
            ],

            decal_leaf: [
                load("sprites/decal_leaf_1.png"),
                load("sprites/decal_leaf_2.png"),
            ],
            decal_grass: [
                load("sprites/decal_grass_1.png"),
                load("sprites/decal_grass_2.png"),
            ],
            decal_oil: load("sprites/decal_oil.png"),
            decal_blood: [
                load("sprites/decal_blood_1.png"),
                load("sprites/decal_blood_2.png"),
            ],
            decal_rubble: [
                load("sprites/decal_rubble_1.png"),
                load("sprites/decal_rubble_2.png"),
            ],
            decal_crack: [
                load("sprites/decal_crack_1.png"),
                load("sprites/decal_crack_2.png"),
            ],
            decal_scorch: load("sprites/decal_scorch.png"),

            counter: load("sprites/counter.png"),
            dead_trees: [
                load("sprites/dead_tree_1.png"),
                load("sprites/dead_tree_2.png"),
            ],
            debris_piles: [
                load("sprites/debris_pile_1.png"),
                load("sprites/debris_pile_2.png"),
            ],

            player_body: [
                [
                    load("sprites/player_body_0_0.png"),
                    load("sprites/player_body_0_1.png"),
                ],
                [
                    load("sprites/player_body_1_0.png"),
                    load("sprites/player_body_1_1.png"),
                ],
                [
                    load("sprites/player_body_2_0.png"),
                    load("sprites/player_body_2_1.png"),
                ],
                [
                    load("sprites/player_body_3_0.png"),
                    load("sprites/player_body_3_1.png"),
                ],
            ],
            player_gun: load("sprites/player_gun.png"),
            raider_body: [
                [
                    load("sprites/raider_body_0_0.png"),
                    load("sprites/raider_body_0_1.png"),
                ],
                [
                    load("sprites/raider_body_1_0.png"),
                    load("sprites/raider_body_1_1.png"),
                ],
                [
                    load("sprites/raider_body_2_0.png"),
                    load("sprites/raider_body_2_1.png"),
                ],
                [
                    load("sprites/raider_body_3_0.png"),
                    load("sprites/raider_body_3_1.png"),
                ],
            ],
            raider_gun: load("sprites/raider_gun.png"),
            infected_body: [
                [
                    load("sprites/infected_body_0_0.png"),
                    load("sprites/infected_body_0_1.png"),
                ],
                [
                    load("sprites/infected_body_1_0.png"),
                    load("sprites/infected_body_1_1.png"),
                ],
                [
                    load("sprites/infected_body_2_0.png"),
                    load("sprites/infected_body_2_1.png"),
                ],
                [
                    load("sprites/infected_body_3_0.png"),
                    load("sprites/infected_body_3_1.png"),
                ],
            ],

            bullet: load("sprites/bullet.png"),
            tracer: load("sprites/tracer.png"),
            muzzle_flash: load("sprites/muzzle_flash.png"),
            casing: load("sprites/casing.png"),
            dust_puff: load("sprites/dust_puff.png"),
            spark: load("sprites/spark.png"),
            shadow: load("sprites/shadow.png"),
            shadow_small: load("sprites/shadow_small.png"),
            blood_splat: load("sprites/blood_splat.png"),
            crosshair: load("sprites/crosshair.png"),
            hit_marker: load("sprites/hit_marker.png"),

            ui_panel: load("sprites/ui_panel.png"),
            ui_slot: load("sprites/ui_slot.png"),
            ui_slot_hover: load("sprites/ui_slot_hover.png"),
            icon_health: load("sprites/icon_health.png"),
            icon_stamina: load("sprites/icon_stamina.png"),
            icon_ammo: load("sprites/icon_ammo.png"),
            portrait: load("sprites/portrait.png"),

            font: assets.load::<Font>("fonts/Silkscreen-Regular.ttf"),
            font_bold: assets.load::<Font>("fonts/Silkscreen-Bold.ttf"),
        }
    }
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>();
    }
}
