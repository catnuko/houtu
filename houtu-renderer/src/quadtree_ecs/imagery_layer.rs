use bevy::{
    asset::LoadState,
    core::cast_slice,
    math::{DMat4, DVec4},
    prelude::*,
    render::{
        define_atomic_id,
        render_resource::{
            encase, BufferDescriptor, BufferInitDescriptor, BufferUsages, Extent3d,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
    },
    utils::{HashMap, Uuid},
};

use bevy_egui::egui::epaint::image;
use houtu_scene::{lerp_f32, Matrix4, Rectangle, TilingScheme, WebMercatorProjection};

use crate::{
    camera::GlobeCamera,
    quadtree::{
        globe_surface_tile::TerrainState,
        imagery_provider::ImageryProvider,
        imagery_storage::ImageryState,
        reproject_texture::{ParamsUniforms, ReprojectTextureTask},
        texture_minification_filter::TextureMinificationFilter,
        tile_key::TileKey,
    },
};

use super::{
    height_map_terrain_data::HeightmapTerrainDataCom,
    imagery::{Imagery, ImageryCache},
    quadtree::Quadtree,
    quadtree_tile::QuadtreeTileLoadState,
    tile_imagery::{TileImagery, TileImageryVec},
};
define_atomic_id!(ImageryLayerId);
#[derive(Component)]
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
    pub rectangle: Rectangle,
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
    pub fn new() -> Self {
        let id = ImageryLayerId::new();
        Self {
            id: id,
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
            rectangle: Rectangle::MAX_VALUE.clone(),
            minification_filter: Self::DEFAULT_MINIFICATION_FILTER,
            magnification_filter: Self::DEFAULT_MAGNIFICATION_FILTER,
        }
    }
    pub fn get_level_with_maximum_texel_spacing(
        provider: &ImageryProviderCom,
        texel_spacing: f64,
        latitude_closest_to_equator: f64,
    ) -> u32 {
        // PERFORMANCE_IDEA: factor out the stuff that doesn't change.
        let tiling_scheme = provider.0.get_tiling_scheme();
        let ellipsoid = tiling_scheme.get_ellipsoid();
        let latitude_factor = if tiling_scheme.get_name() != "GeographicTilingScheme" {
            latitude_closest_to_equator.cos()
        } else {
            1.0
        };
        let tiling_scheme_rectangle = tiling_scheme.get_rectangle();
        let v = tiling_scheme.get_number_of_x_tiles_at_level(0);
        let vv = tiling_scheme_rectangle.compute_width();
        let vvv = provider.0.get_tile_width();
        let level_zero_maximum_texel_spacing =
            (ellipsoid.maximum_radius * vv * latitude_factor) / (vvv * v) as f64;

        let two_to_the_level_power = level_zero_maximum_texel_spacing / texel_spacing;
        let level = two_to_the_level_power.ln() / 2f64.ln();
        let rounded = level.round() as u32;
        return rounded | 0;
    }
    pub fn calculate_texture_translation_and_scale(
        provider: &ImageryProviderCom,
        quad_tile_rectangle: Rectangle,
        tile_imagery: &TileImagery,
    ) -> DVec4 {
        let mut imagery_rectangle = tile_imagery
            .ready_imagery
            .as_ref()
            .unwrap()
            .0
            .read()
            .unwrap()
            .rectangle
            .clone();
        let mut quad_tile_rectangle = quad_tile_rectangle;

        if tile_imagery.use_web_mercator_t {
            let tiling_scheme = provider.0.get_tiling_scheme();
            imagery_rectangle = tiling_scheme.rectangle_to_native_rectangle(&imagery_rectangle);
            quad_tile_rectangle = tiling_scheme.rectangle_to_native_rectangle(&quad_tile_rectangle);
        }

        let terrain_width = quad_tile_rectangle.compute_width();
        let terrain_height = quad_tile_rectangle.compute_height();
        let imagery_width = imagery_rectangle.compute_width();
        let imagery_height = imagery_rectangle.compute_height();

        let scale_x = terrain_width / imagery_width;
        let scale_y = terrain_height / imagery_height;
        return DVec4::new(
            (quad_tile_rectangle.west - imagery_rectangle.west) / imagery_width,
            (imagery_rectangle.north - quad_tile_rectangle.north) / imagery_height,
            scale_x,
            scale_y,
        );
    }
}
#[derive(Component)]
pub struct ImageryProviderCom(pub Box<dyn ImageryProvider>);
#[derive(Bundle)]
pub struct ImageryLayerBundle {
    imagery_layer: ImageryLayer,
    provider: ImageryProviderCom,
    storage: ImageryCache,
    visibility: Visibility,
}
impl ImageryLayerBundle {
    pub fn new(provider: Box<dyn ImageryProvider>) -> Self {
        Self {
            imagery_layer: ImageryLayer::new(),
            provider: ImageryProviderCom(provider),
            storage: ImageryCache::default(),
            visibility: Visibility::Visible,
        }
    }
}
pub fn initialize(
    quadtree: Res<Quadtree>,
    mut quadtree_tile_query: Query<(
        Entity,
        &mut QuadtreeTileLoadState,
        &mut TileImageryVec,
        &Rectangle,
        &TileKey,
        &mut TerrainState,
    )>,
    parent_terrain_data_query: Query<(&Parent, &HeightmapTerrainDataCom, &TileKey)>,
    mut layer_query: Query<(
        Entity,
        &ImageryLayer,
        &ImageryProviderCom,
        &mut ImageryCache,
        &Visibility,
    )>,
) {
    for (entity, mut state, mut imagers, tile_rectangle, tile_key, mut terrain_state) in
        &mut quadtree_tile_query
    {
        if *state != QuadtreeTileLoadState::START {
            continue;
        }
        //prepare_new_tile
        let mut available = quadtree.terrain_provider.get_tile_data_available(tile_key);
        let parent_res = parent_terrain_data_query.get(entity);
        if available.is_none() && parent_res.is_ok() {
            let parent = parent_res.unwrap();
            available = Some(
                parent
                    .1
                     .0
                    .is_child_available(parent.2.x, parent.2.y, tile_key.x, tile_key.y),
            );
        }
        if available == Some(false) {
            *terrain_state = TerrainState::FAILED;
        }

        for (layer_entity, layer, provider, mut cache, visibility) in &mut layer_query {
            if visibility == Visibility::Visible {
                //_create_tile_imagery_skeletons
                if !layer.ready || !provider.0.get_ready() {
                    // let imagery_key = imagery_storage.add(
                    //     &TileKey {
                    //         x: 0,
                    //         y: 0,
                    //         level: 0,
                    //     },
                    //     &self.id,
                    //     provider.0.get_tiling_scheme(),
                    // );
                    // tile.data.add_imagery(imagery_key, None, false);
                    // imagery_storage.add_reference(self.skeleton_placeholder.loading_imagery.as_ref().unwrap());
                    // tile.data.imagery.splice(range, replace_with)
                    // return true;
                    panic!("暂时不可到达这里");
                }
                let use_web_mercator_t = provider.0.get_tiling_scheme().get_name()
                    == "WebMercatorTilingScheme"
                    && tile_rectangle.north < WebMercatorProjection::MAXIMUM_LATITUDE
                    && tile_rectangle.south > -WebMercatorProjection::MAXIMUM_LATITUDE;
                let mut imagery_bounds = provider
                    .0
                    .get_rectangle()
                    .intersection(&layer.rectangle)
                    .expect("多边形相交没结果");
                let mut rectangle = tile_rectangle.intersection(&imagery_bounds);
                if rectangle.is_none() {
                    // There is no overlap between this terrain tile and this imagery
                    // provider.  Unless this is the base layer, no skeletons need to be created.
                    // We stretch texels at the edge of the base layer over the entire globe.
                    /*
                    当前地形瓦片和imagery provider没有重叠，除非这是个基础图层，否则不需要创建骨架。
                    我们拉伸基础图层的边缘上的像素填满整个地球
                     */
                    if !layer.is_base_layer {
                        // return false;
                    }

                    let base_imagery_rectangle = imagery_bounds;
                    let base_terrain_rectangle = tile_rectangle;
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
                        new_rectangle.west =
                            base_terrain_rectangle.west.max(base_imagery_rectangle.west);
                        new_rectangle.east =
                            base_terrain_rectangle.east.min(base_imagery_rectangle.east);
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
                let target_geometric_error = error_ratio
                    * quadtree
                        .terrain_provider
                        .get_level_maximum_geometric_error(tile_key.level);
                let mut imagery_level = ImageryLayer::get_level_with_maximum_texel_spacing(
                    provider,
                    target_geometric_error,
                    latitude_closest_to_equator,
                );
                imagery_level = 0.max(imagery_level);
                let maximum_level = provider.0.get_maximum_level();
                if imagery_level > maximum_level {
                    imagery_level = maximum_level;
                }
                let minimum_level = provider.0.get_minimum_level();
                if imagery_level < minimum_level {
                    imagery_level = minimum_level;
                }

                let imagery_tiling_scheme = provider.0.get_tiling_scheme();
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
                let mut very_close_x = tile_rectangle.compute_width() / 512.0;
                let mut very_close_y = tile_rectangle.compute_height() / 512.0;

                let north_west_tile_rectangle = imagery_tiling_scheme.tile_x_y_to_rectange(
                    north_west_tile_coordinates.x,
                    north_west_tile_coordinates.y,
                    imagery_level,
                );
                if (north_west_tile_rectangle.south - tile_rectangle.north).abs() < very_close_y
                    && north_west_tile_coordinates.y < south_east_tile_coordinates.y
                {
                    //西北瓦片的Y加一相当于略去该瓦片，因为下面的循环是从西北瓦片的Y开始的
                    north_west_tile_coordinates.y += 1;
                }

                if (north_west_tile_rectangle.east - tile_rectangle.west).abs() < very_close_x
                    && north_west_tile_coordinates.x < south_east_tile_coordinates.x
                {
                    north_west_tile_coordinates.x += 1;
                }

                let south_east_tile_rectangle = imagery_tiling_scheme.tile_x_y_to_rectange(
                    south_east_tile_coordinates.x,
                    south_east_tile_coordinates.y,
                    imagery_level,
                );
                if (south_east_tile_rectangle.north - tile_rectangle.south).abs() < very_close_y
                    && south_east_tile_coordinates.y > north_west_tile_coordinates.y
                {
                    //东南瓦片的Y减一相当于略去该瓦片，因为下面的循环是到东南瓦片的Y结束的
                    south_east_tile_coordinates.y -= 1;
                }
                if (south_east_tile_rectangle.west - tile_rectangle.east).abs() < very_close_x
                    && south_east_tile_coordinates.x > north_west_tile_coordinates.x
                {
                    south_east_tile_coordinates.x -= 1;
                }

                // Create TileImagery instances for each imagery tile overlapping this terrain tile.
                // We need to do all texture coordinate computations in the imagery tile's tiling scheme.
                /*
                创建TileImagery实例为每个和地形瓦片重叠的图片瓦片。我们需要在图片瓦片的tilng scheme中计算所有的材质坐标。
                 */
                let mut terrain_rectangle = tile_rectangle.clone();
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
                    clipped_imagery_rectangle = imagery_tiling_scheme
                        .rectangle_to_native_rectangle(&clipped_imagery_rectangle);
                    imagery_bounds =
                        imagery_tiling_scheme.rectangle_to_native_rectangle(&imagery_bounds);
                    use_native = true;
                    very_close_x = terrain_rectangle.compute_width() / 512.0;
                    very_close_y = terrain_rectangle.compute_height() / 512.0;
                } else {
                    use_native = false;
                }

                let mut min_u;
                let mut max_u = 0.0;

                let mut min_v;
                let mut max_v = 0.0;

                // If this is the northern-most or western-most tile in the imagery tiling scheme,
                // it may not start at the northern or western edge of the terrain tile.
                // Calculate where it does start.
                // 如果这在imagery tiling scheme中是个更北和更西的瓦片，那么它不可能在地形瓦片的西边和北边开始
                // 计算它从哪开始
                if !layer.is_base_layer
                    && (clipped_imagery_rectangle.west - terrain_rectangle.west).abs()
                        >= very_close_x
                {
                    max_u = (1.0_f64).min(
                        (clipped_imagery_rectangle.west - terrain_rectangle.west)
                            / terrain_rectangle.compute_width(),
                    );
                }

                if !layer.is_base_layer
                    && (clipped_imagery_rectangle.south - terrain_rectangle.south).abs()
                        >= very_close_y
                {
                    max_v = (1.0_f64).min(
                        (clipped_imagery_rectangle.south - terrain_rectangle.south)
                            / terrain_rectangle.compute_height(),
                    );
                }

                let initial_max_v = max_v;
                for i in north_west_tile_coordinates.x..=south_east_tile_coordinates.x {
                    min_u = max_u; //上一轮的max_u将是这一轮的min_u。min_u在此赋值完成，从左向右，依次有图片p1,p2,p3，分别于地形瓦片相交于pg1,pg2,pg3,那么，pg1的右侧即是pg2的左侧，pg2的右侧是pg3的左侧。

                    imagery_rectangle = if use_native {
                        provider.0.get_tiling_scheme().tile_x_y_to_native_rectange(
                            i,
                            north_west_tile_coordinates.y,
                            imagery_level,
                        )
                    } else {
                        provider.0.get_tiling_scheme().tile_x_y_to_rectange(
                            i,
                            north_west_tile_coordinates.y,
                            imagery_level,
                        )
                    };

                    let clipped_imagery_rectangle_res =
                        imagery_rectangle.simple_intersection(&imagery_bounds);

                    if clipped_imagery_rectangle_res.is_none() {
                        continue;
                    }
                    clipped_imagery_rectangle =
                        clipped_imagery_rectangle_res.expect("rectangle is some");

                    max_u = (1.0 as f64).min(
                        (clipped_imagery_rectangle.east - terrain_rectangle.west)
                            / terrain_rectangle.compute_width(),
                    );

                    // If this is the eastern-most imagery tile mapped to this terrain tile,
                    // and there are more imagery tiles to the east of this one, the max_u
                    // should be 1.0 to make sure rounding errors don't make the last
                    // image fall shy of the edge of the terrain tile.
                    // 如果这是个映射到地形瓦片上的最东边的图片瓦片，这个瓦片的东边还有更多的图片瓦片，
                    // max_u应该是1.0以确保四舍五入的误差不会使最后一个瓦片落在地形瓦片的边缘。

                    if i == south_east_tile_coordinates.x
                        && (layer.is_base_layer
                            || (clipped_imagery_rectangle.east - terrain_rectangle.east).abs()
                                < very_close_x)
                    {
                        max_u = 1.0;
                    }

                    max_v = initial_max_v; //min_v = initial_min_v = 1;，为什么给min_v初始值呢？因为下面会改min_v
                    for j in north_west_tile_coordinates.y..=south_east_tile_coordinates.y {
                        min_v = max_v; //上一轮的min_v将是本轮的max_v，max_v在此赋值完成，从上向下，依次有图片p1,p2,p3，分别于地形瓦片相交于pg1,pg2,pg3,那么，pg1的下侧即是pg2的上侧，pg2的下侧是pg3的上侧。

                        imagery_rectangle = if use_native {
                            provider.0.get_tiling_scheme().tile_x_y_to_native_rectange(
                                i,
                                j,
                                imagery_level,
                            )
                        } else {
                            provider
                                .0
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
                        max_v = (1.0_f64).min(
                            //min_v重新计算的
                            (terrain_rectangle.north - clipped_imagery_rectangle.south)
                                / terrain_rectangle.compute_height(),
                        );

                        // If this is the southern-most imagery tile mapped to this terrain tile,
                        // and there are more imagery tiles to the south of this one, the min_v
                        // should be 0.0 to make sure rounding errors don't make the last
                        // image fall shy of the edge of the terrain tile.

                        // 如果这是个映射到地形瓦片上的最南边的图片瓦片，这个瓦片的南边还有更多的图片瓦片，
                        // max_v应该是0.0以确保四舍五入的误差不会使最后一个瓦片落在地形瓦片的边缘。
                        if j == south_east_tile_coordinates.y
                            && (layer.is_base_layer
                                || (clipped_imagery_rectangle.south - terrain_rectangle.south)
                                    .abs()
                                    < very_close_y)
                        {
                            max_v = 1.0;
                        }

                        let tex_coords_rectangle = DVec4::new(min_u, min_v, max_u, max_v);
                        let key = TileKey::new(i, j, imagery_level);
                        let imagery = cache.add(layer_entity, provider, &key);
                        if i == 1 && j == 2 && imagery_level == 3 {
                            info!("{}", use_web_mercator_t);
                        }
                        imagers.add(
                            imagery,
                            Some(tex_coords_rectangle),
                            use_web_mercator_t,
                            None,
                        );
                    }
                }
            }
        }
        *state = QuadtreeTileLoadState::LOADING;
    }
}
