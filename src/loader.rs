use bevy::prelude::*;

use crate::{assets::TileDefinition, TilesData};

pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {}
}

fn setup(
    mut commands: Commands,
    tile_data: Res<TilesData>,
    tile_data_asset: Res<Assets<TileDefinition>>,
) {
}
