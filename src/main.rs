use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_file::FileDialog;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .init_resource::<MapSettings>()
        // .register_asset_source("", )
        .add_systems(Startup, setup)
        .add_systems(Update, example_ui)
        .add_systems(Update, draw_grid)
        .run();
}

#[derive(Resource)]
struct MapSettings {
    tile_size: f32,
    grid_width: u32,
    grid_height: u32,
    tile_folder: Option<String>,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            tile_size: 64.0,
            grid_width: 6,
            grid_height: 4,
            tile_folder: None,
        }
    }
}

#[derive(Resource)]
struct TilesFolder {
    folders: Handle<LoadedFolder>,
}

// const TILE_SIZE: f32 = 64.0;
// const NUM_TILES: u32 = 6;
// const GRID_LENGTH: f32 = NUM_TILES as f32 * 64.0;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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
) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut state.tile_size, 0.0..=100.0).text("Tile Size"));
        ui.add(egui::Slider::new(&mut state.grid_width, 0..=50).text("Grid Width"));
        ui.add(egui::Slider::new(&mut state.grid_height, 0..=50).text("Grid Height"));

        ui.label(state.tile_folder.clone().unwrap_or(String::from("None")));

        if (ui.button("Folder")).clicked() {
            let cwd = std::env::current_dir().unwrap();
            let mut dialog = FileDialog::select_folder(Some(cwd));
            dialog.open();
            dialog_res.dialog = Some(dialog);
        }

        if let Some(dialog) = &mut dialog_res.dialog {
            if dialog.show(ui.ctx()).selected() {
                if let Some(path) = dialog.path() {
                    state.tile_folder = Some(path.to_string_lossy().to_string());
                }
            }
        }
    });
}
