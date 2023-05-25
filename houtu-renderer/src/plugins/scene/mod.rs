use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    pbr::{wireframe::Wireframe, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::{erased_serde::__private::serde::Deserialize, TypeUuid},
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
        renderer::RenderDevice,
        texture::{CompressedImageFormats, ImageTextureLoader, ImageType, TextureError},
    },
    utils::BoxedFuture,
};

mod tile;
use houtu_scene::Rectangle;
use tile::*;

pub struct ScenePlugin;

impl bevy::app::Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(tile::TilePlugin);
    }
}
