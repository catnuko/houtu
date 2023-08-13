use bevy::{
    core::cast_slice,
    math::{DMat4, DVec4},
    prelude::*,
    render::{
        define_atomic_id,
        render_resource::{
            encase, BufferInitDescriptor, BufferUsages, Extent3d, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
    },
    utils::{HashMap, Uuid},
};

use bevy_egui::egui::epaint::image;
use houtu_scene::{lerp_f32, Matrix4, Rectangle, TilingScheme, WebMercatorProjection};
use wgpu::BufferDescriptor;

use crate::{
    camera::GlobeCamera,
    quadtree::{
        reproject_texture::{ParamsUniforms, ReprojectTextureTask},
        texture_minification_filter::TextureMinificationFilter,
    },
};

use super::{
    imagery_layer_storage::ImageryLayerStorage,
    imagery_provider::ImageryProvider,
    imagery_storage::{Imagery, ImageryKey, ImageryState, ImageryStorage},
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_tile::QuadtreeTile,
    reproject_texture::{
        FinishReprojectTexture, ReprojectTextureTaskQueue, ReprojectTextureTaskState,
    },
    terrain_provider::TerrainProvider,
    tile_imagery::TileImagery,
    tile_key::TileKey,
};
define_atomic_id!(ImageryLayerId);
pub struct ImageryLayer {
    pub id: ImageryLayerId,
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
    pub imagery_provider: Box<dyn ImageryProvider>,
    pub show: bool,
    pub minification_filter: TextureMinificationFilter,
    pub magnification_filter: TextureMinificationFilter,
}
impl ImageryLayer {
    pub const DEFAULT_MINIFICATION_FILTER: TextureMinificationFilter =
        TextureMinificationFilter::Linear;
    pub const DEFAULT_MAGNIFICATION_FILTER: TextureMinificationFilter =
        TextureMinificationFilter::Linear;
    pub const DEFAULT_BRIGHTNESS: f64 = 1.0;
    pub const DEFAULT_CONTRAST: f64 = 1.0;
    pub const DEFAULT_HUE: f64 = 0.0;
    pub const DEFAULT_SATURATION: f64 = 1.0;
    pub const DEFAULT_GAMMA: f64 = 1.0;
    pub const DEFAULT_APPLY_COLOR_TO_ALPHA_THRESHOLD: f64 = 0.004;
    pub fn new(imagery_provider: Box<dyn ImageryProvider>) -> Self {
        Self {
            id: ImageryLayerId::new(),
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
            // skeleton_placeholder: Tileimagery_storage::create_placeholder(imagery_layer_id, None),
            // imagery_cache: HashMap::new(),
            imagery_provider: imagery_provider,
            show: true,
            minification_filter: Self::DEFAULT_MINIFICATION_FILTER,
            magnification_filter: Self::DEFAULT_MAGNIFICATION_FILTER,
        }
    }
    fn get_level_with_maximum_texel_spacing(
        &mut self,
        texel_spacing: f64,
        latitude_closest_to_equator: f64,
    ) -> u32 {
        // PERFORMANCE_IDEA: factor out the stuff that doesn't change.
        let tiling_scheme = self.imagery_provider.get_tiling_scheme();
        let ellipsoid = tiling_scheme.get_ellipsoid();
        let latitude_factor = if false {
            latitude_closest_to_equator.cos()
        } else {
            1.0
        };
        let tiling_scheme_rectangle = tiling_scheme.get_rectangle();
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
    pub fn _create_tile_imagery_skeletons(
        &mut self,
        tile: &mut QuadtreeTile,
        terrain_provider: &Box<dyn TerrainProvider>,
        imagery_storage: &mut ImageryStorage,
    ) -> bool {
        let mut insertion_point = tile.data.imagery.len();
        if !self.ready || !self.imagery_provider.get_ready() {
            let imagery_key = imagery_storage.add(
                &TileKey {
                    x: 0,
                    y: 0,
                    level: 0,
                },
                &self.id,
            );
            tile.data.add_imagery(imagery_key, None, false);
            return true;
        }
        let use_web_mercator_t = self.imagery_provider.get_tiling_scheme().get_name()
            == "WebMercatorTilingScheme"
            && tile.rectangle.north < WebMercatorProjection::MAXIMUM_LATITUDE
            && tile.rectangle.south > -WebMercatorProjection::MAXIMUM_LATITUDE;

        let mut imagery_bounds = self
            .imagery_provider
            .get_rectangle()
            .intersection(&self._rectangle)
            .expect("多边形相交没结果");
        let mut rectangle = tile.rectangle.intersection(&imagery_bounds);

        if rectangle.is_none() {
            // There is no overlap between this terrain tile and this imagery
            // provider.  Unless this is the base layer, no skeletons need to be created.
            // We stretch texels at the edge of the base layer over the entire globe.
            /*
            当前地形瓦片和imagery provider没有重叠，除非这是个基础图层，否则不需要创建骨架。
            我们拉伸基础图层的边缘上的像素填满整个地球
             */
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
            rectangle = Some(new_rectangle)
        }
        let rectangle = rectangle.unwrap();
        let mut latitude_closest_to_equator = 0.0;
        if rectangle.south > 0.0 {
            latitude_closest_to_equator = rectangle.south;
        } else if rectangle.north < 0.0 {
            latitude_closest_to_equator = rectangle.north;
        }

        // Compute the required level in the imagery tiling scheme.
        // The error_ratio should really be imagerySSE / terrainSSE rather than this hard-coded value.
        // But first we need configurable imagery SSE and we need the rendering to be able to handle more
        // images attached to a terrain tile than there are available texture units.  So that's for the future.

        // 在tiling scheme中计算需要的层级。error_ratio应当是imagery或terrain的屏幕空间误差，而不是这里的硬编码。
        // 但是首先，我们需要可配置的imagery的屏幕空间误差，我们需要渲染能处理更多的附加到地形瓦片上的image，而不是可用的纹理单元。
        let error_ratio = 1.0;
        let target_geometric_error =
            error_ratio * terrain_provider.get_level_maximum_geometric_error(tile.key.level);
        let mut imagery_level = self.get_level_with_maximum_texel_spacing(
            target_geometric_error,
            latitude_closest_to_equator,
        );
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
            .position_to_tile_x_y(&rectangle.north_west(), imagery_level)
            .expect("north_west_tile_coordinates");
        let mut south_east_tile_coordinates = imagery_tiling_scheme
            .position_to_tile_x_y(&rectangle.south_east(), imagery_level)
            .expect("south_east_tile_coordinates");

        // If the southeast corner of the rectangle lies very close to the north or west side
        // of the southeast tile, we don't actually need the southernmost or easternmost
        // tiles.
        // Similarly, if the northwest corner of the rectangle lies very close to the south or east side
        // of the northwest tile, we don't actually need the northernmost or westernmost tiles.

        // We define "very close" as being within 1/512 of the width of the tile.

        // 如果矩形的东南角非常贴近东南角瓦片的西边或北边，那么我们实际上不需要更南和更东的瓦片。
        // 同样，如果西北角非常贴近西北角瓦片的东边和南边，那么我们实际上也不需要更北和更西的瓦片
        // 将小于瓦片宽度/512定义为非常题贴近。
        // 这里的目标是计算地形瓦片上的图片瓦片有哪些，如果一个图片瓦片和地形瓦片没重叠。即使离得很近，也不需要。
        let mut very_close_x = tile.rectangle.compute_width() / 512.0;
        let mut very_close_y = tile.rectangle.compute_height() / 512.0;

        let north_west_tile_rectangle = imagery_tiling_scheme.tile_x_y_to_rectange(
            north_west_tile_coordinates.x,
            north_west_tile_coordinates.y,
            imagery_level,
        );
        if (north_west_tile_rectangle.south - tile.rectangle.north).abs() < very_close_y
            && north_west_tile_coordinates.y < south_east_tile_coordinates.y
        {
            //西北瓦片的Y加一相当于略去该瓦片，因为下面的循环是从西北瓦片的Y开始的
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
            //东南瓦片的Y减一相当于略去该瓦片，因为下面的循环是到东南瓦片的Y结束的
            south_east_tile_coordinates.y -= 1;
        }
        if (south_east_tile_rectangle.west - tile.rectangle.east).abs() < very_close_x
            && south_east_tile_coordinates.x > north_west_tile_coordinates.x
        {
            south_east_tile_coordinates.x -= 1;
        }

        // Create TileImagery instances for each imagery tile overlapping this terrain tile.
        // We need to do all texture coordinate computations in the imagery tile's tiling scheme.
        /*
        创建TileImagery实例为每个和地形瓦片重叠的图片瓦片。我们需要在图片瓦片的tilng scheme中计算所有的材质坐标。
         */
        let mut terrain_rectangle = tile.rectangle.clone();
        let mut imagery_rectangle = imagery_tiling_scheme.tile_x_y_to_rectange(
            north_west_tile_coordinates.x,
            north_west_tile_coordinates.y,
            imagery_level,
        );
        let mut clipped_imagery_rectangle = imagery_rectangle
            .intersection(&imagery_bounds)
            .expect("clipped_imagery_rectangle");

        let mut use_native = false;
        if use_web_mercator_t {
            terrain_rectangle =
                imagery_tiling_scheme.rectangle_to_native_rectangle(&terrain_rectangle);
            imagery_rectangle =
                imagery_tiling_scheme.rectangle_to_native_rectangle(&imagery_rectangle);
            clipped_imagery_rectangle =
                imagery_tiling_scheme.rectangle_to_native_rectangle(&clipped_imagery_rectangle);
            imagery_bounds = imagery_tiling_scheme.rectangle_to_native_rectangle(&imagery_bounds);
            use_native = true;
            very_close_x = terrain_rectangle.compute_width() / 512.0;
            very_close_y = terrain_rectangle.compute_height() / 512.0;
        } else {
            use_native = false;
        }

        let mut min_u;
        let mut max_u = 0.0;

        let mut min_v = 1.0;
        let mut max_v;

        // If this is the northern-most or western-most tile in the imagery tiling scheme,
        // it may not start at the northern or western edge of the terrain tile.
        // Calculate where it does start.
        // 如果这在imagery tiling scheme中是个更北和更西的瓦片，那么它不可能在地形瓦片的西边和北边开始
        // 计算它从哪开始
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

        let initial_min_v = min_v;
        for i in north_west_tile_coordinates.x..=south_east_tile_coordinates.x {
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

            let clipped_imagery_rectangle_res =
                imagery_rectangle.simple_intersection(&imagery_bounds);

            if clipped_imagery_rectangle_res.is_none() {
                continue;
            }
            clipped_imagery_rectangle = clipped_imagery_rectangle_res.expect("rectangle is some");

            max_u = (1.0 as f64).min(
                (clipped_imagery_rectangle.east - terrain_rectangle.west)
                    / terrain_rectangle.compute_width(),
            );

            // If this is the eastern-most imagery tile mapped to this terrain tile,
            // and there are more imagery tiles to the east of this one, the max_u
            // should be 1.0 to make sure rounding errors don't make the last
            // image fall shy of the edge of the terrain tile.
            // 如果这是个映射到地形瓦片上的更东的图片瓦片，这个瓦片的东边还有更多的图片瓦片，
            // max_u应该是1.0以确保四舍五入的误差不会使最后一个瓦片落在地形瓦片的边缘。

            if i == south_east_tile_coordinates.x
                && (self.is_base_layer()
                    || (clipped_imagery_rectangle.east - terrain_rectangle.east).abs()
                        < very_close_x)
            {
                max_u = 1.0;
            }

            min_v = initial_min_v;
            for j in north_west_tile_coordinates.y..=south_east_tile_coordinates.y {
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
                let clipped_imagery_rectangle_res =
                    imagery_rectangle.simple_intersection(&imagery_bounds);

                if clipped_imagery_rectangle_res.is_none() {
                    continue;
                }
                clipped_imagery_rectangle =
                    clipped_imagery_rectangle_res.expect("rectangle is some");
                min_v = (0.0 as f64).max(
                    (clipped_imagery_rectangle.south - terrain_rectangle.south)
                        / terrain_rectangle.compute_height(),
                );

                // If this is the southern-most imagery tile mapped to this terrain tile,
                // and there are more imagery tiles to the south of this one, the min_v
                // should be 0.0 to make sure rounding errors don't make the last
                // image fall shy of the edge of the terrain tile.

                // 如果这是个映射到地形瓦片上的更南的图片瓦片，这个瓦片的南边还有更多的图片瓦片，
                // max_v应该是0.0以确保四舍五入的误差不会使最后一个瓦片落在地形瓦片的边缘。
                if j == south_east_tile_coordinates.y
                    && (self.is_base_layer()
                        || (clipped_imagery_rectangle.south - terrain_rectangle.south).abs()
                            < very_close_y)
                {
                    min_v = 0.0;
                }

                let tex_coords_rectangle = DVec4::new(min_u, min_v, max_u, max_v);
                let key = TileKey::new(i, j, imagery_level);
                let imagery_key = imagery_storage.add(&key, &self.id);
                tile.data.add_imagery(
                    imagery_key.clone(),
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
    // pub fn get_imagery(&self, key: &ImageryKey) -> Option<&Imagery> {
    //     return self.imagery_cache.get(key);
    // }
    // pub fn get_imagery_mut(&mut self, key: &ImageryKey) -> Option<&mut Imagery> {
    //     return self.imagery_cache.get_mut(key);
    // }

    ///新增一个Imagery，并返回ImageryKey
    // pub fn add_imagery(&mut self, key: &TileKey) -> ImageryKey {
    //     let imagery_key = ImageryKey::new(*key, self.id);
    //     let imagery = self.imagery_cache.get(&imagery_key);
    //     if imagery.is_none() {
    //         let new_imagery = imagery_storage::new(
    //             key.clone(),
    //             self.id.clone(),
    //             key.parent().and_then(|x| Some(ImageryKey::new(x, self.id))),
    //         );
    //         let key = new_imagery.key.clone();
    //         self.imagery_cache.insert(new_imagery.key, new_imagery);
    //         bevy::log::info!("add imagery,{:?}", key.key);
    //         return key;
    //     }
    //     return imagery_key;
    // }

    pub fn calculate_texture_translation_and_scale(
        &mut self,
        tile_rectangle: Rectangle,
        tile_imagery: &TileImagery,
        imagery_rectangle: Rectangle,
    ) -> DVec4 {
        let mut imagery_rectangle = imagery_rectangle;
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
        let scale_y = terrain_height / imagery_rectangle.compute_height();
        return DVec4::new(
            (scale_x * (quand_tile_rectangle.west - imagery_rectangle.west)) / terrain_width,
            (scale_y * (quand_tile_rectangle.south - imagery_rectangle.south)) / terrain_height,
            scale_x,
            scale_y,
        );
    }
    pub fn reproject_texture(
        imagery_key: &mut ImageryKey,
        imagery_storage: &mut ImageryStorage,
        need_geographic_projection: bool,
        images: &mut Assets<Image>,
        width: u32,
        height: u32,
        render_world_queue: &mut ReprojectTextureTaskQueue,
        indices_and_edges_cache: &IndicesAndEdgesCacheArc,
        render_device: &RenderDevice,
        globe_camera: &GlobeCamera,
        projection_name: &'static str,
    ) {
        let imagery = imagery_storage.get_mut(&imagery_key).unwrap();
        let rectangle = imagery.rectangle.clone();
        if need_geographic_projection
            && projection_name != "GeographicTilingScheme"
            && rectangle.compute_width() / width as f64 > 1e-5
        {
            imagery_storage.add_reference(&imagery_key);
            let mut sin_latitude = rectangle.south.sin() as f32;
            let south_mercator_y = 0.5 * ((1.0 + sin_latitude) / (1.0 - sin_latitude)).ln();

            sin_latitude = rectangle.north.sin() as f32;
            let north_mercator_y = 0.5 * ((1.0 + sin_latitude) / (1.0 - sin_latitude).ln());
            let one_over_mercator_height = 1.0 / (north_mercator_y - south_mercator_y);
            let mut web_mercator_t: Vec<f32> = vec![0.0; 2 * 64];

            let imagery = imagery_storage.get_mut(&imagery_key).unwrap();
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
            let webmercartor_buffer =
                render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("webmercator_buffer"),
                    contents: cast_slice(&web_mercator_t),
                    usage: BufferUsages::VERTEX,
                });
            let indices = indices_and_edges_cache
                .0
                .clone()
                .lock()
                .unwrap()
                .get_regular_grid_indices(2, 64);
            let index_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("indices_buffer"),
                contents: cast_slice(&indices),
                usage: BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST,
            });
            let v = &globe_camera.viewport;
            let _viewport_orthographic_matrix = DMat4::compute_orthographic_off_center(
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
                viewport_orthographic: _viewport_orthographic_matrix,
            };
            let mut buffer = encase::UniformBuffer::new(Vec::new());
            buffer.write(&unifrom_params).unwrap();

            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: None,
                contents: &buffer.into_inner(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
            let size = Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            };
            let mut image = Image {
                texture_descriptor: TextureDescriptor {
                    label: "reproject_texture".into(),
                    size: size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Rgba8UnormSrgb,
                    usage: TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT
                        | TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC,
                    view_formats: &[],
                },
                ..Default::default()
            };
            image.resize(size);
            let image_handle = images.add(image);
            let u32_size = std::mem::size_of::<u32>() as u32;
            let output_buffer_size = (u32_size * width * height) as wgpu::BufferAddress;
            let output_buffer = render_device.create_buffer(&BufferDescriptor {
                size: output_buffer_size,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                label: None,
                mapped_at_creation: false,
            });
            let task = ReprojectTextureTask {
                output_buffer: output_buffer,
                output_texture: image_handle,
                image: imagery.texture.as_ref().expect("imagery.texture").clone(),
                webmercartor_buffer,
                index_buffer,
                uniform_buffer: buffer,
                imagery_key: imagery.key,
                state: ReprojectTextureTaskState::Start,
            };
            render_world_queue.push(task);
        } else {
            // if (need_geographic_projection) {
            //     imagery.texture = texture;
            // }
            // ImageryLayer::finalize_reproject_texture();
            let imagery = imagery_storage.get_mut(&imagery_key).unwrap();
            imagery.state = ImageryState::READY;
        }
    }

    pub fn destroy(&mut self) {}
    pub fn process_imagery_state_machine(
        &mut self,
        imagery_key: &ImageryKey,
        asset_server: &AssetServer,
        need_geographic_projection: bool,
        skip_loading: bool,
        images: &mut Assets<Image>,
        render_world_queue: &mut ReprojectTextureTaskQueue,
        indices_and_edges_cache: &IndicesAndEdgesCacheArc,
        render_device: &RenderDevice,
        globe_camera: &GlobeCamera,
        imagery_storage: &mut ImageryStorage,
    ) {
        let loading_imagery = imagery_storage.get_mut(imagery_key).unwrap();

        if loading_imagery.state == ImageryState::UNLOADED && !skip_loading {
            loading_imagery.state = ImageryState::TRANSITIONING;
            let request = self
                .imagery_provider
                .request_image(&imagery_key.key.clone(), asset_server);
            let loading_imagery = imagery_storage.get_mut(imagery_key).unwrap();
            if let Some(v) = request {
                loading_imagery.texture = Some(v);
                loading_imagery.state = ImageryState::RECEIVED;
            } else {
                loading_imagery.state = ImageryState::UNLOADED;
            }
        }

        let loading_imagery = imagery_storage.get_mut(imagery_key).unwrap();

        if loading_imagery.state == ImageryState::RECEIVED {
            loading_imagery.state = ImageryState::TRANSITIONING;
            loading_imagery.state = ImageryState::TEXTURE_LOADED;
        }

        // If the imagery is already ready, but we need a geographic version and don't have it yet,
        // we still need to do the reprojection step. imagery can happen if the Web Mercator version
        // is fine initially, but the geographic one is needed later.
        // 如果图片已经准备好，我们需要一个geographic版本的图片。但是现在还没有，下一步将重新投影该影像
        // 投影到web墨卡托投影才算完成
        let needs_reprojection =
            loading_imagery.state != ImageryState::READY && need_geographic_projection;

        if loading_imagery.state == ImageryState::TEXTURE_LOADED || needs_reprojection {
            loading_imagery.state = ImageryState::TRANSITIONING;
            let mut key = loading_imagery.key.clone();

            ImageryLayer::reproject_texture(
                &mut key,
                imagery_storage,
                need_geographic_projection,
                images,
                256,
                256,
                render_world_queue,
                indices_and_edges_cache,
                render_device,
                globe_camera,
                self.imagery_provider.get_tiling_scheme().get_name(),
            );
        }
    }
    pub fn finalize_reproject_texture(&self, pixel_format: TextureFormat) {
        // let mut minification_filter = &self.minification_filter;
        // let mut magnification_filter = &self.magnification_filter;
        // let uses_linear_texture_filter = *minification_filter == TextureMinificationFilter::Linear
        //     && *magnification_filter == TextureMinificationFilter::Linear;
        // if uses_linear_texture_filter&&
    }
}
pub fn finish_reproject_texture_system(
    mut render_world_queue: ResMut<ReprojectTextureTaskQueue>,
    mut imagery_storage: ResMut<ImageryStorage>,
    mut evt_reader: EventReader<FinishReprojectTexture>,
) {
    for evt in evt_reader.iter() {
        if let Some(task) = render_world_queue.remove(&evt.imagery_key) {
            let imagery = imagery_storage.get_mut(&evt.imagery_key).unwrap();
            imagery.texture = Some(task.output_texture.clone());
            imagery.state = ImageryState::READY;
            imagery_storage.release_reference(&evt.imagery_key);
        } else {
        }
    }
}
