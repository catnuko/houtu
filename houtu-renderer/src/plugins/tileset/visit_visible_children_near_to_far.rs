use std::sync::{Arc, Mutex};

use bevy::core::FrameCount;

use bevy::prelude::*;
use houtu_scene::{
    Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme, Rectangle, TerrainExaggeration,
    TilingScheme,
};

use crate::plugins::camera::GlobeCamera;

use super::globe_surface_tile::{computeTileVisibility, TileVisibility};
use super::terrain_datasource::{TerrainDataSource, TerrainDataSourceData};

use super::tile_quad_tree::{visitTile, GlobeSurfaceTileQuery, TileQuadTree};
use super::tile_selection_result::TileSelectionResult;
use super::traversal_details::{
    get_traversal_details, AllTraversalQuadDetails, RootTraversalDetails, TraversalDetails,
};

use super::TileKey;

use super::quadtree_tile::{
    NodeChildren, Quadrant, QuadtreeTile, QuadtreeTileParent, TileLoadHigh, TileLoadLow,
    TileLoadMedium, TileNode, TileToRender, TileToUpdateHeight,
};
use super::tile_replacement_queue::TileReplacementState;
pub fn visit_visible_children_near_to_far(
    commands: &mut Commands,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
    ellipsoid: &Ellipsoid,
    ellipsoidal_occluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    terrain_datasource: &mut TerrainDataSource,
    ancestor_meets_sse: &mut bool,
    queue_params_set: &mut ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
    )>,

    all_traversal_quad_details: &mut ResMut<AllTraversalQuadDetails>,
    root_traversal_details: &mut ResMut<RootTraversalDetails>,
    southwest: &TileNode,
    southeast: &TileNode,
    northwest: &TileNode,
    northeast: &TileNode,
    quadtree_tile_entity: Entity,
) {
    let get_tile_ndoe_entity = |node_id: &TileNode| -> Option<Entity> {
        if let TileNode::Internal(v) = node_id {
            Some(v.clone())
        } else {
            None
        }
    };

    let southwest_entity = get_tile_ndoe_entity(southwest).expect("data不存在");
    let southeast_entity = get_tile_ndoe_entity(southeast).expect("data不存在");
    let northwest_entity = get_tile_ndoe_entity(northwest).expect("data不存在");
    let northeast_entity = get_tile_ndoe_entity(northeast).expect("data不存在");
    let (east, west, south, north, level) = {
        let v = quadtree_tile_query.get(southwest_entity).unwrap();
        (v.2.east, v.2.west, v.2.south, v.2.north, v.5.level)
    };

    let cameraPositionCartographic = globe_camera.get_position_cartographic();
    if cameraPositionCartographic.longitude < east {
        if cameraPositionCartographic.latitude < north {
            // Camera in southwest quadrant
            visit_if_visible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidal_occluder,
                quadtree_tile_query,
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
                ancestor_meets_sse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                southwest_entity,
            );
            visit_if_visible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidal_occluder,
                quadtree_tile_query,
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
                ancestor_meets_sse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                southeast_entity,
            );
            visit_if_visible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidal_occluder,
                quadtree_tile_query,
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
                ancestor_meets_sse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                northwest_entity,
            );
            visit_if_visible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidal_occluder,
                quadtree_tile_query,
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
                ancestor_meets_sse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                northeast_entity,
            );
        } else {
            // Camera in northwest quadrant
            visit_if_visible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidal_occluder,
                quadtree_tile_query,
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
                ancestor_meets_sse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                northwest_entity,
            );
            visit_if_visible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidal_occluder,
                quadtree_tile_query,
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
                ancestor_meets_sse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                southwest_entity,
            );
            visit_if_visible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidal_occluder,
                quadtree_tile_query,
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
                ancestor_meets_sse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                northeast_entity,
            );
            visit_if_visible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidal_occluder,
                quadtree_tile_query,
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
                ancestor_meets_sse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                southeast_entity,
            );
        }
    } else if cameraPositionCartographic.latitude < north {
        // Camera southeast quadrant
        visit_if_visible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            southeast_entity,
        );
        visit_if_visible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            southwest_entity,
        );
        visit_if_visible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            northeast_entity,
        );
        visit_if_visible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            northwest_entity,
        );
    } else {
        // Camera in northeast quadrant
        visit_if_visible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            northeast_entity,
        );
        visit_if_visible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            northwest_entity,
        );
        visit_if_visible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            southeast_entity,
        );
        visit_if_visible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            southwest_entity,
        );
    }
    let (_, _, _, _, _, key, _, _, _, location, _, terrain_datasource_data) =
        quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    let quadDetailsCombine = { all_traversal_quad_details.get_mut(level).combine() };
    let traversal_details = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        location,
        key,
    );
    *traversal_details = quadDetailsCombine;
}
pub fn visit_if_visible(
    commands: &mut Commands,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
    ellipsoid: &Ellipsoid,
    ellipsoidal_occluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    terrain_datasource: &mut TerrainDataSource,
    ancestor_meets_sse: &mut bool,
    all_traversal_quad_details: &mut ResMut<AllTraversalQuadDetails>,
    root_traversal_details: &mut ResMut<RootTraversalDetails>,
    queue_params_set: &mut ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
    )>,
    quadtree_tile_entity: Entity,
) {
    // info!("visit_if_visible entity={:?}", quadtree_tile_entity);
    if computeTileVisibility(
        // commands,
        // ellipsoid,
        ellipsoidal_occluder,
        quadtree_tile_query,
        globe_camera,
        quadtree_tile_entity,
    ) != TileVisibility::NONE
    {
        return visitTile(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            quadtree_tile_entity,
        );
    }
    tile_quad_tree.debug.tiles_culled += 1;
    tile_quad_tree
        .replacement_queue
        .mark_tile_rendered(quadtree_tile_query, quadtree_tile_entity);
    let (
        entity,
        globe_surface_tile,
        rectangle,
        mut other_state,
        mut replacement_state,
        key,
        node_id,
        node_children,
        state,
        location,
        parent,
        terrain_datasource_data,
    ) = quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    let mut entity_mut = commands.entity(entity);

    let traversal_details = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        location,
        key,
    );
    traversal_details.all_are_renderable = true;
    traversal_details.any_were_rendered_last_frame = false;
    traversal_details.not_yet_renderable_count = 0;
    if contains_needed_position(&rectangle, tile_quad_tree) {
        // if data.0.is_none() || data.0.as_ref().unwrap().vertex_array.is_none() {
        //     entity_mut.insert(TileLoadMedium);
        // }

        // let last_frame = &tile_quad_tree.last_selection_frame_number;
        // let last_frame_selection_result = if other_state.last_selection_result_frame == *last_frame {
        //     &other_state.last_selection_result
        // } else {
        //     &TileSelectionResult::NONE
        // };
        // if *last_frame_selection_result != TileSelectionResult::CULLED_BUT_NEEDED
        //     && *last_frame_selection_result != TileSelectionResult::RENDERED
        // {
        //     // tile_quad_tree._tileToUpdateHeights.push(tile);
        //     entity_mut.insert(TileToUpdateHeight);
        // }

        // other_state.last_selection_result = TileSelectionResult::CULLED_BUT_NEEDED;
    } else if tile_quad_tree.preload_siblings || key.level == 0 {
        // Load culled level zero tiles with low priority.
        // For all other levels, only load culled tiles if preload_siblings is enabled.
        entity_mut.insert(TileLoadLow);
        other_state.last_selection_result = TileSelectionResult::CULLED;
    } else {
        other_state.last_selection_result = TileSelectionResult::CULLED;
    }

    other_state.last_selection_result_frame = Some(frame_count.0);
}
fn contains_needed_position(
    rectangle: &Rectangle,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
) -> bool {
    return tile_quad_tree.camera_position_cartographic.is_some()
        && rectangle.contains(&tile_quad_tree.camera_position_cartographic.unwrap())
        || tile_quad_tree
            .camera_reference_frame_origin_cartographic
            .is_some()
            && rectangle.contains(
                &tile_quad_tree
                    .camera_reference_frame_origin_cartographic
                    .unwrap(),
            );
}

pub fn make_new_quadtree_tile(
    commands: &mut Commands,
    key: TileKey,
    rectangle: Rectangle,
    location: Quadrant,
    parent: QuadtreeTileParent,
) -> TileNode {
    let mut entity_mut = commands.spawn((
        QuadtreeTile::new(key, rectangle, location, parent),
        TerrainDataSourceData::default(),
    ));
    let entity = entity_mut.id();
    let node_id = TileNode::Internal(entity);
    entity_mut.insert((TileReplacementState::new(entity), node_id.clone()));
    return node_id;
}
pub fn subdivide(
    commands: &mut Commands,
    node_id: &TileNode,
    key: &TileKey,
    children: &mut NodeChildren,
    terrain_datasource: &mut TerrainDataSource,
) {
    if let TileNode::Internal(v) = children.southeast {
        return;
    }
    if let TileNode::Internal(index) = node_id {
        let southwest = key.southwest();
        let southwest_rectangle = terrain_datasource.tiling_scheme.tile_x_y_to_rectange(
            southwest.x,
            southwest.y,
            southwest.level,
        );
        let southeast = key.southeast();
        let southeast_rectangle = terrain_datasource.tiling_scheme.tile_x_y_to_rectange(
            southeast.x,
            southeast.y,
            southeast.level,
        );
        let northwest = key.northwest();
        let northwest_rectangle = terrain_datasource.tiling_scheme.tile_x_y_to_rectange(
            northwest.x,
            northwest.y,
            northwest.level,
        );
        let northeast = key.northeast();
        let northeast_rectangle = terrain_datasource.tiling_scheme.tile_x_y_to_rectange(
            northeast.x,
            northeast.y,
            northeast.level,
        );
        let sw = make_new_quadtree_tile(
            commands,
            southwest,
            southwest_rectangle,
            Quadrant::Southwest,
            QuadtreeTileParent(node_id.clone()),
        );
        let se = make_new_quadtree_tile(
            commands,
            southeast,
            southeast_rectangle,
            Quadrant::Southeast,
            QuadtreeTileParent(node_id.clone()),
        );
        let nw = make_new_quadtree_tile(
            commands,
            northwest,
            northwest_rectangle,
            Quadrant::Northwest,
            QuadtreeTileParent(node_id.clone()),
        );
        let ne = make_new_quadtree_tile(
            commands,
            northeast,
            northeast_rectangle,
            Quadrant::Northeast,
            QuadtreeTileParent(node_id.clone()),
        );
        children.northwest = nw;
        children.northeast = ne;
        children.southwest = sw;
        children.southeast = se;
    }
}
