use std::collections::HashMap;

use assets::{AssetPlugin, Tile, TileDefinition};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use grid::GridPlugin;
use menus::MenuPlugin;
// use paint::PaintPlugin;

mod assets;
mod grid;
mod loader;
mod menus;
// mod paint;

#[derive(States, Default, Debug, Clone, Hash, PartialEq, Eq)]
pub enum AppState {
    #[default]
    Loading,
    Painting,
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum AppSystemSets {
    LoadingStuff,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin)
        .add_plugins(AssetPlugin)
        // .add_plugins(PaintPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(GridPlugin)
        .add_state::<AppState>()
        .init_resource::<MapSettings>()
        .init_resource::<TileAtlases>()
        // .register_asset_source("", )
        .add_systems(Startup, setup.in_set(AppSystemSets::LoadingStuff))
        // .add_systems(Update, draw_grid)
        // .add_systems(OnEnter(AppState::ConfiguringMap), configure_grid)
        // .add_systems(
        //     Update,
        //     (selected_tile).run_if(resource_exists::<TilesData>()),
        // )
        .add_systems(Update, load_tiles.run_if(resource_changed::<TilesData>()))
        // .add_systems(Startup, load_tiles.run_if(resource_exists::<TilesData>()))
        .run();
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// struct Tile {
//     path: String,
// }

#[derive(Resource, Clone)]
struct MapSettings {
    tile_size: f32,
    paint_tile: Option<Tile>,
    atlases: HashMap<String, Handle<TextureAtlas>>,
    // tile_folder: Option<String>,
    // tiles_list: Vec<Tile>,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            tile_size: 64.0,
            paint_tile: None,
            atlases: HashMap::new(),
            // tile_folder: None,
            // tiles_list: Vec::new(),
        }
    }
}

#[derive(Event)]
struct FolderEvent(String);

#[derive(Resource, Default)]
struct TilesData(Handle<TileDefinition>);

#[derive(Resource, Default)]
struct TileAtlases(Vec<(String, Handle<TextureAtlas>)>);

#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
    let tiles_data = asset_server.load("data/tiles.ron");
    commands.insert_resource(TilesData(tiles_data));
}

fn load_tiles(
    tile_handle: Res<TilesData>,
    tile_assets: Res<Assets<TileDefinition>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut state: ResMut<MapSettings>,
    // mut atlases: ResMut<TileAtlases>,
) {
    if let Some(tiles) = tile_assets.get(&tile_handle.0) {
        for tile in &tiles.tiles {
            if let Some(atlas) = &tile.atlas_definition {
                let texture_handle = asset_server.load(&tile.path);
                let tx_atlas = TextureAtlas::from_grid(
                    texture_handle,
                    atlas.tile_size,
                    atlas.columns,
                    atlas.rows,
                    atlas.padding,
                    atlas.offsest,
                );

                let atlas_handle = texture_atlases.add(tx_atlas);
                // texture_atlases.add(tx_atlas);
                state.atlases.insert(tile.path.clone(), atlas_handle);
            }
        }
    }
}

#[derive(Component)]
struct ConfigMenu;
