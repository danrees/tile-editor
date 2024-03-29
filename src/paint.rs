use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::{menus::ResizeEvent, MainCamera, MapSettings};

pub struct PaintPlugin;

impl Plugin for PaintPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LeftClickEvent>()
            .add_event::<RightClickEvent>()
            .init_resource::<GridState>()
            .add_systems(Startup, setup_features_menu)
            .add_systems(Update, setup_grid_cells)
            .add_systems(Update, translate_coords)
            .add_systems(
                Update,
                (select_grid_cell, open_features_ui).run_if(resource_exists::<GridCells>()),
            );
    }
}

#[derive(Event)]
struct LeftClickEvent(Vec2);

#[derive(Event)]
struct RightClickEvent(Vec2);

struct Tile {
    position: Rect,
    entity: Entity,
}

#[derive(Component)]
struct TileFeatures {
    collisions: bool,
}

#[derive(Component)]
struct Grid;

#[derive(Component)]
struct Cell;

#[derive(Component)]
struct SelectedCell;

#[derive(Resource)]
struct GridCells(Vec<Tile>);

impl GridCells {
    fn find_cell(&self, pos: Vec2) -> Option<&Tile> {
        self.0.iter().find(|cell| cell.position.contains(pos))
    }
}

fn translate_coords(
    mb_input: Res<Input<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut writer: EventWriter<LeftClickEvent>,
    mut r_writer: EventWriter<RightClickEvent>,
) {
    for btn in mb_input.get_just_pressed() {
        let (cam, coords) = camera.single();
        let win = window.single();

        if let Some(world_position) = win
            .cursor_position()
            .and_then(|cursor| cam.viewport_to_world(coords, cursor))
            .map(|ray| ray.origin.truncate())
        {
            match btn {
                MouseButton::Left => {
                    info!("left clicked");
                    writer.send(LeftClickEvent(world_position));
                }
                MouseButton::Right => {
                    info!("right clicked");
                    r_writer.send(RightClickEvent(world_position));
                }
                _ => info!("unsupported btn"),
            }
        }
    }
}

#[derive(Resource, Default)]
struct GridState {
    selected_cell: Option<Tile>,
    open: bool,
}

fn setup_features_menu(mut feature_state: ResMut<GridState>, mut contexts: EguiContexts) {
    let window = egui::Window::new("features").open(&mut feature_state.open);

    window.show(contexts.ctx_mut(), |ui| {
        ui.label("Features");
    });
}

fn open_features_ui(
    // mut econtexts: EguiContexts,
    mut grid_cells: ResMut<GridCells>,
    mut reader: EventReader<RightClickEvent>,
    mut feature_state: ResMut<GridState>,
    cell_q: Query<Option<&TileFeatures>, With<Cell>>,
) {
    for click in reader.read() {
        if let Some(cell) = grid_cells.find_cell(click.0) {
            if let Ok(e) = cell_q.get(cell.entity) {
                feature_state.open = true;
            }
        }
    }
}

fn select_grid_cell(
    mut commands: Commands,
    mut reader: EventReader<LeftClickEvent>,
    mut grid_cells: ResMut<GridCells>,
    settings: Res<MapSettings>,
    cell_q: Query<&Transform, With<Cell>>,
    grid_q: Query<Entity, With<Grid>>,
    selected_q: Query<Entity, With<SelectedCell>>,
) {
    for click in reader.read() {
        for (i, cell) in grid_cells.0.iter_mut().enumerate() {
            if cell.position.contains(click.0) {
                if let Some((tile, index)) = &settings.paint_tile {
                    info!("clicked index {} at coords {}", i, click.0);
                    if let Some(e) = commands.get_entity(cell.entity) {
                        e.despawn_recursive();
                    }
                    let e = grid_q.single();
                    if let Some(atlas) = settings.atlases.get(&tile.path) {
                        let transform = cell_q.get(cell.entity).unwrap();
                        commands.get_entity(e).unwrap().with_children(|parent| {
                            let new_entity = parent.spawn((
                                SpriteSheetBundle {
                                    sprite: TextureAtlasSprite {
                                        index: *index,
                                        custom_size: Some(Vec2::splat(settings.tile_size)),
                                        ..default()
                                    },
                                    texture_atlas: atlas.clone(),
                                    transform: transform.clone(),
                                    ..default()
                                },
                                Cell,
                            ));

                            cell.entity = new_entity.id();
                        });
                    }
                }
            }
        }
    }
}

fn setup_grid_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut resize_events: EventReader<ResizeEvent>,
    query: Query<Entity, With<Grid>>,
) {
    for event in resize_events.read() {
        let grid_width = event.tile_size * event.grid_width as f32;
        let grid_height = event.tile_size * event.grid_height as f32;

        let x_start = 1.0 * (grid_width / 2.0);
        let y_start = 1.0 * (grid_height / 2.0);

        let mut cells = Vec::new();
        if let Ok(grid) = query.get_single() {
            commands.entity(grid).despawn_recursive();
        }
        commands
            .spawn((Grid, SpatialBundle { ..default() }))
            .with_children(|commands| {
                for y in (0..event.grid_height).rev() {
                    for x in 0..event.grid_width {
                        let cell_start_x = x as f32 * event.tile_size - x_start;
                        let cell_start_y = y as f32 * event.tile_size - y_start;
                        let cell_start = Vec2::new(cell_start_x, cell_start_y);
                        let cell_end = Vec2::new(
                            cell_start_x + event.tile_size,
                            cell_start_y + event.tile_size,
                        );
                        let place_holder = commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: meshes
                                    .add(Mesh::from(shape::Quad::new(Vec2::splat(
                                        event.tile_size - 1.0,
                                    ))))
                                    .into(),
                                transform: Transform::from_xyz(
                                    cell_start_x + (event.tile_size / 2.),
                                    cell_start_y + (event.tile_size / 2.),
                                    0.,
                                ),
                                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                                ..default()
                            },
                            Cell,
                        ));

                        let tile = Tile {
                            position: Rect::from_corners(cell_start, cell_end),
                            entity: place_holder.id(),
                        };
                        cells.push(tile);
                    }
                }
            });
        commands.insert_resource(GridCells(cells));
    }
}
