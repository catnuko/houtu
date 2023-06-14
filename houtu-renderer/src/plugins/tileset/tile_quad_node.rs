use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Hash)]
pub struct TileQuadNode {}

#[derive(Component, Clone, Copy, Debug, Hash)]
pub struct TileToRender;

#[derive(Component, Clone, Copy, Debug, Hash)]
pub struct TileLoadHigh;

#[derive(Component, Clone, Copy, Debug, Hash)]
pub struct TileLoadMedium;

#[derive(Component, Clone, Copy, Debug, Hash)]
pub struct TileLoadLow;
