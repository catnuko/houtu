use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};

use self::fetch_img_job::FetchImgJob;

mod fetch_img_job;
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {}
}
#[derive(Resource)]
pub struct ImgServer {
    img_storage: HashMap<u32, String>,
    next_id: u32,
}
impl ImgServer {
    fn get_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        return id;
    }
    pub fn get(&mut self, src: String) {
        let id = self.get_id();
    }
    pub fn remove_img(&mut self, id: &u32) {
        self.img_storage.remove(id);
    }
}

#[derive(Event)]
pub struct FetchImg {
    pub src: String,
}
fn task_update(
    events: EventReader<FetchImg>,
    mut job_spawner: houtu_jobs::JobSpawner,
    asset_server: ResMut<AssetServer>,
    mut finished_jobs: bevy_jobs::FinishedJobs,
) {
    for fetch_img in events.iter() {
        if fetch_img.src.starts_with("http") || fetch_img.src.starts_with("https") {
            job_spawner.spawn(FetchImgJob {
                url: fetch_img.src.clone(),
            })
        } else {
            let img = asset_server.load(fetch_img.src);
        }
    }
    while let Some(result) = finished_jobs.take_next::<FetchImgJob>() {}
}
