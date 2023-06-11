//! Prefab for Terrain
use amethyst::{
    assets::{AssetStorage, Handle, Loader, PrefabData, ProgressCounter},
    core::ecs::{Entity, Read, ReadExpect, WriteStorage},
    error::Error,
    renderer::{
        formats::texture::TexturePrefab,
        mtl::{MaterialDefaults},
        types::Texture,
    }
};
use serde::{Deserialize, Serialize};
use crate::Terrain;

/// PrefabData for loading ´Terrain´s
#[derive(Debug, Deserialize, Serialize)]
#[serde(default, bound = "")]
pub struct TerrainPrefab {
    pub size: u32,
    pub max_level: u8,
    pub height_scale: f32,
    pub height_offset: f32,
    pub heightmap: Option<TexturePrefab>,
    pub normal: Option<TexturePrefab>,
    pub albedo: Option<TexturePrefab>,
    #[serde(skip)]
    handle: Option<Handle<Terrain>>,
}

impl TerrainPrefab {
    /// Clone the loaded terrain prefab to a new instance.
    pub fn clone_loaded(&self) -> Self {
        assert!(self.handle.is_some());

        Self {
            handle: self.handle.clone(),
            ..Self::default()
        }
    }
}

impl Default for TerrainPrefab {
    fn default() -> Self {
        TerrainPrefab {
            size: 128,
            max_level: 4,
            height_scale: 1.0,
            height_offset: 0.0,
            heightmap: None,
            normal: None,
            albedo: None,
            handle: None,
        }
    }
}

fn load_handle(prefab: &Option<TexturePrefab>, def: &Handle<Texture>) -> Handle<Texture> {
    prefab
        .as_ref()
        .and_then(|tp| match tp {
            TexturePrefab::Handle(h) => Some(h.clone()),
            _ => None,
        })
        .unwrap_or_else(|| def.clone())
}

impl<'a> PrefabData<'a> for TerrainPrefab {
    type SystemData = (
        WriteStorage<'a, Handle<Terrain>>,
        ReadExpect<'a, MaterialDefaults>,
        <TexturePrefab as PrefabData<'a>>::SystemData,
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<Terrain>>,
    );
    type Result = (); 

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        let &mut (ref mut terrain, _, _, _, _) = system_data;
        terrain.insert(entity, self.handle.as_ref().unwrap().clone())?;
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let &mut (_, ref mat_default, ref mut tp_data, ref loader, ref storage) = system_data;
        let mut ret = false;
        if let Some(ref mut texture) = self.heightmap {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.normal {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.albedo {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        
        if self.handle.is_none() {
            let mtl = Terrain {
                size: self.size,
                max_level: self.max_level,
                height_offset: self.height_offset,
                height_scale: self.height_scale,
                // Todo: create default heightmap
                heightmap: load_handle(&self.heightmap, &mat_default.0.albedo),
                normal: load_handle(&self.normal, &mat_default.0.normal),
                albedo: load_handle(&self.albedo, &mat_default.0.albedo),
            };

            self.handle
                .replace(loader.load_from_data(mtl, progress, storage));
            ret = true;
        }
        Ok(ret)
    }
}

