use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

use crate::{MainCamera, MapSettings};

pub struct PaintPlugin;

impl Plugin for PaintPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LeftClickEvent>()
            .add_systems(Startup, setup_grid_cells)
            .add_systems(Update, translate_coords)
            .add_systems(
                Update,
                select_grid_cell.run_if(resource_exists::<GridCells>()),
            );
    }
}

#[derive(Event)]
struct LeftClickEvent(Vec2);

struct Tile {
    position: Rect,
    entity: Entity,
}

#[derive(Component)]
struct Cell;

#[derive(Resource)]
struct GridCells(Vec<Tile>);

fn translate_coords(
    mb_input: Res<Input<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut writer: EventWriter<LeftClickEvent>,
) {
    if mb_input.just_pressed(MouseButton::Left) {
        info!("clicked");
        let (cam, coords) = camera.single();
        let win = window.single();

        if let Some(world_position) = win
            .cursor_position()
            .and_then(|cursor| cam.viewport_to_world(coords, cursor))
            .map(|ray| ray.origin.truncate())
        {
            writer.send(LeftClickEvent(world_position));
        }
    }
}

fn select_grid_cell(
    mut commands: Commands,
    mut reader: EventReader<LeftClickEvent>,
    mut grid_cells: ResMut<GridCells>,
    settings: Res<MapSettings>,
    cell_q: Query<&Transform, With<Cell>>,
) {
    for click in reader.read() {
        for (i, cell) in grid_cells.0.iter_mut().enumerate() {
            if cell.position.contains(click.0) {
                if let Some((tile, index)) = &settings.selected_tile {
                    info!("clicked index {} at coords {}", i, click.0);
                    if let Some(e) = commands.get_entity(cell.entity) {
                        e.despawn_recursive();
                    }
                    if let Some(atlas) = settings.atlases.get(&tile.path) {
                        let transform = cell_q.get(cell.entity).unwrap();
                        let new_entity = commands.spawn((
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
                    }
                }
            }
        }
    }
}

fn setup_grid_cells(
    mut commands: Commands,
    settings: Res<MapSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let grid_width = settings.tile_size * settings.grid_width as f32;
    let grid_height = settings.tile_size * settings.grid_height as f32;

    let x_start = 1.0 * (grid_width / 2.0);
    let y_start = 1.0 * (grid_height / 2.0);

    let mut cells = Vec::new();
    for y in (0..settings.grid_height).rev() {
        for x in 0..settings.grid_width {
            let cell_start_x = x as f32 * settings.tile_size - x_start;
            let cell_start_y = y as f32 * settings.tile_size - y_start;
            let cell_start = Vec2::new(cell_start_x, cell_start_y);
            let cell_end = Vec2::new(
                cell_start_x + settings.tile_size,
                cell_start_y + settings.tile_size,
            );
            let place_holder = commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(Mesh::from(shape::Quad::new(Vec2::splat(
                            settings.tile_size - 1.0,
                        ))))
                        .into(),
                    transform: Transform::from_xyz(
                        cell_start_x + (settings.tile_size / 2.),
                        cell_start_y + (settings.tile_size / 2.),
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
    commands.insert_resource(GridCells(cells));
}
