use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    pbr::{MaterialPipeline, MaterialPipelineKey},
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
use houtu_scene::{
    GeographicProjection, GeographicTilingScheme, HeightmapTerrainData, IndicesAndEdgesCache,
    TileKey, TilingScheme, WebMercatorProjection, WebMercatorTilingScheme,
};
use thiserror::Error;

use rand::Rng;
mod terrain_mesh;
mod tile_load;
use tile_load::*;

use super::wmts::{WMTSOptions, WMTS};

pub struct ScenePlugin;

impl bevy::app::Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TerrainMeshMaterial>::default())
            .register_type::<TerrainMeshMaterial>()
            // .add_asset::<CustomAsset>()
            // .init_asset_loader::<CustomAssetLoader>()
            .add_startup_system(setup);
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut terrain_materials: ResMut<Assets<TerrainMeshMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // let tiling_scheme = GeographicTilingScheme::default();
    let tiling_scheme = WebMercatorTilingScheme::default();

    let level = 2;
    let num_of_x_tiles = tiling_scheme.get_number_of_x_tiles_at_level(level);
    let num_of_y_tiles = tiling_scheme.get_number_of_y_tiles_at_level(level);
    let wmts = WMTS::new(WMTSOptions {
        url: "http://t0.tianditu.gov.cn/img_c/wmts?tk=b931d6faa76fc3fbe622bddd6522e57b",
        layer: "img",
        format: Some("tiles"),
        tile_matrix_set_id: "c",
        ..Default::default()
    });
    let mut indicesAndEdgesCache = IndicesAndEdgesCache::new();
    let c1 = [Color::WHITE, Color::GREEN];
    for y in 0..num_of_y_tiles {
        for x in 0..num_of_x_tiles {
            let width: u32 = 32;
            let height: u32 = 32;
            let buffer: Vec<f64> = vec![0.; (width * height) as usize];
            let mut heigmapTerrainData = HeightmapTerrainData::new(
                buffer, width, height, None, None, None, None, None, None, None,
            );
            let terrain_mesh = heigmapTerrainData._createMeshSync(
                &tiling_scheme,
                x,
                y,
                level,
                None,
                None,
                &mut indicesAndEdgesCache,
            );
            let mesh = meshes.add(terrain_mesh::TerrainMeshWarp(terrain_mesh).into());
            let mut rng = rand::thread_rng();
            let r: f32 = rng.gen();
            let g: f32 = rng.gen();
            let b: f32 = rng.gen();
            let url = wmts.build_url(&TileKey::new(y, x, level));

            commands.spawn((
                MaterialMeshBundle {
                    mesh: mesh,
                    material: terrain_materials.add(TerrainMeshMaterial {
                        color: Color::rgba(r, g, b, 1.0),
                        // image: asset_server.load("icon.png"),
                        // image: asset_server.load(format!("https://t5.tianditu.gov.cn/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=vec&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={}&TILEROW={}&TILEMATRIX={}&tk=b931d6faa76fc3fbe622bddd6522e57b",x,y,level)),
                        // image: asset_server.load(format!("tile/{}/{}/{}.png", level, y, x,)),
                        image:Some( asset_server.load(format!(
                            "https://maps.omniscale.net/v2/houtu-b8084b0b/style.default/{}/{}/{}.png",
                            level, x, y,
                        ))),
                        // image: None,
                    }),
                    // material: standard_materials.add(Color::rgba(r, g, b, 1.0).into()),
                    ..Default::default()
                },
                TileKey::new(y, x, level),
                // TileState::START,
            ));
        }
    }
}
#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone, Reflect, Resource)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d81"]
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

#[derive(Debug, TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5051"]
pub struct CustomAsset(pub Image);
impl Into<Image> for CustomAsset {
    fn into(self) -> Image {
        self.0
    }
}

// #[derive(Default)]
pub struct CustomAssetLoader {
    supported_compressed_formats: CompressedImageFormats,
}

impl AssetLoader for CustomAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            // use the file extension for the image type
            let ext = load_context.path().extension().unwrap().to_str().unwrap();

            let dyn_img = Image::from_buffer(
                bytes,
                ImageType::Extension(ext),
                self.supported_compressed_formats,
                true,
            )
            .map_err(|err| FileTextureError {
                error: err,
                path: format!("{}", load_context.path().display()),
            })?;

            load_context.set_default_asset(LoadedAsset::new(dyn_img));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["png"]
    }
}

impl FromWorld for CustomAssetLoader {
    fn from_world(world: &mut World) -> Self {
        let supported_compressed_formats = match world.get_resource::<RenderDevice>() {
            Some(render_device) => CompressedImageFormats::from_features(render_device.features()),

            None => CompressedImageFormats::all(),
        };
        Self {
            supported_compressed_formats,
        }
    }
}
/// An error that occurs when loading a texture from a file.
#[derive(Error, Debug)]
pub struct FileTextureError {
    error: TextureError,
    path: String,
}
impl std::fmt::Display for FileTextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Error reading image file {}: {}, this is an error in `bevy_render`.",
            self.path, self.error
        )
    }
}
