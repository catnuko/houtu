use std::collections::LinkedList;

use bevy::prelude::*;

use super::{
    imagery_layer_storage::ImageryLayerStorage, imagery_storage::ImageryStorage,
    quadtree_tile_storage::QuadtreeTileStorage, tile_key::TileKey,
};

pub struct TileReplacementQueue {
    list: LinkedList<TileKey>,
    last_before_start_of_frame: Option<TileKey>,
    count: usize,
    // storage: &'a mut QuadtreeTileStorage,
}
impl TileReplacementQueue {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
            last_before_start_of_frame: None,
            count: 0,
        }
    }
    pub fn clear(&mut self) {
        self.list.clear()
    }
    pub fn get_head(&self) -> Option<TileKey> {
        return self.list.front().and_then(|x| Some(x.clone()));
    }
    pub fn get_tail(&self) -> Option<TileKey> {
        return self.list.back().and_then(|x| Some(x.clone()));
    }
    pub fn get_count(&self) -> usize {
        return self.list.len();
    }
    pub fn mark_start_of_render_frame(&mut self) {
        let head = self.get_head();
        if let Some(v) = head {
            self.last_before_start_of_frame = Some(v.clone());
        } else {
            self.last_before_start_of_frame = None;
        }
    }
    pub fn trim_tiles(&mut self, storage: &mut QuadtreeTileStorage, maximum_tiles: u32) {
        // let mut tile_to_trim = self.get_tail();
        let mut keep_trimming = true;
        let mut count = self.count;
        while (keep_trimming
            // && self.last_before_start_of_frame.is_some()
            && count > maximum_tiles as usize)
            && self.get_tail().is_some()
        {
            // Stop trimming after we process the last tile not used in the
            // current frame.
            keep_trimming = self.get_tail() != self.last_before_start_of_frame;

            let tile_key = self.get_tail().unwrap().clone();
            let tile = storage.get(&tile_key).unwrap();
            let previous = tile.replacement_previous;

            if tile.eligible_for_unloading() {
                let entity = tile_key.clone();
                self.remove(storage, &entity);
            }
            if let Some(entity) = previous {
                if let Some(_v) = self.get_tail() {
                    // *v = entity;
                    self.list.pop_back();
                    self.list.push_back(entity);
                }
            }
            count = self.count;
        }
    }
    fn remove(&mut self, storage: &mut QuadtreeTileStorage, entity: &TileKey) {
        let item = storage.get_mut(entity).unwrap();
        {
            if self.last_before_start_of_frame.is_some()
                && self.last_before_start_of_frame.unwrap() == item.key
            {
                self.last_before_start_of_frame = item.replacement_next.clone();
            }
        }
        let head_mut = self.get_head();
        let item = storage.get_mut(entity).unwrap();
        if head_mut == Some(item.key) {
            if let Some(t) = item.replacement_next {
                if let Some(_v) = head_mut {
                    // *v = t.clone();
                    self.list.pop_front();
                    self.list.push_front(t.clone());
                }
            } else {
                if let None = head_mut {
                    self.list.clear();
                }
            }
        } else {
            let _entity = item.replacement_previous.unwrap().clone();
            let state_entity = item.replacement_next.clone();
            item.replacement_next = state_entity;
        }

        let tail_mut = self.get_tail();
        let item = storage.get_mut(entity).unwrap();
        if tail_mut.is_some() && tail_mut == Some(item.key) {
            if let Some(t) = item.replacement_previous {
                if let Some(_v) = tail_mut {
                    // *v = t.clone();
                    self.list.pop_back();
                    self.list.push_back(t.clone());
                }
            } else {
                if let None = tail_mut {
                    self.list.pop_back();
                }
            }
        } else {
            let _entity = item.replacement_next.unwrap().clone();
            let state_entity = item.replacement_previous.clone();
            item.replacement_previous = state_entity;
        }
        self.count -= 1;
    }
    pub fn mark_tile_rendered(&mut self, storage: &mut QuadtreeTileStorage, entity: TileKey) {
        let head_mut = self.get_head();
        let item = storage.get_mut(&entity).unwrap();
        if head_mut.is_some() && head_mut.unwrap() == item.key {
            if self.last_before_start_of_frame.is_some()
                && self.last_before_start_of_frame.unwrap() == item.key
            {
                if let Some(t) = item.replacement_previous {
                    self.last_before_start_of_frame = Some(t.clone());
                } else {
                    self.last_before_start_of_frame = None;
                }
            }
            return;
        }

        self.count += 1;
        let head_mut = self.get_head();
        let item = storage.get_mut(&entity).unwrap();
        if head_mut.is_none() {
            item.replacement_next = None;
            item.replacement_previous = None;
            if let Some(_v) = head_mut {
                // *v = item.key;
                self.list.pop_front();
                self.list.push_front(item.key);
            }
            let item_key = item.key.clone();
            let tail_mut = self.get_tail();
            if let Some(_v) = tail_mut {
                // *v = item.key;
                self.list.pop_back();
                self.list.push_back(item_key);
            }
            return;
        }

        if item.replacement_next.is_some() || item.replacement_previous.is_some() {
            self.remove(storage, &entity);
        }

        let head_mut = self.get_head();
        let item = storage.get_mut(&entity).unwrap();
        item.replacement_previous = None;
        if let Some(v) = head_mut {
            item.replacement_next = Some(v.clone());
            let entity = v.clone();
            let _state_entity = Some(item.key.clone());
            item.replacement_previous = Some(entity);
        } else {
            item.replacement_next = None;
        }
    }
}
