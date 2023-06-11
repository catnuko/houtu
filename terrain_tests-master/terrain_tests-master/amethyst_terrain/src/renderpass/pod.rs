use amethyst::{
    core::{
        math::{convert, Matrix4}, Transform
    },
    renderer::{
        rendy::{
            mesh::{AsAttribute, AsVertex, VertexFormat},
            hal::format::Format
        }
    }
};
use glsl_layout::{self, *};
use crate::component::Terrain;
use cnquadtree::{TerrainQuadtree, TerrainQuadtreeLeaf, Direction, TerrainQuadtreeNode};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct PatchScale {
    pub patch_scale: float,
}
impl AsAttribute for PatchScale {
    const NAME: &'static str = "patch_scale";
    const FORMAT: Format = Format::R32Sfloat;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct PatchOrigin {
    pub patch_origin: vec3,
}
impl AsAttribute for PatchOrigin {
    const NAME: &'static str = "patch_origin";
    const FORMAT: Format = Format::Rgb32Sfloat;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct NeighbourScales {
    pub neighbour_scales: ivec4,
}
impl AsAttribute for NeighbourScales {
    const NAME: &'static str = "neighbour_scales";
    const FORMAT: Format = Format::Rgba32Sint;
}


#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub(crate) struct InstancedPatchArgs {
    pub patch_scale: float,
    pub patch_origin: vec3,
    pub neighbour_scales: ivec4,
}

impl InstancedPatchArgs {
    #[inline]
    pub fn from_object_data(patch: &TerrainQuadtreeLeaf, quadtree: &TerrainQuadtree) -> Self {
        let mut neighbour_scales : [i32; 4] = [64, 64, 64, 64];
        let neighbours = patch.get_neighbours();
        
        if neighbours[Direction::North] != TerrainQuadtreeNode::None && quadtree.get_level(neighbours[Direction::North]) < patch.level() {
            let diff = patch.level() - quadtree.get_level(neighbours[Direction::North]);
            neighbour_scales[0] = neighbour_scales[0] >> diff;
        } 
        if neighbours[Direction::East] != TerrainQuadtreeNode::None && quadtree.get_level(neighbours[Direction::East]) < patch.level() {
            let diff = patch.level() - quadtree.get_level(neighbours[Direction::East]);
            neighbour_scales[1] = neighbour_scales[1] >> diff;
        }
        if neighbours[Direction::South] != TerrainQuadtreeNode::None && quadtree.get_level(neighbours[Direction::South]) < patch.level() {
            let diff = patch.level() - quadtree.get_level(neighbours[Direction::South]);
            neighbour_scales[2] = neighbour_scales[2] >> diff;
        }
        if neighbours[Direction::West] != TerrainQuadtreeNode::None && quadtree.get_level(neighbours[Direction::West]) < patch.level() {
            let diff = patch.level() - quadtree.get_level(neighbours[Direction::West]);
            neighbour_scales[3] = neighbour_scales[3] >> diff;
        }
        InstancedPatchArgs {
            patch_scale: patch.half_extents()[0],
            patch_origin: [patch.origin()[0], 0., patch.origin()[1]].into(),
            neighbour_scales: neighbour_scales.into()
        }
    }
}

impl AsVertex for InstancedPatchArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((PatchScale::vertex(), PatchOrigin::vertex(), NeighbourScales::vertex()))
    }
}


// For all shader stages (binding = 0)
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(16))]
pub(crate) struct TerrainArgs {
    model: mat4,
    terrain_size: ivec2,
    terrain_height_scale: float,
    terrain_height_offset: float,
    wireframe: glsl_layout::boolean,
}
impl TerrainArgs {
    #[inline]
    pub fn from_object_data(transform: &Transform, terrain: &Terrain) -> Self {
        let model: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(*transform.global_matrix()).into();
        TerrainArgs {
            model: model.into(),
            terrain_size: [terrain.size as i32, terrain.size as i32].into(),
            terrain_height_scale: terrain.height_scale.into(),
            terrain_height_offset: terrain.height_offset.into(),
            wireframe: false.into(),
        }
    }
}