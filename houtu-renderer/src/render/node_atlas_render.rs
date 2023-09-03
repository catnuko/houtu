use super::node_atlas::AtlasAttachment;
use bevy::{
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
        Extract,
    },
    utils::HashMap,
};

#[derive(Component)]
pub struct GpuNodeAtlas {
    pub attachments: Vec<AtlasAttachment>,
    pub array_texture: Texture,
    pub texture_size: UVec3,
    pub quantization_bits12: bool,
    pub has_web_mercator_t: bool,
}

impl GpuNodeAtlas {
    pub fn create_texture_view(&self) -> TextureView {
        self.array_texture.create_view(&TextureViewDescriptor {
            label: Some("array_texture_view"),
            dimension: Some(TextureViewDimension::D2Array),
            array_layer_count: Some(self.texture_size.z),
            base_array_layer: 0,
            ..Default::default()
        })
    }
    pub fn update(
        &mut self,
        command_encoder: &mut CommandEncoder,
        queue: &mut RenderQueue,
        images: &RenderAssets<Image>,
    ) {
        for (index, attachment) in self.attachments.iter().enumerate() {
            let index = index as u32;
            if let Some(atlas_attachment) = images.get(&attachment.handle) {
                command_encoder.copy_texture_to_texture(
                    ImageCopyTexture {
                        texture: &atlas_attachment.texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    ImageCopyTexture {
                        texture: &self.array_texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: index,
                        },
                        aspect: TextureAspect::All,
                    },
                    Extent3d {
                        width: atlas_attachment.texture.width(),
                        height: atlas_attachment.texture.height(),
                        depth_or_array_layers: 1,
                    },
                );
                // info!("copy over")
            } else {
                error!("Something went wrong, attachment is not available!")
            }
        }
    }
}
