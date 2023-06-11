use amethyst::{
    assets::{AssetStorage, Completion, Handle, Loader, PrefabData, ProgressCounter},
    core::{
        ecs::{
            Component, Entity, HashMapStorage, Join, Read, ReadExpect, ReadStorage, System, Write,
            WriteStorage,
        },
        transform::GlobalTransform,
    },
    renderer::{
        build_mesh_with_combo, ActiveCamera, Attributes, Camera, ComboMeshCreator, Encoder,
        Factory, FilterMethod, Mesh, MeshCreator, MeshData, PngFormat, PosTex, Position, Rgba,
        SamplerInfo, Separate, Shape, ShapeUpload, SurfaceType, TexCoord, Texture, TextureMetadata,
        VertexFormat, WrapMode,
    },
    Error,
};

use crate::component::{ActiveTerrain, Terrain};

#[derive(Default)]
pub struct TerrainSystem {
    progress: Option<ProgressCounter>,
}

impl<'a> System<'a> for TerrainSystem {
    type SystemData = (
        Option<Read<'a, ActiveTerrain>>,
        WriteStorage<'a, Terrain>,
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
    );

    fn run(
        &mut self,
        (active, mut terrains, loader, mesh_storage, texture_storage): Self::SystemData,
    ) {

    }
}
