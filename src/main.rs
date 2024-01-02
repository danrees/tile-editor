use assets::{AssetPlugin, Tile, TileDefinition};
use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_file::FileDialog;

mod assets;
// mod loader;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin)
        .add_plugins(AssetPlugin)
        .add_event::<FolderEvent>()
        .init_resource::<MapSettings>()
        .init_resource::<TileAtlases>()
        // .register_asset_source("", )
        .add_systems(Startup, (setup, tile_load))
        .add_systems(Update, draw_grid)
        .add_systems(Update, (example_ui).run_if(resource_exists::<TilesData>()))
        // .add_systems(Startup, load_tiles.run_if(resource_exists::<TilesData>()))
        .run();
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// struct Tile {
//     path: String,
// }

#[derive(Resource)]
struct MapSettings {
    tile_size: f32,
    grid_width: u32,
    grid_height: u32,
    selected_tile: Option<(Tile, usize)>,
    // tile_folder: Option<String>,
    // tiles_list: Vec<Tile>,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            tile_size: 64.0,
            grid_width: 6,
            grid_height: 4,
            selected_tile: None,
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let tiles_data = asset_server.load("data/tiles.ron");
    commands.insert_resource(TilesData(tiles_data));
}

// fn load_tiles(
//     tile_handle: Res<TilesData>,
//     tile_assets: Res<Assets<TileDefinition>>,
//     asset_server: Res<AssetServer>,
//     mut texture_atlases: ResMut<Assets<TextureAtlas>>,
//     // mut atlases: ResMut<TileAtlases>,
// ) {
//     if let Some(tiles) = tile_assets.get(&tile_handle.0) {
//         for tile in &tiles.tiles {
//             if let Some(atlas) = &tile.atlas_defintion {
//                 let texture_handle = asset_server.load(&tile.path);
//                 let tx_atlas = TextureAtlas::from_grid(
//                     texture_handle,
//                     atlas.tile_size,
//                     atlas.columns,
//                     atlas.rows,
//                     atlas.padding,
//                     atlas.offsest,
//                 );
//                 let atlas_handle = texture_atlases.add(tx_atlas);
//                 // TODO: what do I do with the atlas handle?
//                 if let Some(td) = tile_assets.get(tile_handle.0) {
//                     atlases.0.push((tile.name.clone(), atlas_handle));
//                 }
//             }
//         }
//     }
// }

fn draw_grid(mut gizmos: Gizmos, settings: Res<MapSettings>) {
    let grid_width = settings.tile_size * settings.grid_width as f32;
    let grid_height = settings.tile_size * settings.grid_height as f32;

    let tiles_wide = settings.grid_width;
    let tiles_high = settings.grid_height;

    let x_start = -1.0 * (grid_width / 2.0);
    let y_start = -1.0 * (grid_height / 2.0);

    for y in 0..(tiles_high + 1) {
        let y_pos = y as f32 * settings.tile_size;
        gizmos.line_2d(
            Vec2::new(x_start, y_start + y_pos),
            Vec2::new(x_start + grid_width, y_start + y_pos),
            Color::RED,
        );
    }

    for x in 0..(tiles_wide + 1) {
        let x_pos = x as f32 * settings.tile_size;
        gizmos.line_2d(
            Vec2::new(x_start + x_pos, y_start),
            Vec2::new(x_start + x_pos, y_start + grid_height),
            Color::RED,
        );
    }
}

struct DialogState {
    dialog: Option<FileDialog>,
}

impl Default for DialogState {
    fn default() -> Self {
        Self { dialog: None }
    }
}

fn example_ui(
    //mut commands: Commands,
    mut contexts: EguiContexts,
    mut state: ResMut<MapSettings>,
    // asset_server: Res<AssetServer>,
    mut dialog_res: Local<DialogState>,
    mut folder_event: EventWriter<FolderEvent>,
    // tile_atlases: Res<TileAtlases>,
    // atlases: Res<Assets<TextureAtlas>>,
    tile_handle: Res<TilesData>,
    tile_assets: Res<Assets<TileDefinition>>,
) {
    let ui_window = egui::Window::new("main");
    ui_window.show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut state.tile_size, 0.0..=100.0).text("Tile Size"));
        ui.add(egui::Slider::new(&mut state.grid_width, 0..=50).text("Grid Width"));
        ui.add(egui::Slider::new(&mut state.grid_height, 0..=50).text("Grid Height"));

        // ui.label(state.tile_folder.clone().unwrap_or(String::from("None")));

        ui.heading("Tiles");
        ui.vertical(|vui| {
            if let Some(tile_a) = tile_assets.get(&tile_handle.0) {
                for tile in &tile_a.tiles {
                    vui.collapsing(&tile.name.clone(), |cui| {
                        if let Some(atlas_def) = &tile.atlas_definition {
                            for i in 0..(atlas_def.columns * atlas_def.rows) {
                                if cui
                                    .selectable_label(
                                        if let Some((t, ind)) = &state.selected_tile {
                                            t.name == tile.name && *ind == i
                                        } else {
                                            false
                                        },
                                        format!("{}-{}", &tile.name.clone(), i),
                                    )
                                    .clicked()
                                {
                                    state.selected_tile = Some(((*tile).clone(), i));
                                }
                                //cui.label(format!("{}-{}", &tile.name.clone(), i));
                            }
                        }
                    });
                }
            }
        });
        // if (ui.button("Folder")).clicked() {
        //     let cwd = std::env::current_dir().unwrap();
        //     let mut dialog = FileDialog::select_folder(Some(cwd));
        //     dialog.open();
        //     dialog_res.dialog = Some(dialog);
        // }

        // if let Some(dialog) = &mut dialog_res.dialog {
        //     if dialog.show(ui.ctx()).selected() {
        //         if let Some(path) = dialog.path() {
        //             state.tile_folder = Some(path.to_string_lossy().to_string());
        //             folder_event.send(FolderEvent(path.to_string_lossy().to_string()));
        //         }
        //     }
        // }
    });
}

fn tile_load(asset_server: Res<AssetServer>, mut map_settings: ResMut<MapSettings>) {
    let tiles_folder = asset_server.load_folder("tiles");
}

// fn tile_list(
//     mut folder_events: eventreader<folderevent>,
//     map_settings: resmut<mapsettings>,
//     mut images: resmut<assets<image>>,
//     mut contexts: eguicontexts,
// ) {
//     for event in folder_events.read() {
//         let name = event.0.clone();

//         contexts.add_image()
//     }
// }
