use bevy::{
    asset::LoadState,
    math::{DMat4, DVec3},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        main_graph::node::CAMERA_DRIVER,
        render_asset::RenderAssets,
        render_graph::RenderGraph,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        view::NoFrustumCulling,
        RenderApp, RenderSet,
    },
    utils::HashMap,
};
use houtu_scene::TerrainQuantization;

use crate::{
    camera::GlobeCamera,
    quadtree::{
        imagery_layer_storage::ImageryLayerStorage, imagery_storage::ImageryStorage,
        quadtree_tile::QuadtreeTile, tile_key::TileKey,
    },
};

use super::{terrain_plugin::TerrainAttachment, wrap_terrain_mesh::WrapTerrainMesh, TileRendered};

#[derive(Component, Clone)]
pub struct TerrainConfig {
    pub minimum_height: f32,
    pub maximum_height: f32,
    pub quantization_bits12: bool,
    pub has_web_mercator_t: bool,
    pub center_3d: Vec3,
    pub scale_and_bias: Mat4,
    pub mvp: Mat4,
    pub attachments: Vec<TerrainAttachment>,
    pub tile_key: TileKey,
}
impl TerrainConfig {
    pub fn count(&self) -> u32 {
        return self.attachments.len() as u32;
    }
    pub fn get_array_texture_size(&self) -> UVec3 {
        let mut max_width: u32 = 0;
        let mut height: u32 = 0;
        for attachment in &self.attachments {
            if max_width < attachment.width {
                max_width = attachment.width
            }
            height += attachment.height;
        }
        return UVec3 {
            x: max_width,
            y: height,
            z: self.attachments.len() as u32,
        };
    }
    pub fn create(&self, device: &RenderDevice, queue: &RenderQueue) -> Texture {
        let first_attachment = self.attachments.get(0).expect("expect first attachment");
        let blue = [0, 0, 255, 255];
        let data = vec![
            blue;
            (first_attachment.width as usize)
                * (first_attachment.height as usize)
                * self.attachments.len()
        ];

        let texture = device.create_texture_with_data(
            &queue,
            &TextureDescriptor {
                label: Some(
                    &(format!(
                        "terrain_atlas_attachment_{}_{}_{}",
                        self.tile_key.x, self.tile_key.y, self.tile_key.level
                    )),
                ),
                size: Extent3d {
                    width: first_attachment.width,
                    height: first_attachment.height,
                    depth_or_array_layers: self.attachments.len() as u32,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            bytemuck::cast_slice(&data),
        );
        return texture;
    }
}

#[derive(Bundle)]
pub struct TerrainBundle {
    pub mesh: Handle<Mesh>,
    pub config: TerrainConfig,
    pub tile_rendered: TileRendered,
    pub spatial_bundle: SpatialBundle,
    pub no_frustum_culling: NoFrustumCulling,
}
impl TerrainBundle {
    pub fn from_quadtree_tile(
        tile: &mut QuadtreeTile,
        globe_camera: &mut GlobeCamera,
        imagery_storage: &ImageryStorage,
        imagery_layer_storage: &ImageryLayerStorage,
        meshes: &mut Assets<Mesh>,
        images: &mut Assets<Image>,
    ) -> Self {
        let data = tile.data.get_cloned_terrain_data();
        let surface_tile = data.lock().unwrap();
        let terrain_mesh = surface_tile.get_mesh().unwrap();
        let mvp = globe_camera.get_mvp();

        let mut attachments = vec![];
        let wrap_terrain_mesh = WrapTerrainMesh(terrain_mesh);
        let mesh: Mesh = wrap_terrain_mesh.into();
        for tile_imagery in tile.data.imagery.iter_mut() {
            let ready_imagery = tile_imagery.ready_imagery.as_ref().unwrap().clone();
            let imagery = ready_imagery.0.read().unwrap();
            let imagery_layer = imagery_layer_storage.get(imagery.get_layer_id()).unwrap();
            if imagery_layer.alpha == 0.0 {
                continue;
            }
            let texture = match imagery.texture.as_ref() {
                Some(v) => v.clone(),
                None => panic!("readyImagery is not actually ready!"),
            };
            // let image = images
            //     .get_mut(&texture)
            //     .expect("expect gpu image of imagery");
            // image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
            if tile_imagery.texture_translation_and_scale.is_none() {
                tile_imagery.texture_translation_and_scale =
                    Some(imagery_layer.calculate_texture_translation_and_scale(
                        tile.rectangle.clone(),
                        tile_imagery,
                        imagery.rectangle.clone(),
                    ))
            }
            let attachment = TerrainAttachment {
                handle: texture,
                translation_and_scale: tile_imagery
                    .texture_translation_and_scale
                    .unwrap()
                    .as_vec4(),
                coordinate_rectangle: tile_imagery.texture_coordinate_rectangle.unwrap().as_vec4(),
                web_mercator_t: if tile_imagery.use_web_mercator_t {
                    1.0
                } else {
                    0.0
                },
                alpha: imagery_layer.alpha as f32,
                night_alpha: imagery_layer.night_alpha as f32,
                day_alpha: imagery_layer.day_alpha as f32,
                brightness: imagery_layer.brightness as f32,
                contrast: imagery_layer.contrast as f32,
                hue: imagery_layer.hue as f32,
                saturation: imagery_layer.saturation as f32,
                one_over_gamma: imagery_layer.gamma as f32,
                width: imagery_layer.imagery_provider.get_tile_width(),
                height: imagery_layer.imagery_provider.get_tile_height(),
            };
            attachments.push(attachment);
        }
        let terrain_config = TerrainConfig {
            scale_and_bias: terrain_mesh.encoding.matrix.as_mat4(),
            center_3d: terrain_mesh.center.as_vec3(),
            mvp: mvp.as_mat4(),
            minimum_height: terrain_mesh.encoding.minimum_height as f32,
            maximum_height: terrain_mesh.encoding.maximum_height as f32,
            quantization_bits12: terrain_mesh.encoding.quantization == TerrainQuantization::BITS12,
            has_web_mercator_t: terrain_mesh.encoding.has_web_mercator_t,
            attachments: attachments,
            tile_key: tile.key.clone(),
        };
        return Self {
            config: terrain_config,
            mesh: meshes.add(mesh),
            tile_rendered: TileRendered(tile.key),
            spatial_bundle: SpatialBundle::INHERITED_IDENTITY,
            no_frustum_culling: NoFrustumCulling,
        };
    }
}
