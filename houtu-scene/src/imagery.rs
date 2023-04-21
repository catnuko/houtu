use bevy::prelude::{Handle, Image};

pub enum ImageLayerState {
    UNLOADED = 0,
    TRANSITIONING = 1,
    RECEIVED = 2,
    TEXTURE_LOADED = 3,
    READY = 4,
    FAILED = 5,
    INVALID = 6,
    PLACEHOLDER = 7,
}
#[derive(Clone, Debug, Resource)]
pub struct Imagery {
    pub state: ImageLayerState,
    pub x: i32,
    pub y: i32,
    pub level: i32,
    pub image_url: String,
    pub texture: Option<Handle<Image>>,
}
impl Default for Imagery {
    fn default() -> Self {
        Self {
            state: ImageLayerState::UNLOADED,
            x: 0,
            y: 0,
            level: 0,
            image_url: "".into(),
            texture: None,
        }
    }
}
impl Imagery {
    pub fn new(x: f32, y: f32, level: f32) -> Self {
        Self {
            state: ImageLayerState::UNLOADED,
            x: x,
            y: y,
            level: z,
            image_url: "".into(),
            texture: None,
        }
    }
    pub fn set_image_url(&mut self, url: str) {
        self.image_url = url;
    }
    pub fn set_texture(&mut self, texture: Handle<Image>) {
        self.texture = Some(texture);
    }
}
fn handle_load_file_job_finished_events<F: geo_file_loader::FileLoader + Send + Sync + 'static>(
    mut finished_jobs: bevy_jobs::FinishedJobs,
    mut create_layer_event_writer: EventWriter<houtu_events::CreateLayerEvent>,
) where
    <F as geo_file_loader::FileLoader>::Error: Send + Sync + 'static,
{
    while let Some(outcome) = finished_jobs.take_next::<crate::jobs::LoadFileJob<F>>() {
        match outcome {
            Ok(outcome) => create_layer_event_writer.send(houtu_events::CreateLayerEvent {
                name: outcome.name,
                ogc_type: "WMTS".to_string(),
                url: outcome.url,
                source_crs: outcome.source_crs,
            }),
            Err(e) => {
                bevy::log::error!("Encountered error when loading file: {:?}", e);
            }
        }
    }
}
