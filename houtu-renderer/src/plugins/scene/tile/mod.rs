use bevy::{core::FrameCount, math::DVec3, prelude::*, utils::Uuid};

mod create_tile_job;
mod datasource;
mod layer_id;
mod terrian_material;
mod tile_key;
mod tile_layer;
pub mod tile_layer_loader;
pub mod tile_layer_state;
mod tile_replace_queue;
mod tile_selection_result;
mod tile_state;
mod tile_z;
use datasource::*;
use houtu_jobs::JobSpawner;
use houtu_scene::{
    Cartographic, CullingVolume, Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme,
    IndicesAndEdgesCache, Projection, Rectangle, TilingScheme,
};
use layer_id::*;
use lazy_static::lazy_static;
use rand::Rng;
use terrian_material::*;
use tile_key::*;
use tile_layer::*;
use tile_replace_queue::*;
use tile_selection_result::*;
use tile_state::*;
use tile_z::*;

use crate::plugins::{
    camera::GlobeCameraControl,
    quadtree::{self, QuadtreeNode, QuadtreeTile, QuadtreeTileLoadState, QuadtreeTileValue},
};

use self::{
    create_tile_job::CreateTileJob,
    tile_layer_loader::{EnQueue, TileLayerLoader},
};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        let tile_layer = TileLayer::new(Some(GeographicTilingScheme::default()));
        app.add_plugin(MaterialPlugin::<TerrainMeshMaterial>::default())
            .add_plugin(tile_layer_loader::Plugin)
            .register_type::<TerrainMeshMaterial>()
            .insert_resource(tile_layer)
            .insert_resource(TileLayerLoader::new(
                tile_layer.datasource as &dyn DataSource,
            ))
            .insert_resource(IndicesAndEdgesCache::new())
            // .add_system(layer_system)
            // .add_system(tile_state_system)
            .add_system(create_tile_job::handle_created_tile_system);
    }
    fn name(&self) -> &str {
        "houtu_tile_plugin"
    }
}

type QuadTreeTileLayer = TileLayer<GlobeSurfaceTileDataSource, GeographicTilingScheme>;
fn setup(mut tile_layer: ResMut<QuadTreeTileLayer>, mut commands: Commands) {}

fn layer_system(
    mut tile_layer: Res<QuadTreeTileLayer>,
    mut ellipsoid: Res<Ellipsoid>,
    mut commands: Commands,
    query: Query<&mut GlobeCameraControl>,
    mut job_spawner: houtu_jobs::JobSpawner,
    mut frame_count: Res<FrameCount>,
    mut enqueue_evt: EventWriter<EnQueue>,
) {
    if tile_layer.quadtree.roots.len() == 0 {
        if tile_layer.datasource.is_ready() {
            let tiling_scehme = tile_layer.datasource.get_tiling_scheme();
            let tiles = QuadtreeTile::c(tiling_scehme);
            tiles.iter().for_each(|x| tile_layer.quadtree.add_node(*x))
        } else {
            return;
        }
    }
    let occluders = {
        if tile_layer.quadtree.roots.len() > 1 {
            Some(tile_layer._occluders)
        } else {
            None
        }
    };
    for globe_camera_control in query.iter() {
        for (tile) in tile_layer.quadtree.roots.iter() {
            if !tile.renderable {
                queueTileLoad(tile, &mut enqueue_evt, tile_layer_loader::QueueType::High);
            } else {
                visit_if_visible(
                    &mut *tile_layer,
                    &mut *tile,
                    &occluders.unwrap(),
                    &globe_camera_control,
                    &tile_layer.tiling_scheme.projection,
                    &frame_count,
                    &mut enqueue_evt,
                );
            }
        }
    }
    tile_layer._lastSelectionFrameNumber = frame_count.0;
}
fn visit_if_visible<P: Projection>(
    tile_layer: &mut QuadTreeTileLayer,
    tile: &mut QuadtreeTile,
    occluders: &EllipsoidalOccluder,
    globe_camera_control: &GlobeCameraControl,
    projection: &P,
    frame_count: &Res<FrameCount>,
    mut enqueue_evt: EventWriter<EnQueue>,
) {
    let position_cartographic = &globe_camera_control
        .position_cartographic
        .expect("GlobeCameraControl:position_cartographic is undifined");
    if tile_layer.computeTileVisibility(
        tile,
        globe_camera_control.cullingVolume,
        occluders,
        &globe_camera_control.position_cartesian,
        position_cartographic,
        projection,
    ) != houtu_scene::Visibility::NONE
    {
        // return visitTile();
    }
    if contains_needed_position(&tile.rectangle, Some(position_cartographic)) {
        if tile.terrain_mesh.is_none() {
            queueTileLoad(tile, &mut enqueue_evt, tile_layer_loader::QueueType::Medium);
        }
        let lastFrame = tile_layer._lastSelectionFrameNumber;
        let lastFrameSelectionResult = {
            if tile.last_selection_result_frame == lastFrame {
                tile.last_selection_result
            } else {
                TileSelectionResult::NONE
            }
        };
        if lastFrameSelectionResult != TileSelectionResult::CullButNeeded
            && lastFrameSelectionResult != TileSelectionResult::RENDERED
        {
            // tile_layer._tileToUpdateHeights.push(tile);
            tile.to_update_heights = true
        }
        tile.last_selection_result = TileSelectionResult::CullButNeeded;
    } else if tile_layer.preloadSiblings && tile.level == 0 {
        queueTileLoad(tile, &mut enqueue_evt, tile_layer_loader::QueueType::Low);

        tile.last_selection_result = TileSelectionResult::CULLED;
    } else {
        tile.last_selection_result = TileSelectionResult::CULLED;
    }
    tile.last_selection_result_frame = frame_count.0;
}
fn queueTileLoad(
    tile: &QuadtreeTile,
    &mut enqueue_evt: EventWriter<EnQueue>,
    queue_type: tile_layer_loader::QueueType,
) {
    enqueue_evt.send(tile_layer_loader::EnQueue {
        tile,
        queue_tyoe: queue_type,
    });
}
// fn visitTile(
//     tile_layer:&QuadTreeTileLayer,
//     datasource: &dyn DataSource,
//      tile: &QuadtreeTile,
//       globe_camera_control: &GlobeCameraControl,
//       ancestorMeetsSse:bool,
//     mut enqueue_evt: EventWriter<EnQueue>,
//     frame_count: &Res<FrameCount>,

// ) {

//       let meetsSse =
//       screen_space_error(datasource,tile  ,globe_camera_control) <
//       tile_layer.maximumScreenSpaceError;

//     let southwestChild = tile.southwestChild;
//     let southeastChild = tile.southeastChild;
//     let northwestChild = tile.northwestChild;
//     let northeastChild = tile.northeastChild;

//     let lastFrame = tile_layer._lastSelectionFrameNumber;
//     let lastFrameSelectionResult ={
//         if tile.last_selection_result_frame == lastFrame{
//             tile.last_selection_result
//         }else{
//             TileSelectionResult::NONE
//         }
//     }

//     let datasource = tile_layer.datasource;

//     if meetsSse || ancestorMeetsSse {
//       // This tile (or an ancestor) is the one we want to render this frame, but we'll do different things depending
//       // on the state of this tile and on what we did _last_ frame.

//       // We can render it if _any_ of the following are true:
//       // 1. We rendered it (or kicked it) last frame.
//       // 2. This tile was culled last frame, or it wasn't even visited because an ancestor was culled.
//       // 3. The tile is completely done loading.
//       // 4. a) Terrain is ready, and
//       //    b) All necessary imagery is ready. Necessary imagery is imagery that was rendered with this tile
//       //       or any descendants last frame. Such imagery is required because rendering this tile without
//       //       it would cause detail to disappear.
//       //
//       // Determining condition 4 is more expensive, so we check the others first.
//       //
//       // Note that even if we decide to render a tile here, it may later get "kicked" in favor of an ancestor.

//       let one_rendered_last_frame =
//         TileSelectionResult::originalResult(lastFrameSelectionResult) ==
//         TileSelectionResult::RENDERED;
//       let two_culled_or_not_visited =
//         TileSelectionResult::originalResult(lastFrameSelectionResult) ==
//           TileSelectionResult::CULLED ||
//         lastFrameSelectionResult == TileSelectionResult::NONE;
//       let three_completely_loaded = tile.state == QuadtreeTileLoadState::DONE;

//       let renderable =
//         one_rendered_last_frame || two_culled_or_not_visited || three_completely_loaded;

//       if !renderable {
//         // Check the more expensive condition 4 above. This requires details of the thing
//         // we're rendering (e.g. the globe surface), so delegate it to the tile provider.
//           renderable = datasource.canRenderWithoutLosingDetail(tile);
//       }

//       if renderable {
//         // Only load this tile if it (not just an ancestor) meets the SSE.
//         if meetsSse {
//         queueTileLoad(tile, &mut enqueue_evt, tile_layer_loader::QueueType::Medium);

//         }
//         //         tile_layer._tilesToRender.push(tile);

//         traversalDetails.all_are_renderable = tile.renderable;
//         traversalDetails.any_were_rendered_last_frame =
//           lastFrameSelectionResult == TileSelectionResult::RENDERED;
//         traversalDetails.not_yet_renderable_count =  {
//           if tile.renderable{
//             0
//           }else{
//             1
//           }
//         } ;

//         tile.last_selection_result_frame = frame_count.0;
//         tile.last_selection_result = TileSelectionResult::RENDERED;

//         if !traversalDetails.any_were_rendered_last_frame {
//           // Tile is newly-rendered this frame, so update its heights.
//           tile_layer._tileToUpdateHeights.push(tile);
//         }

//         return;
//       }

//       // Otherwise, we can't render this tile (or its fill) because doing so would cause detail to disappear
//       // that was visible last frame. Instead, keep rendering any still-visible descendants that were rendered
//       // last frame and render fills for newly-visible descendants. E.g. if we were rendering level 15 last
//       // frame but this frame we want level 14 and the closest renderable level <= 14 is 0, rendering level
//       // zero would be pretty jarring so instead we keep rendering level 15 even though its SSE is better
//       // than required. So fall through to continue traversal...
//       ancestorMeetsSse = true;

//       // Load this blocker tile with high priority, but only if this tile (not just an ancestor) meets the SSE.
//       if meetsSse {
//         queueTileLoad(tile, &mut enqueue_evt, tile_layer_loader::QueueType::High);

//       }
//     }

//     if datasource.can_refine(tile) {
//       let all_are_upsampled =
//         southwestChild.upsampled_from_parent &&
//         southeastChild.upsampled_from_parent &&
//         northwestChild.upsampled_from_parent &&
//         northeastChild.upsampled_from_parent;

//       if all_are_upsampled {
//         // No point in rendering the children because they're all upsampled.  Render this tile instead.
//         tile_layer._tilesToRender.push(tile);

//         // Rendered tile that's not waiting on children loads with medium priority.
//         queueTileLoad(tile, &mut enqueue_evt, tile_layer_loader::QueueType::Medium);

//         // Make sure we don't unload the children and forget they're upsampled.
//         tile_layer._tileReplacementQueue.mark_tile_rendered(southwestChild);
//         tile_layer._tileReplacementQueue.mark_tile_rendered(southeastChild);
//         tile_layer._tileReplacementQueue.mark_tile_rendered(northwestChild);
//         tile_layer._tileReplacementQueue.mark_tile_rendered(northeastChild);

//         traversalDetails.all_are_renderable = tile.renderable;
//         traversalDetails.any_were_rendered_last_frame =
//           lastFrameSelectionResult == TileSelectionResult::RENDERED;
//         traversalDetails.not_yet_renderable_count =  {
//           if tile.renderable{
//             0
//           }else{
//             1
//           }
//         } ;

//         tile.last_selection_result_frame = frame_count.0;
//         tile.last_selection_result = TileSelectionResult::RENDERED;

//         if !traversalDetails.any_were_rendered_last_frame {
//           // Tile is newly-rendered this frame, so update its heights.
//           tile_layer._tileToUpdateHeights.push(tile);
//         }

//         return;
//       }

//       // SSE is not good enough, so refine.
//       tile.last_selection_result_frame = frame_count.0;
//       tile.last_selection_result = TileSelectionResult::REFINED;

//       let first_rendered_descendant_index = tile_layer._tilesToRender.length;
//       let load_index_low = tile_layer._tileLoadQueueLow.length;
//       let load_index_medium = tile_layer._tileLoadQueueMedium.length;
//       let load_index_high = tile_layer._tileLoadQueueHigh.length;
//       let tiles_to_update_heights_index = tile_layer._tileToUpdateHeights.length;

//       // No need to add the children to the load queue because they'll be added (if necessary) when they're visited.
//       visit_visible_children_near_to_far(
//         tile_layer,
//         southwestChild,
//         southeastChild,
//         northwestChild,
//         northeastChild,
//         frameState,
//         ancestorMeetsSse,
//         traversalDetails
//       );

//       // If no descendant tiles were added to the render list by the function above, it means they were all
//       // culled even though this tile was deemed visible. That's pretty common.

//       if first_rendered_descendant_index != tile_layer._tilesToRender.length {
//         // At least one descendant tile was added to the render list.
//         // The traversalDetails tell us what happened while visiting the children.

//         let all_are_renderable = traversalDetails.all_are_renderable;
//         let any_were_rendered_last_frame =
//           traversalDetails.any_were_rendered_last_frame;
//         let not_yet_renderable_count = traversalDetails.not_yet_renderable_count;
//         let queued_for_load = false;

//         if !all_are_renderable && !any_were_rendered_last_frame {
//           // Some of our descendants aren't ready to render yet, and none were rendered last frame,
//           // so kick them all out of the render list and render this tile instead. Continue to load them though!

//           // Mark the rendered descendants and their ancestors - up to this tile - as kicked.
//           let renderList = tile_layer._tilesToRender;
//           for i in first_rendered_descendant_index..renderList(){

//             let workTile = renderList[i];
//             while (
//               workTile != undefined &&
//               workTile.last_selection_result != TileSelectionResult::KICKED &&
//               workTile != tile
//             ) {
//               workTile.last_selection_result = TileSelectionResult::kick(
//                 workTile.last_selection_result
//               );
//               workTile = workTile.parent;
//             }
//           }

//           // Remove all descendants from the render list and add this tile.
//           tile_layer._tilesToRender.length = first_rendered_descendant_index;
//           tile_layer._tileToUpdateHeights.length = tiles_to_update_heights_index;
//                   tile_layer._tilesToRender.push(tile);

//           tile.last_selection_result = TileSelectionResult::RENDERED;

//           // If we're waiting on heaps of descendants, the above will take too long. So in that case,
//           // load this tile INSTEAD of loading any of the descendants, and tell the up-level we're only waiting
//           // on this tile. Keep doing this until we actually manage to render this tile.
//           let was_rendered_last_frame =
//             lastFrameSelectionResult == TileSelectionResult::RENDERED;
//           if (
//             !was_rendered_last_frame &&
//             not_yet_renderable_count > tile_layer.loadingDescendantLimit
//           ) {
//             // Remove all descendants from the load queues.
//             tile_layer._tileLoadQueueLow.length = load_index_low;
//             tile_layer._tileLoadQueueMedium.length = load_index_medium;
//             tile_layer._tileLoadQueueHigh.length = load_index_high;
//         // queueTileLoad(tile_layer.datasource as &dyn DataSource, &mut tile_layer._tileLoadQueueMedium, tile, globe_camera_control,&mut enqueue_evt);
//         queueTileLoad(tile, &mut enqueue_evt, tile_layer_loader::QueueType::Medium);

//             traversalDetails.not_yet_renderable_count = {
//               if tile.renderable{
//                 0
//               }else{
//                 1
//               }
//             } ;
//             queued_for_load = true;
//           }

//           traversalDetails.all_are_renderable = tile.renderable;
//           traversalDetails.any_were_rendered_last_frame = was_rendered_last_frame;

//           if !was_rendered_last_frame {
//             // Tile is newly-rendered this frame, so update its heights.
//             tile_layer._tileToUpdateHeights.push(tile);
//           }

//           ++debug.tilesWaitingForChildren;
//         }

//         if tile_layer.preloadAncestors && !queued_for_load {
//         // queueTileLoad(tile_layer.datasource as &dyn DataSource, &mut tile_layer._tileLoadQueueLow, tile, globe_camera_control,&mut enqueue_evt);
//         queueTileLoad(tile, &mut enqueue_evt, tile_layer_loader::QueueType::Low);

//         }
//       }

//       return;
//     }

//     *tile.last_selection_result_frame = frame_count.0;
//     *tile.last_selection_result = TileSelectionResult::RENDERED;

//     // We'd like to refine but can't because we have no availability data for this tile's children,
//     // so we have no idea if refinining would involve a load or an upsample. We'll have to finish
//     // loading this tile first in order to find that out, so load this refinement blocker with
//     // high priority.
//             tile_layer._tilesToRender.push(tile);
//     // queueTileLoad(tile_layer.datasource as &dyn DataSource, &mut tile_layer._tileLoadQueueHigh, tile, globe_camera_control,&mut enqueue_evt);
//     queueTileLoad(tile, &mut enqueue_evt, tile_layer_loader::QueueType::High);

//     traversalDetails.all_are_renderable = tile.renderable;
//     traversalDetails.any_were_rendered_last_frame =
//       lastFrameSelectionResult == TileSelectionResult::RENDERED;
//     traversalDetails.not_yet_renderable_count =  {
//       if tile.renderable{
//         0
//       }else{
//         1
//       }
//     } ;
// }

fn screen_space_error(
    datasource: &dyn DataSource,
    tile: &QuadtreeTileValue,
    globe_camera_control: &GlobeCameraControl,
) {
    let max_geometric_error = datasource.getLevelMaximumGeometricError(tile.level);

    let distance = tile._distance;
    let height = globe_camera_control.drawingBufferHeight;
    let sse_denominator = globe_camera_control.sse_denominator;

    let error = (max_geometric_error * height) / (distance * sse_denominator);

    error /= globe_camera_control.pixelRatio;

    return error;
}
fn contains_needed_position(
    rectangle: &Rectangle,
    cameraPositionCartographic: Option<&Cartographic>,
) -> bool {
    return cameraPositionCartographic.is_some()
        && rectangle.contains(&cameraPositionCartographic.unwrap());
}
// fn tile_state_system(
//     mut tile_layer: ResMut<QuadTreeTileLayer>,
//     mut query: Query<&mut Tile>,
//     mut job_spawner: JobSpawner,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut images: ResMut<Assets<Image>>,
//     mut terrain_materials: ResMut<Assets<TerrainMeshMaterial>>,
//     mut standard_materials: ResMut<Assets<StandardMaterial>>,
//     asset_server: Res<AssetServer>,
// ) {
//     for mut tile in &mut query {
//         match tile.state {
//             TileState::START => {
//                 job_spawner.spawn(CreateTileJob {
//                     x: tile.x,
//                     y: tile.y,
//                     level: tile.level,
//                     width: tile.width,
//                     height: tile.height,
//                 });
//                 tile.state = TileState::LOADING;
//             }
//             TileState::LOADING => {}
//             TileState::READY => {
//                 let terrain_mesh = tile.terrain_mesh.as_ref().unwrap();
//                 let mesh = meshes.add(terrain_mesh.into());
//                 let mut rng = rand::thread_rng();
//                 let r: f32 = rng.gen();
//                 let g: f32 = rng.gen();
//                 let b: f32 = rng.gen();
//                 commands.spawn((
//                     MaterialMeshBundle {
//                         mesh: mesh,
//                         material: terrain_materials.add(TerrainMeshMaterial {
//                             color: Color::rgba(r, g, b, 1.0),
//                             image: Some(asset_server.load("icon.png")),
//                             // image: asset_server.load(format!("https://t5.tianditu.gov.cn/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=vec&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={}&TILEROW={}&TILEMATRIX={}&tk=b931d6faa76fc3fbe622bddd6522e57b",x,y,level)),
//                             // image: asset_server.load(format!("tile/{}/{}/{}.png", level, y, x,)),
//                             // image:Some( asset_server.load(format!(
//                             //     "https://maps.omniscale.net/v2/houtu-b8084b0b/style.default/{}/{}/{}.png",
//                             //     tile.level, tile.x, tile.y,
//                             // ))),
//                             // image: None,
//                         }),
//                         // material: standard_materials.add(Color::rgba(r, g, b, 1.0).into()),
//                         ..Default::default()
//                     },
//                     TileKey::new(tile.y, tile.x, tile.level),
//                     // TileState::START,
//                 ));
//             }
//             _ => {}
//         }
//     }
// }
// fn compareDistanceToPoint(a: &Tile, b: &Tile, camera_position: &Cartographic) -> f64 {
//     let mut center = a.rectangle.center();
//     let alon = center.longitude - camera_position.longitude;
//     let alat = center.latitude - camera_position.latitude;

//     center = b.rectangle.center();
//     let blon = center.longitude - camera_position.longitude;
//     let blat = center.latitude - camera_position.latitude;

//     return alon * alon + alat * alat - (blon * blon + blat * blat);
// }
pub fn get_zoom_level(h: f64) -> u32 {
    if h <= 100. {
        //0.01
        return 19;
    } else if h <= 300. {
        //0.02
        return 18;
    } else if h <= 660. {
        //0.05
        return 17;
    } else if h <= 1300. {
        //0.1
        return 16;
    } else if h <= 2600. {
        //0.2
        return 15;
    } else if h <= 6400. {
        //0.5
        return 14;
    } else if h <= 13200. {
        //1
        return 13;
    } else if h <= 26000. {
        //2
        return 12;
    } else if h <= 67985. {
        //5
        return 11;
    } else if h <= 139780. {
        //10
        return 10;
    } else if h <= 250600. {
        //20
        return 9;
    } else if h <= 380000. {
        //30
        return 8;
    } else if h <= 640000. {
        //50
        return 7;
    } else if h <= 1280000. {
        //100
        return 6;
    } else if h <= 2600000. {
        //200
        return 5;
    } else if h <= 6100000. {
        //500
        return 4;
    } else if h <= 11900000. {
        //1000
        return 3;
    } else {
        return 2;
    }
}
struct TraversalDetails {
    all_are_renderable: bool,
    any_were_rendered_last_frame: bool,
    not_yet_renderable_count: u32,
}
impl Default for TraversalDetails {
    fn default() -> Self {
        Self {
            all_are_renderable: true,
            any_were_rendered_last_frame: false,
            not_yet_renderable_count: 0,
        }
    }
}
struct TraversalQuadDetails {
    pub southwest: TraversalDetails,
    pub southeast: TraversalDetails,
    pub northwest: TraversalDetails,
    pub northeast: TraversalDetails,
}
impl TraversalQuadDetails {
    fn combine(&self) -> TraversalDetails {
        let southwest = self.southwest;
        let southeast = self.southeast;
        let northwest = self.northwest;
        let northeast = self.northeast;
        let mut result = TraversalDetails::default();
        result.all_are_renderable = southwest.all_are_renderable
            && southeast.all_are_renderable
            && northwest.all_are_renderable
            && northeast.all_are_renderable;
        result.any_were_rendered_last_frame = southwest.any_were_rendered_last_frame
            || southeast.any_were_rendered_last_frame
            || northwest.any_were_rendered_last_frame
            || northeast.any_were_rendered_last_frame;
        result.not_yet_renderable_count = southwest.not_yet_renderable_count
            + southeast.not_yet_renderable_count
            + northwest.not_yet_renderable_count
            + northeast.not_yet_renderable_count;
        return result;
    }
}
lazy_static! {
    static ref traversalQuadsByLevel: Vec<TraversalDetails> = vec![TraversalDetails::default(); 31];
}
// fn visit_visible_children_near_to_far<P: Projection>(
//     southwest: &QuadtreeTile,
//     southeast: &QuadtreeTile,
//     northwest: &QuadtreeTile,
//     northeast: &QuadtreeTile,
//     datasource: &dyn DataSource,
//     cameraPosition: &Cartographic,
//     occluders: &EllipsoidalOccluder,
// ) {
//     let quadDetails = traversalQuadsByLevel[southwest.level];

//     let southwestDetails = quadDetails.southwest;
//     let southeastDetails = quadDetails.southeast;
//     let northwestDetails = quadDetails.northwest;
//     let northeastDetails = quadDetails.northeast;

//     if cameraPosition.longitude < southwest.rectangle.east {
//         if cameraPosition.latitude < southwest.rectangle.north {
//             // Camera in southwest quadrant
//             visit_if_visible(
//                 primitive,
//                 southwest,
//                 tileProvider,
//                 frameState,
//                 occluders,
//                 ancestorMeetsSse,
//                 southwestDetails,
//             );
//             visit_if_visible(
//                 primitive,
//                 southeast,
//                 tileProvider,
//                 frameState,
//                 occluders,
//                 ancestorMeetsSse,
//                 southeastDetails,
//             );
//             visit_if_visible(
//                 primitive,
//                 northwest,
//                 tileProvider,
//                 frameState,
//                 occluders,
//                 ancestorMeetsSse,
//                 northwestDetails,
//             );
//             visit_if_visible(
//                 primitive,
//                 northeast,
//                 tileProvider,
//                 frameState,
//                 occluders,
//                 ancestorMeetsSse,
//                 northeastDetails,
//             );
//         } else {
//             // Camera in northwest quadrant
//             visit_if_visible(
//                 primitive,
//                 northwest,
//                 tileProvider,
//                 frameState,
//                 occluders,
//                 ancestorMeetsSse,
//                 northwestDetails,
//             );
//             visit_if_visible(
//                 primitive,
//                 southwest,
//                 tileProvider,
//                 frameState,
//                 occluders,
//                 ancestorMeetsSse,
//                 southwestDetails,
//             );
//             visit_if_visible(
//                 primitive,
//                 northeast,
//                 tileProvider,
//                 frameState,
//                 occluders,
//                 ancestorMeetsSse,
//                 northeastDetails,
//             );
//             visit_if_visible(
//                 primitive,
//                 southeast,
//                 tileProvider,
//                 frameState,
//                 occluders,
//                 ancestorMeetsSse,
//                 southeastDetails,
//             );
//         }
//     } else if cameraPosition.latitude < southwest.rectangle.north {
//         // Camera southeast quadrant
//         visit_if_visible(
//             primitive,
//             southeast,
//             tileProvider,
//             frameState,
//             occluders,
//             ancestorMeetsSse,
//             southeastDetails,
//         );
//         visit_if_visible(
//             primitive,
//             southwest,
//             tileProvider,
//             frameState,
//             occluders,
//             ancestorMeetsSse,
//             southwestDetails,
//         );
//         visit_if_visible(
//             primitive,
//             northeast,
//             tileProvider,
//             frameState,
//             occluders,
//             ancestorMeetsSse,
//             northeastDetails,
//         );
//         visit_if_visible(
//             primitive,
//             northwest,
//             tileProvider,
//             frameState,
//             occluders,
//             ancestorMeetsSse,
//             northwestDetails,
//         );
//     } else {
//         // Camera in northeast quadrant
//         visit_if_visible(
//             primitive,
//             northeast,
//             tileProvider,
//             frameState,
//             occluders,
//             ancestorMeetsSse,
//             northeastDetails,
//         );
//         visit_if_visible(
//             primitive,
//             northwest,
//             tileProvider,
//             frameState,
//             occluders,
//             ancestorMeetsSse,
//             northwestDetails,
//         );
//         visit_if_visible(
//             primitive,
//             southeast,
//             tileProvider,
//             frameState,
//             occluders,
//             ancestorMeetsSse,
//             southeastDetails,
//         );
//         visit_if_visible(
//             primitive,
//             southwest,
//             tileProvider,
//             frameState,
//             occluders,
//             ancestorMeetsSse,
//             southwestDetails,
//         );
//     }

//     quadDetails.combine(traversalDetails);
// }
