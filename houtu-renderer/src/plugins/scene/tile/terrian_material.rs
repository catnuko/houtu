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
#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone, Reflect, Resource)]
#[uuid = "886f4558-1621-492a-856e-ea1dbc9902d9"]
#[reflect(Default, Debug)]
pub struct TerrainMeshMaterial {
    #[uniform(0)]
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub image: Option<Handle<Image>>,
}
impl Material for TerrainMeshMaterial {
    fn fragment_shader() -> ShaderRef {
        "terrain_material.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "terrain_material.wgsl".into()
    }
    // fn specialize(
    //     _pipeline: &MaterialPipeline<Self>,
    //     descriptor: &mut RenderPipelineDescriptor,
    //     _layout: &MeshVertexBufferLayout,
    //     _key: MaterialPipelineKey<Self>,
    // ) -> Result<(), SpecializedMeshPipelineError> {
    //     // This is the important part to tell bevy to render this material as a line between vertices
    //     descriptor.primitive.polygon_mode = PolygonMode::Line;
    //     Ok(())
    // }
}
