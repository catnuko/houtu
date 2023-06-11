use amethyst::{
    assets::{Asset, Handle},
    core::ecs::{Entity, prelude::DenseVecStorage},
    renderer::types::Texture,
};

/// Terrain asset type
/// Stores relevant information about a single terrain asset
// TODO: Doc
#[derive(Debug, Clone, PartialEq)]
pub struct Terrain {
    /// Size of the terrain using a 1:1 mapping to pixel in our textures
    pub size: u32,
    /// Scaling factor for the stored heightmap values
    // TODO: Further investigate how to derive realistic values
    pub height_scale: f32,
    ///  Offset for the terrain
    // TODO: remove this in favor of Transforms
    pub height_offset: f32,
    /// Maximum level of depth in the underlaying quadtree
    pub max_level: u8,
    /// Heightmap handle
    pub heightmap: Handle<Texture>,
    /// Normalmap handle
    pub normal: Handle<Texture>,
    /// Albedo texture handle
    pub albedo: Handle<Texture>,
}

impl Asset for Terrain {
    const NAME: &'static str = "amethyst_terrain::Terrain";
    type Data = Self;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}


/// Trait providing generic access to a collection of texture handles
/// TODO: Avoid setting up our own StaticTextureSet
pub trait StaticTextureSet<'a>:
    Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static
{
    /// Iterator type to access this texture sets handles
    type Iter: Iterator<Item = &'a Handle<Texture>>;

    /// Returns an iterator to the textures associated with a given terrain.
    fn textures(mat: &'a Terrain) -> Self::Iter;

    /// ALWAYS RETURNS 1
    fn len() -> usize {
        1
    }
}


macro_rules! impl_texture {
    ($name:ident, $prop:ident) => {
        #[doc = "Macro Generated Texture Type"]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name;
        impl<'a> StaticTextureSet<'a> for $name {
            type Iter = std::iter::Once<&'a Handle<Texture>>;
            #[inline(always)]
            fn textures(terrain: &'a Terrain) -> Self::Iter {
                std::iter::once(&terrain.$prop)
            }
        }
    };
}

impl_texture!(TexHeightmap, heightmap);
impl_texture!(TexNormal, normal);
impl_texture!(TexAlbedo, albedo);

macro_rules! recursive_iter {
    (@value $first:expr, $($rest:expr),*) => { $first.chain(recursive_iter!(@value $($rest),*)) };
    (@value $last:expr) => { $last };
    (@type $first:ty, $($rest:ty),*) => { std::iter::Chain<$first, recursive_iter!(@type $($rest),*)> };
    (@type $last:ty) => { $last };
}

macro_rules! impl_texture_set_tuple {
    ($($from:ident),*) => {
        impl<'a, $($from,)*> StaticTextureSet<'a> for ($($from),*,)
        where
            $($from: StaticTextureSet<'a>),*,
        {
            type Iter = recursive_iter!(@type $($from::Iter),*);
            #[inline(always)]
            fn textures(terrain: &'a Terrain) -> Self::Iter {
                recursive_iter!(@value $($from::textures(terrain)),*)
            }
            fn len() -> usize {
                $($from::len() + )* 0
            }
        }
    }
}

impl_texture_set_tuple!(A, B, C);


/// Active clipmap resource, used by the renderer to choose which camera to get the view matrix from.
/// If no active camera is found, the first camera will be used as a fallback.
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveTerrain {
    /// Camera entity
    pub entity: Entity,
}