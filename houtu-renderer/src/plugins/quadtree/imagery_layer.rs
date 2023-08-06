

use bevy::{
    core::cast_slice,
    math::{DMat4, DVec4},
    prelude::*,
    render::{
        render_resource::{
            encase, BufferInitDescriptor, BufferUsages, Extent3d, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
    },
    utils::{HashMap, Uuid},
};

use houtu_scene::{
    lerp_f32,
    Matrix4, Rectangle, TilingScheme,
};

use crate::plugins::{
    camera::GlobeCamera,
    quadtree::reproject_texture::{ParamsUniforms, ReprojectTextureTask},
};

use super::{
    imagery::{Imagery, ImageryState, ShareMutImagery},
    imagery_layer_storage::ImageryLayerStorage,
    imagery_provider::ImageryProvider,
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_tile::QuadtreeTile,
    reproject_texture::ReprojectTextureTaskQueue,
    terrain_provider::TerrainProvider,
    tile_imagery::TileImagery,
    tile_key::TileKey,
};

pub struct ImageryLayerOtherState {
    pub minimumTerrainLevel: Option<u32>,
    pub maximumTerrainLevel: Option<u32>,
}

pub struct ImageryLayer {
    pub id: Uuid,
    pub alpha: f64,
    pub night_alpha: f64,
    pub day_alpha: f64,
    pub brightness: f64,
    pub contrast: f64,
    pub hue: f64,
    pub saturation: f64,
    pub gamma: f64,
    pub z_index: u32,
    pub is_base_layer: bool,
    pub ready: bool,
    pub cutout_rectangle: Option<Rectangle>,
    pub color_to_alpha: f64,
    pub color_to_alpha_threshold: f64,
    pub _rectangle: Rectangle,
    // pub skeleton_placeholder: TileImagery,
    pub imagery_cache: HashMap<TileKey, ShareMutImagery>,
    pub imagery_provider: Box<dyn ImageryProvider>,
    pub show: bool,
}
impl ImageryLayer {
    pub fn new(imagery_provider: Box<dyn ImageryProvider>) -> Self {
        Self {
            id: Uuid::new_v4(),
            alpha: 1.0,
            night_alpha: 1.0,
            day_alpha: 1.0,
            brightness: 1.0,
            contrast: 1.0,
            hue: 0.0,
            saturation: 1.0,
            gamma: 1.0,
            z_index: 0,
            is_base_layer: false,
            cutout_rectangle: None,
            color_to_alpha: 1.0,
            color_to_alpha_threshold: 0.004,
            ready: true,
            _rectangle: Rectangle::MAX_VALUE.clone(),
            // skeleton_placeholder: TileImagery::create_placeholder(imagery_layer_id, None),
            imagery_cache: HashMap::new(),
            imagery_provider: imagery_provider,
            show: true,
        }
    }
    fn getLevelWithMaximumTexelSpacing(
        &mut self,
        texel_spacing: f64,
        latitude_closest_to_equator: f64,
    ) -> u32 {
        // PERFORMANCE_IDEA: factor out the stuff that doesn't change.
        let tiling_scheme = self.imagery_provider.get_tiling_scheme();
        let ellipsoid = tiling_scheme.ellipsoid;
        let latitude_factor = if false {
            latitude_closest_to_equator.cos()
        } else {
            1.0
        };
        let tiling_scheme_rectangle = tiling_scheme.rectangle;
        let level_zero_maximum_texel_spacing =
            (ellipsoid.maximum_radius * tiling_scheme_rectangle.compute_width() * latitude_factor)
                / (self.imagery_provider.get_tile_width()
                    * tiling_scheme.get_number_of_x_tiles_at_level(0)) as f64;

        let two_to_the_level_power = level_zero_maximum_texel_spacing / texel_spacing;
        let level = two_to_the_level_power.ln() / 2f64.ln();
        let rounded = level.round() as u32;
        return rounded | 0;
    }

    fn is_base_layer(&self) -> bool {
        return self.is_base_layer;
    }
    pub fn _createTileImagerySkeletons(
        &mut self,
        tile: &mut QuadtreeTile,
        // key: &TileKey,
        terrain_provider: &Box<dyn TerrainProvider>,
    ) -> bool {
        let mut insertion_point = tile.data.imagery.len();
        if !self.ready || !self.imagery_provider.get_ready() {
            let key = TileKey {
                x: 0,
                y: 0,
                level: 0,
            };
            let imagery = self.add_imagery(&key).unwrap();
            tile.data.add(imagery.clone(), None, false);
            return true;
        }

        let use_web_mercator_t = false;

        let imagery_bounds = self
            .imagery_provider
            .get_rectangle()
            .intersection(&self._rectangle)
            .expect("多边形相交没结果");
        let mut intersection_rectangle = tile.rectangle.intersection(&imagery_bounds);

        if intersection_rectangle.is_none() {
            // There is no overlap between this terrain tile and this imagery
            // provider.  Unless this is the base layer, no skeletons need to be created.
            // We stretch texels at the edge of the base layer over the entire globe.
            if !self.is_base_layer {
                return false;
            }

            let base_imagery_rectangle = imagery_bounds;
            let base_terrain_rectangle = &tile.rectangle;
            let mut new_rectangle = Rectangle::default();

            if base_terrain_rectangle.south >= base_imagery_rectangle.north {
                new_rectangle.north = base_imagery_rectangle.north;
                new_rectangle.south = base_imagery_rectangle.north;
            } else if base_terrain_rectangle.north <= base_imagery_rectangle.south {
                new_rectangle.north = base_imagery_rectangle.south;
                new_rectangle.south = base_imagery_rectangle.south;
            } else {
                new_rectangle.south = base_terrain_rectangle
                    .south
                    .max(base_imagery_rectangle.south);
                new_rectangle.north = base_terrain_rectangle
                    .north
                    .min(base_imagery_rectangle.north);
            }

            if base_terrain_rectangle.west >= base_imagery_rectangle.east {
                new_rectangle.west = base_imagery_rectangle.east;
                new_rectangle.east = base_imagery_rectangle.east;
            } else if base_terrain_rectangle.east <= base_imagery_rectangle.west {
                new_rectangle.west = base_imagery_rectangle.west;
                new_rectangle.east = base_imagery_rectangle.west;
            } else {
                new_rectangle.west = base_terrain_rectangle.west.max(base_imagery_rectangle.west);
                new_rectangle.east = base_terrain_rectangle.east.min(base_imagery_rectangle.east);
            }
            intersection_rectangle = Some(new_rectangle)
        }

        let mut latitude_closest_to_equator = 0.0;
        if tile.rectangle.south > 0.0 {
            latitude_closest_to_equator = tile.rectangle.south;
        } else if tile.rectangle.north < 0.0 {
            latitude_closest_to_equator = tile.rectangle.north;
        }

        // Compute the required level in the imagery tiling scheme.
        // The error_ratio should really be imagerySSE / terrainSSE rather than this hard-coded value.
        // But first we need configurable imagery SSE and we need the rendering to be able to handle more
        // images attached to a terrain tile than there are available texture units.  So that's for the future.
        let error_ratio = 1.0;
        let target_geometric_error =
            error_ratio * terrain_provider.get_level_maximum_geometric_error(tile.key.level);
        let mut imagery_level = self
            .getLevelWithMaximumTexelSpacing(target_geometric_error, latitude_closest_to_equator);
        imagery_level = 0.max(imagery_level);
        let maximum_level = self.imagery_provider.get_maximum_level();
        if imagery_level > maximum_level {
            imagery_level = maximum_level;
        }
        let minimum_level = self.imagery_provider.get_minimum_level();
        if imagery_level < minimum_level {
            imagery_level = minimum_level;
        }

        let imagery_tiling_scheme = self.imagery_provider.get_tiling_scheme();
        let mut north_west_tile_coordinates = imagery_tiling_scheme
            .position_to_tile_x_y(&tile.rectangle.north_west(), imagery_level)
            .expect("north_west_tile_coordinates");
        let mut south_east_tile_coordinates = imagery_tiling_scheme
            .position_to_tile_x_y(&tile.rectangle.south_east(), imagery_level)
            .expect("south_east_tile_coordinates");

        // If the southeast corner of the rectangle lies very close to the north or west side
        // of the southeast tile, we don't actually need the southernmost or easternmost
        // tiles.
        // Similarly, if the northwest corner of the rectangle lies very close to the south or east side
        // of the northwest tile, we don't actually need the northernmost or westernmost tiles.

        // We define "very close" as being within 1/512 of the width of the tile.
        let mut very_close_x = tile.rectangle.compute_width() / 512.0;
        let mut very_close_y = tile.rectangle.compute_height() / 512.0;

        let north_west_tile_rectangle = imagery_tiling_scheme.tile_x_y_to_rectange(
            north_west_tile_coordinates.x,
            north_west_tile_coordinates.y,
            imagery_level,
        );
        if north_west_tile_rectangle.south - tile.rectangle.north.abs() < very_close_y
            && north_west_tile_coordinates.y < south_east_tile_coordinates.y
        {
            north_west_tile_coordinates.y += 1;
        }

        if (north_west_tile_rectangle.east - tile.rectangle.west).abs() < very_close_x
            && north_west_tile_coordinates.x < south_east_tile_coordinates.x
        {
            north_west_tile_coordinates.x += 1;
        }

        let south_east_tile_rectangle = imagery_tiling_scheme.tile_x_y_to_rectange(
            south_east_tile_coordinates.x,
            south_east_tile_coordinates.y,
            imagery_level,
        );
        if (south_east_tile_rectangle.north - tile.rectangle.south).abs() < very_close_y
            && south_east_tile_coordinates.y > north_west_tile_coordinates.y
        {
            south_east_tile_coordinates.y -= 1;
        }
        if (south_east_tile_rectangle.west - tile.rectangle.east).abs() < very_close_x
            && south_east_tile_coordinates.x > north_west_tile_coordinates.x
        {
            south_east_tile_coordinates.x -= 1;
        }

        // Create TileImagery instances for each imagery tile overlapping this terrain tile.
        // We need to do all texture coordinate computations in the imagery tile's tiling scheme.

        let terrain_rectangle = tile.rectangle.clone();
        let mut imagery_rectangle = imagery_tiling_scheme.tile_x_y_to_rectange(
            north_west_tile_coordinates.x,
            north_west_tile_coordinates.y,
            imagery_level,
        );
        let mut clipped_imagery_rectangle = imagery_rectangle
            .intersection(&imagery_bounds)
            .expect("clipped_imagery_rectangle");

        // let imageryTileXYToRectangle;
        let mut use_native = false;
        if use_web_mercator_t {
            imagery_tiling_scheme.rectangle_to_native_rectangle(&terrain_rectangle);
            imagery_tiling_scheme.rectangle_to_native_rectangle(&imagery_rectangle);
            imagery_tiling_scheme.rectangle_to_native_rectangle(&clipped_imagery_rectangle);
            imagery_tiling_scheme.rectangle_to_native_rectangle(&imagery_bounds);
            // imageryTileXYToRectangle = imagery_tiling_scheme
            //     .tile_x_y_to_native_rectange
            //     .bind(imagery_tiling_scheme);
            use_native = true;
            very_close_x = terrain_rectangle.compute_width() / 512.0;
            very_close_y = terrain_rectangle.compute_height() / 512.0;
        } else {
            // imageryTileXYToRectangle = imagery_tiling_scheme
            //     .tile_x_y_to_rectange
            //     .bind(imagery_tiling_scheme);
            use_native = false;
        }

        let mut min_u;
        let mut max_u = 0.0;

        let mut min_v = 1.0;
        let mut max_v;

        // If this is the northern-most or western-most tile in the imagery tiling scheme,
        // it may not start at the northern or western edge of the terrain tile.
        // Calculate where it does start.
        if !self.is_base_layer()
            && (clipped_imagery_rectangle.west - terrain_rectangle.west).abs() >= very_close_x
        {
            max_u = (1.0 as f64).min(
                (clipped_imagery_rectangle.west - terrain_rectangle.west)
                    / terrain_rectangle.compute_width(),
            );
        }

        if !self.is_base_layer()
            && (clipped_imagery_rectangle.north - terrain_rectangle.north).abs() >= very_close_y
        {
            min_v = (0.0 as f64).max(
                (clipped_imagery_rectangle.north - terrain_rectangle.south)
                    / terrain_rectangle.compute_height(),
            );
        }

        let initialMinV = min_v;
        for i in north_west_tile_coordinates.x..south_east_tile_coordinates.x {
            min_u = max_u;

            imagery_rectangle = if use_native {
                self.imagery_provider
                    .get_tiling_scheme()
                    .tile_x_y_to_native_rectange(i, north_west_tile_coordinates.y, imagery_level)
            } else {
                self.imagery_provider
                    .get_tiling_scheme()
                    .tile_x_y_to_rectange(i, north_west_tile_coordinates.y, imagery_level)
            };

            let clippedImageryRectangleRes = imagery_rectangle.simpleIntersection(&imagery_bounds);

            if clippedImageryRectangleRes.is_none() {
                continue;
            }
            clipped_imagery_rectangle = clippedImageryRectangleRes.expect("rectangle is some");

            max_u = (1.0 as f64).min(
                (clipped_imagery_rectangle.east - terrain_rectangle.west)
                    / terrain_rectangle.compute_width(),
            );

            // If this is the eastern-most imagery tile mapped to this terrain tile,
            // and there are more imagery tiles to the east of this one, the max_u
            // should be 1.0 to make sure rounding errors don't make the last
            // image fall shy of the edge of the terrain tile.
            if i == south_east_tile_coordinates.x
                && (self.is_base_layer()
                    || (clipped_imagery_rectangle.east - terrain_rectangle.east).abs()
                        < very_close_x)
            {
                max_u = 1.0;
            }

            min_v = initialMinV;
            for j in north_west_tile_coordinates.y..south_east_tile_coordinates.y {
                max_v = min_v;

                imagery_rectangle = if use_native {
                    self.imagery_provider
                        .get_tiling_scheme()
                        .tile_x_y_to_native_rectange(i, j, imagery_level)
                } else {
                    self.imagery_provider
                        .get_tiling_scheme()
                        .tile_x_y_to_rectange(i, j, imagery_level)
                };
                let clippedImageryRectangleRes =
                    imagery_rectangle.simpleIntersection(&imagery_bounds);

                if clippedImageryRectangleRes.is_none() {
                    continue;
                }
                clipped_imagery_rectangle = clippedImageryRectangleRes.expect("rectangle is some");
                min_v = (0.0 as f64).max(
                    (clipped_imagery_rectangle.south - terrain_rectangle.south)
                        / terrain_rectangle.compute_height(),
                );

                // If this is the southern-most imagery tile mapped to this terrain tile,
                // and there are more imagery tiles to the south of this one, the min_v
                // should be 0.0 to make sure rounding errors don't make the last
                // image fall shy of the edge of the terrain tile.
                if j == south_east_tile_coordinates.y
                    && (self.is_base_layer()
                        || (clipped_imagery_rectangle.south - terrain_rectangle.south).abs()
                            < very_close_y)
                {
                    min_v = 0.0;
                }

                let tex_coords_rectangle = DVec4::new(min_u, min_v, max_u, max_v);
                let key = TileKey::new(i, j, imagery_level);
                let imagery = self.add_imagery(&key).unwrap();
                tile.data.add(
                    imagery.clone(),
                    Some(tex_coords_rectangle),
                    use_web_mercator_t,
                );
                insertion_point += 1;
            }
        }

        return true;
    }
    // pub fn get_imagery_mut(&mut self, key: &TileKey) -> Option<&mut Imagery> {
    //     return self.imagery_cache.get_mut(key);
    // }
    pub fn get_imagery(&self, key: &TileKey) -> Option<&ShareMutImagery> {
        return self.imagery_cache.get(key);
    }
    pub fn get_imagery_mut(&mut self, key: &TileKey) -> Option<&mut ShareMutImagery> {
        return self.imagery_cache.get_mut(key);
    }
    pub fn add_imagery(&mut self, key: &TileKey) -> Option<&ShareMutImagery> {
        let imagery = self.imagery_cache.get(key);
        if imagery.is_none() {
            let parent = key.parent().and_then(|parent_key| {
                self.imagery_cache
                    .get(&parent_key)
                    .and_then(|parent| Some(parent.clone()))
            });
            let new_imagery = ShareMutImagery::new(key.clone(), self.id.clone(), parent);
            self.imagery_cache.insert(*key, new_imagery);
        }
        return self.get_imagery(key);
    }

    pub fn calculate_texture_translation_and_scale(
        &mut self,
        tile_rectangle: Rectangle,
        tile_imagery: &TileImagery,
    ) -> DVec4 {
        let mut imagery_rectangle = tile_imagery
            .ready_imagery
            .as_ref()
            .unwrap()
            .get_reactangle();
        let mut quand_tile_rectangle = tile_rectangle;

        if tile_imagery.use_web_mercator_t {
            let tiling_scheme = self.imagery_provider.get_tiling_scheme();
            imagery_rectangle = tiling_scheme.rectangle_to_native_rectangle(&imagery_rectangle);
            quand_tile_rectangle =
                tiling_scheme.rectangle_to_native_rectangle(&quand_tile_rectangle);
        }

        let terrain_width = quand_tile_rectangle.compute_width();
        let terrain_height = quand_tile_rectangle.compute_height();

        let scale_x = terrain_width / imagery_rectangle.compute_width();
        let scale_Y = terrain_height / imagery_rectangle.compute_height();
        return DVec4::new(
            (scale_x * (quand_tile_rectangle.west - imagery_rectangle.west)) / terrain_width,
            (scale_Y * (quand_tile_rectangle.south - imagery_rectangle.south)) / terrain_height,
            scale_x,
            scale_Y,
        );
    }
    pub fn reproject_texture(
        imagery: &Imagery,
        _need_geographic_projection: bool,
        images: &mut Assets<Image>,
        width: u32,
        height: u32,
        render_world_queue: &mut ReprojectTextureTaskQueue,
        indices_and_edges_cache: &IndicesAndEdgesCacheArc,
        render_device: &RenderDevice,
        globe_camera: &GlobeCamera,
    ) {
        info!("reproject texture key={:?}", imagery.key);
        let output_texture = images.add(Image {
            texture_descriptor: TextureDescriptor {
                label: "reproject_texture".into(),
                size: Extent3d {
                    width: width,

                    height: height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 0,
                sample_count: 0,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            ..Default::default()
        });
        let rectangle = &imagery.rectangle;
        let mut sin_latitude = rectangle.south.sin() as f32;
        let south_mercator_y = 0.5 * ((1.0 + sin_latitude) / (1.0 - sin_latitude)).ln();

        sin_latitude = rectangle.north.sin() as f32;
        let north_mercator_y = 0.5 * ((1.0 + sin_latitude) / (1.0 - sin_latitude).ln());
        let one_over_mercator_height = 1.0 / (north_mercator_y - south_mercator_y);
        let mut web_mercator_t: Vec<f32> = vec![0.0; 2 * 64];
        let south = imagery.rectangle.south as f32;
        let north = imagery.rectangle.north as f32;

        let mut output_index = 0;
        for web_mercator_t_index in 0..64 {
            let fraction = web_mercator_t_index as f32 / 63.0;
            let latitude = lerp_f32(south, north, fraction);
            sin_latitude = latitude.sin();
            let mercator_y = 0.5 * ((1.0 + sin_latitude) / (1.0 - sin_latitude)).ln();
            let mercator_fraction = (mercator_y - south_mercator_y) * one_over_mercator_height;
            web_mercator_t[output_index] = mercator_fraction;
            output_index += 1;
            web_mercator_t[output_index] = mercator_fraction;
            output_index += 1;
        }
        let webmercartor_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("webmercator_buffer"),
            contents: cast_slice(&web_mercator_t),
            usage: BufferUsages::VERTEX,
        });
        let indices = indices_and_edges_cache
            .0
            .clone()
            .lock()
            .unwrap()
            .getRegularGridIndices(2, 64);
        let index_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("indices_buffer"),
            contents: cast_slice(&indices),
            usage: BufferUsages::VERTEX,
        });
        let v = &globe_camera.viewport;
        let _viewportOrthographicMatrix = DMat4::compute_orthographic_off_center(
            v.x,
            v.x + v.width,
            v.y,
            v.y + v.height,
            0.0,
            1.0,
        )
        .to_mat4_32();
        let unifrom_params = ParamsUniforms {
            texture_dimensions: UVec2::new(width, height),
            viewport_orthographic: _viewportOrthographicMatrix,
        };
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&unifrom_params).unwrap();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            contents: &buffer.into_inner(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let task = ReprojectTextureTask {
            key: imagery.key,
            output_texture,
            image: imagery.texture.as_ref().expect("imagery.texture").clone(),
            webmercartor_buffer,
            index_buffer,
            uniform_buffer: buffer,
            imagery_layer_id: imagery.imagery_layer_id.clone(),
        };
        render_world_queue.push(task);
    }

    pub fn destroy(&mut self) {}
}
pub fn finish_reproject_texture_system(
    render_world_queue: ResMut<ReprojectTextureTaskQueue>,
    mut imagery_layer_storage: ResMut<ImageryLayerStorage>,
) {
    let (_, receiver) = render_world_queue.status_channel.clone();
    for _i in 0..render_world_queue.count() {
        let Ok((imagery_layer_id,key))  =receiver.try_recv()else{continue;};
        let task = render_world_queue.get(&key).expect("task");
        let imagery_layer = imagery_layer_storage.get_mut(&imagery_layer_id).unwrap();
        let imagery = imagery_layer.get_imagery_mut(&key).unwrap();
        imagery.set_texture(task.output_texture.clone());
        imagery.set_state(ImageryState::READY);
    }
}
