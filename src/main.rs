use std::collections::HashMap;

use assets::{AssetPlugin, Tile, TileDefinition};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use paint::PaintPlugin;

mod assets;
mod paint;
// mod loader;

#[derive(States, Default, Debug, Clone, Hash, PartialEq, Eq)]
pub enum AppState {
    #[default]
    ConfiguringMap,
    Painting,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin)
        .add_plugins(AssetPlugin)
        .add_plugins(PaintPlugin)
        .add_state::<AppState>()
        .add_event::<FolderEvent>()
        .add_event::<ResizeEvent>()
        .init_resource::<MapSettings>()
        .init_resource::<TileAtlases>()
        // .register_asset_source("", )
        .add_systems(Startup, setup)
        // .add_systems(Update, draw_grid)
        // .add_systems(OnEnter(AppState::ConfiguringMap), configure_grid)
        .add_systems(
            Update,
            (selected_tile, example_ui).run_if(resource_exists::<TilesData>()),
        )
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
    grid_width: u32,
    grid_height: u32,
    selected_tile: Option<(Tile, usize)>,
    atlases: HashMap<String, Handle<TextureAtlas>>,
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
            atlases: HashMap::new(),
            // tile_folder: None,
            // tiles_list: Vec::new(),
        }
    }
}

#[derive(Event)]
struct FolderEvent(String);

#[derive(Event)]
pub struct ResizeEvent {
    tile_size: f32,
    grid_width: u32,
    grid_height: u32,
}

impl From<GridSettings> for ResizeEvent {
    fn from(value: GridSettings) -> Self {
        Self {
            tile_size: value.tile_size,
            grid_width: value.grid_width,
            grid_height: value.grid_height,
        }
    }
}

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

#[derive(Debug, Clone)]
struct GridSettings {
    tile_size: f32,
    grid_width: u32,
    grid_height: u32,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            tile_size: 64.,
            grid_width: 6,
            grid_height: 4,
        }
    }
}

fn example_ui(
    //mut commands: Commands,
    mut contexts: EguiContexts,
    mut state: ResMut<MapSettings>,
    // asset_server: Res<AssetServer>,
    // tile_atlases: Res<TileAtlases>,
    // atlases: Res<Assets<TextureAtlas>>,
    tile_handle: Res<TilesData>,
    tile_assets: Res<Assets<TileDefinition>>,
    mut settings: Local<GridSettings>,
    mut resize_events: EventWriter<ResizeEvent>,
) {
    let ui_window = egui::Window::new("main");
    ui_window.show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut settings.tile_size, 0.0..=100.0).text("Tile Size"));
        ui.add(egui::Slider::new(&mut settings.grid_width, 0..=50).text("Grid Width"));
        ui.add(egui::Slider::new(&mut settings.grid_height, 0..=50).text("Grid Height"));

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
            if vui.button("Apply").clicked() {
                // state.tile_size = settings.tile_size;
                // state.grid_width = settings.grid_width;
                // state.grid_height = settings.grid_height;
                resize_events.send(ResizeEvent::from((*settings).clone()));
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

#[derive(Component)]
struct ConfigMenu;

fn configure_grid(mut commands: Commands, settings: Res<MapSettings>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    // justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            ConfigMenu,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Grid Setup",
                TextStyle {
                    font_size: 40.,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        })
        .with_children(|parent| {
            spawn_widget(parent, "Grid Width");
        });
}

fn spawn_widget(builder: &mut ChildBuilder, text: &str) {
    builder
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size: 30.,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        })
        .with_children(|builder| {
            spawn_button_with_text(builder, "+");
            spawn_button_with_text(builder, "-");
        });
}

fn spawn_button_with_text(builder: &mut ChildBuilder, label: &str) {
    builder
        .spawn(NodeBundle {
            style: Style { ..default() },
            ..default()
        })
        .with_children(|builder| {
            builder
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(40.),
                        height: Val::Px(40.),
                        border: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                    border_color: BorderColor(Color::WHITE),
                    ..default()
                })
                .with_children(|builder| {
                    builder.spawn(TextBundle::from_section(
                        label,
                        TextStyle {
                            font_size: 20.,
                            color: Color::RED,
                            ..default()
                        },
                    ));
                });
        });
}

fn selected_tile(mut commands: Commands, state: Res<MapSettings>) {
    if let Some((tile, index)) = &state.selected_tile {
        if let Some(atlas) = state.atlases.get(&tile.path) {
            display_selected_tile(&mut commands, atlas.clone(), *index);
        }
    }
}

fn display_selected_tile(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    atlas_index: usize,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(AtlasImageBundle {
                style: Style {
                    width: Val::Px(64.),
                    height: Val::Px(64.),
                    ..default()
                },
                texture_atlas,
                texture_atlas_image: UiTextureAtlasImage {
                    index: atlas_index,
                    ..default()
                },
                ..default()
            });
        });
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
