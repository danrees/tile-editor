use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};

use crate::{assets::TileDefinition, AppState, MainCamera, TilesData};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<ActionState>()
            .add_state::<BrushState>()
            .add_event::<ClickEvent>()
            .add_event::<CellInteractionEvent>()
            .init_resource::<GridSize>()
            .add_systems(OnEnter(AppState::Painting), spawn_grid)
            .add_systems(OnExit(AppState::Painting), despawn_grid)
            .add_systems(
                Update,
                (translate_coords, interact_cell).run_if(in_state(AppState::Painting)),
            )
            .add_systems(Update, paint.run_if(in_state(ActionState::Paint)));
    }
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum ActionState {
    #[default]
    Select,
    Paint,
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum BrushState {
    #[default]
    Single,
    Fill,
}

#[derive(Resource)]
struct GridSize {
    cols: usize,
    rows: usize,
    tile_size: f32,
}

impl Default for GridSize {
    fn default() -> Self {
        Self {
            cols: 6,
            rows: 4,
            tile_size: 64.,
        }
    }
}

#[derive(Bundle)]
struct CellBundle {
    cell: Cell,
    transformation: TransformBundle,
}

#[derive(Component)]
struct Cell;

#[derive(Component)]
struct Grid;

#[derive(Component)]
struct Brush {
    index: usize,
    atlas: Handle<TextureAtlas>,
}

#[derive(Component)]
struct SelectedBrush;

fn spawn_grid(
    mut commands: Commands,
    grid_size: Res<GridSize>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((Grid, SpatialBundle::default()))
        .with_children(|parent| {
            let x_start = (grid_size.tile_size * grid_size.cols as f32) / 2.;
            let y_start = (grid_size.tile_size * grid_size.rows as f32) / 2.;
            for y in 0..grid_size.rows {
                for x in 0..grid_size.cols {
                    parent.spawn((
                        Cell,
                        MaterialMesh2dBundle {
                            mesh: meshes
                                .add(Mesh::from(shape::Quad::new(Vec2::splat(
                                    grid_size.tile_size - 1.0,
                                ))))
                                .into(),
                            transform: Transform::from_translation(Vec3::new(
                                x_start + (x as f32 * grid_size.tile_size),
                                y_start + (y as f32 * grid_size.tile_size),
                                1.0,
                            )),
                            material: materials.add(ColorMaterial::from(Color::PURPLE)),
                            ..default()
                        },
                    ));
                }
            }
        });
}

fn despawn_grid(mut commands: Commands, mut query: Query<Entity, With<Grid>>) {
    let e = query.single_mut();
    commands.get_entity(e).unwrap().despawn_recursive();
}

#[derive(Event)]
enum ClickEvent {
    LeftClick(Vec2),
    RightClick(Vec2),
}

#[derive(Event)]
struct CellInteractionEvent(Entity);

fn get_coords(window: &Window, camera: &Camera, transform: &GlobalTransform) -> Option<Vec2> {
    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(transform, cursor))
        .map(|ray| ray.origin.truncate())
}

fn translate_coords(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    buttons: Res<Input<MouseButton>>,
    mut writer: EventWriter<ClickEvent>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let (cam, coords) = camera.single();
        let win = window.single();

        if let Some(loc) = get_coords(win, cam, coords) {
            writer.send(ClickEvent::LeftClick(loc));
        }
    }
    if buttons.just_pressed(MouseButton::Right) {
        let (cam, coords) = camera.single();
        let win = window.single();

        if let Some(loc) = get_coords(win, cam, coords) {
            writer.send(ClickEvent::RightClick(loc));
        }
    }
}

fn interact_cell(
    mut reader: EventReader<ClickEvent>,
    mut writer: EventWriter<CellInteractionEvent>,
    grid: Query<&Children, With<Grid>>,
    cells: Query<&GlobalTransform, With<Cell>>,
    grid_size: Res<GridSize>,
) {
    for event in reader.read() {
        match event {
            ClickEvent::LeftClick(click) => {
                for g in &grid {
                    for c in g {
                        if let Ok(transform) = cells.get(*c) {
                            if Rect::from_center_size(
                                transform.translation().xy(),
                                Vec2::splat(grid_size.tile_size),
                            )
                            .contains(*click)
                            {
                                writer.send(CellInteractionEvent(c.clone()));
                                break;
                            }
                        }
                    }
                }
            }
            ClickEvent::RightClick(click) => todo!(),
        }
    }
}

fn paint(
    mut commands: Commands,
    mut reader: EventReader<CellInteractionEvent>,
    cell: Query<(Entity, &Parent, &Transform), With<Cell>>,
    brush: Query<&Brush, With<SelectedBrush>>,
    grid_size: Res<GridSize>,
) {
    for event in reader.read() {
        if let Ok((current, grid, transform)) = cell.get(event.0) {
            let current_brush = brush.single();
            // TODO: not sure if this will be inefficient to despawn and recreate if it hasn't changed
            commands.entity(current).despawn_recursive();
            commands.entity(grid.get()).with_children(|parent| {
                parent.spawn((
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            index: current_brush.index,
                            custom_size: Some(Vec2::splat(grid_size.tile_size)),
                            ..default()
                        },
                        texture_atlas: current_brush.atlas.clone_weak(),
                        transform: transform.clone(),
                        ..default()
                    },
                    Cell,
                ));
            });
        }
    }
}
