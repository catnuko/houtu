//! Amethyst terrain rendering using Cardinal Neighbour Quadtrees for Level-of-Detail
//! 
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]
#![warn(dead_code)]

pub use crate::{
    component::{
        Terrain, 
        ActiveTerrain,
    },
    renderpass::{
        DrawTerrain,
        DrawTerrainDesc,
        RenderTerrain,
        TerrainConfig,
        TerrainViewMode,
    },
    prefab::{
        TerrainPrefab,
    }
    // system::{
    //     TerrainSystem,
    // },
};



mod component;
mod renderpass;
mod system;
mod prefab;