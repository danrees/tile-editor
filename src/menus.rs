use bevy::prelude::*;
use bevy_egui::{egui, render_systems::EguiTextureId, EguiContexts};

use crate::{
    assets::{AtlasDefinition, TileDefinition},
    grid::BrushState,
    AppSystemSets, MapSettings, TilesData,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResizeEvent>()
            .init_resource::<MenuAtlasRegistry>()
            .add_systems(Startup, setup.after(AppSystemSets::LoadingStuff))
            .add_systems(Update, (example_ui, handle_paint_tile_click))
            .add_systems(
                Update,
                (selected_tile_set).run_if(resource_exists_and_changed::<MapSettings>()),
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
#[derive(Event)]
pub struct ResizeEvent {
    pub tile_size: f32,
    pub grid_width: u32,
    pub grid_height: u32,
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
struct MenuAtlasRegistry(Vec<MenuImageAtlasItem>);

struct MenuImageAtlasItem {
    id: egui::TextureId,
    loc: Rect,
}

fn setup(
    mut contexts: EguiContexts,
    state: Res<MapSettings>,
    atlases: Res<Assets<TextureAtlas>>,
    mut registry: ResMut<MenuAtlasRegistry>,
) {
    for (_path, handle) in &state.atlases {
        if let Some(h) = atlases.get(handle) {
            let id = contexts.add_image(h.texture.clone_weak());
            for texture in &h.textures {
                let item = MenuImageAtlasItem {
                    id,
                    loc: texture.clone(),
                };
                registry.0.push(item);
            }
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
    asset_server: Res<AssetServer>,
    atlases: Res<Assets<TextureAtlas>>,
    brush_state: Res<State<BrushState>>,
    mut next_brush_state: ResMut<NextState<BrushState>>,
) {
    let grid_settings = egui::SidePanel::left("grid-settings");
    //let ui_window = egui::Window::new("main");
    grid_settings.show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut settings.tile_size, 0.0..=100.0).text("Tile Size"));
        ui.add(egui::Slider::new(&mut settings.grid_width, 0..=50).text("Grid Width"));
        ui.add(egui::Slider::new(&mut settings.grid_height, 0..=50).text("Grid Height"));

        // ui.label(state.tile_folder.clone().unwrap_or(String::from("None")));

        ui.heading("Tiles");
        ui.vertical(|vui| {
            if let Some(tile_a) = tile_assets.get(&tile_handle.0) {
                for tile in &tile_a.tiles {
                    if vui
                        .selectable_label(
                            if let Some(t) = &state.paint_tile {
                                t.name == tile.name
                            } else {
                                false
                            },
                            tile.name.clone(),
                        )
                        .clicked()
                    {
                        state.paint_tile = Some((*tile).clone());
                    }
                }
            }
            if vui.button("Apply").clicked() {
                // state.tile_size = settings.tile_size;
                // state.grid_width = settings.grid_width;
                // state.grid_height = settings.grid_height;
                resize_events.send(ResizeEvent::from((*settings).clone()));
            }
        });
    });

    let brush_settings = egui::SidePanel::right("brush-settings");
    brush_settings.show(contexts.ctx_mut(), |ui| {
        ui.heading("Brushes");
        if ui
            .selectable_label(brush_state.eq(&BrushState::Single), "Brush")
            .clicked()
        {
            next_brush_state.set(BrushState::Single);
        }
        if ui
            .selectable_label(brush_state.eq(&BrushState::Fill), "Fill")
            .clicked()
        {
            next_brush_state.set(BrushState::Fill);
        }
    });
}

fn selected_tile_set(mut commands: Commands, state: Res<MapSettings>) {
    if let Some(tile) = &state.paint_tile {
        if let Some(atlas) = state.atlases.get(&tile.path) {
            display_selected_tile_set(&mut commands, atlas.clone(), &tile.atlas_definition);
        }
    }
}

#[derive(Component)]
struct Paintable;

fn display_selected_tile_set(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    atlas_def: &Option<AtlasDefinition>,
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
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    if let Some(ad) = atlas_def {
                        for i in 0..(ad.rows * ad.columns) {
                            parent.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(64.),
                                        height: Val::Px(64.),
                                        ..default()
                                    },
                                    texture_atlas: texture_atlas.clone(),
                                    texture_atlas_image: UiTextureAtlasImage {
                                        index: i,
                                        ..default()
                                    },
                                    ..default()
                                },
                                Interaction::default(),
                                Paintable,
                            ));
                        }
                    }
                });
        });
}

fn handle_paint_tile_click(
    interactions_q: Query<&Interaction, (Changed<Interaction>, With<Paintable>)>,
) {
    for interaction in &interactions_q {
        match *interaction {
            Interaction::Pressed => {
                info!("Pressed");
            }
            Interaction::Hovered => {
                info!("Hovered");
            }
            Interaction::None => {
                info!("None");
            }
        }
    }
}
