use std::collections::HashMap;

use bevy::{asset::AssetLoader, prelude::*};
use futures_lite::AsyncReadExt;
use serde::Deserialize;
use thiserror::Error;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TileDefinition>()
            .init_asset_loader::<TileDefinitionLoader>();
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct AtlasDefinition {
    pub tile_size: Vec2,
    pub columns: usize,
    pub rows: usize,
    pub padding: Option<Vec2>,
    pub offsest: Option<Vec2>,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct Tile {
    pub name: String,
    pub path: String,
    pub atlas_definition: Option<AtlasDefinition>,
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct TileDefinition {
    pub tiles: Vec<Tile>,
}

#[derive(Default)]
struct TileDefinitionLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DefinitionFileError {
    #[error("Could not load: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse data file: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for TileDefinitionLoader {
    type Asset = TileDefinition;

    type Settings = ();

    type Error = DefinitionFileError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let definition = ron::de::from_bytes::<TileDefinition>(&bytes)?;

            Ok(definition)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
