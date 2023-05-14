use std::marker::PhantomData;

use houtu_jobs::{AsyncReturn, Context, FinishedJobs, Job, JobSpawner};

use super::{file, network};
use bevy::ecs::event::Events;
use bevy::prelude::*;

#[derive(Debug)]
pub enum LoadFileEvent<F: file::FileLoader<R>, R> {
    FromNetwork {
        name: String,
        url: String,
        crs: String,
    },
    FromBytes {
        file_name: String,
        file_loader: F,
        crs: String,
    },
    Phantom(PhantomData<R>),
}

pub fn handle_network_fetch_finished_jobs<F, R>(
    mut load_event_reader: ResMut<Events<LoadFileEvent<F, R>>>,
    mut finished_jobs: FinishedJobs,
) where
    R: Sync + Send + 'static,
    F: file::FileLoader<R> + Sync + Send + 'static,
    <F as file::FileLoader<R>>::Error: Send + Sync + 'static,
{
    while let Some(outcome) = finished_jobs.take_next::<network::NetworkFetchJob>() {
        match outcome {
            Ok(fetched) => load_event_reader.send(LoadFileEvent::FromBytes {
                file_loader: F::from_bytes(fetched.bytes),
                file_name: fetched.name,
                crs: fetched.crs,
            }),
            Err(e) => {
                bevy::log::error!("Could not fetch file: {:?}", e);
            }
        }
    }
}

pub fn handle_load_file_events<F, R>(
    mut load_event_reader: ResMut<Events<LoadFileEvent<F, R>>>,
    mut job_spawner: JobSpawner,
) where
    R: Sync + Send + 'static,
    F: file::FileLoader<R> + Sync + Send + 'static,
    <F as file::FileLoader<R>>::Error: Send + Sync + 'static,
{
    for event in load_event_reader.drain() {
        match event {
            LoadFileEvent::FromNetwork { url, crs, name } => {
                job_spawner.spawn(network::NetworkFetchJob { url, crs, name })
            }
            LoadFileEvent::FromBytes {
                file_name,
                file_loader,
                crs,
            } => job_spawner.spawn(file::LoadFileJob {
                file_loader,
                source_crs: crs,
                path: file_name,
                phantom: PhantomData,
            }),
            _ => {}
        }
    }
}

pub fn handle_load_file_job_finished_events<F, R>(
    mut finished_jobs: FinishedJobs,
    // mut create_layer_event_writer: EventWriter<CreateLayerEvent>,
) where
    R: Sync + Send + 'static,
    F: file::FileLoader<R> + Sync + Send + 'static,
    <F as file::FileLoader<R>>::Error: Send + Sync + 'static,
{
    while let Some(outcome) = finished_jobs.take_next::<file::LoadFileJob<F, R>>() {
        match outcome {
            Ok(outcome) => {
                bevy::log::info!("Loaded file: {:?}", outcome.path);
            }
            Err(e) => {
                bevy::log::error!("Encountered error when loading file: {:?}", e);
            }
        }
    }
}
