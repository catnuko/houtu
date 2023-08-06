use houtu_jobs::{
    AsyncReturn, Context, Job, JobOutcomePayload, Progress,
};
use std::{fmt, marker::PhantomData};

pub trait FileLoader<R> {
    type Error: fmt::Debug;

    const FILE_TYPE_NAME: &'static str;

    fn from_bytes(bytes: bytes::Bytes) -> Self;
    fn load(self) -> Result<R, Self::Error>;
}

pub struct LoadFileJob<F: FileLoader<R>, R> {
    pub file_loader: F,
    pub path: String,
    pub source_crs: String,

    pub phantom: PhantomData<R>,
}
pub struct LoadFileJobOutcome<R> {
    pub result: R,
    pub path: String,
    pub source_crs: String,
}

impl<F, R> Job for LoadFileJob<F, R>
where
    R: Sync + Send + 'static,
    F: FileLoader<R> + Sync + Send + 'static,
    <F as FileLoader<R>>::Error: Send + Sync + 'static,
{
    type Outcome = Result<LoadFileJobOutcome<R>, F::Error>;

    fn name(&self) -> String {
        format!("Loading {} file", F::FILE_TYPE_NAME)
    }

    fn perform(self, _: Context) -> AsyncReturn<Self::Outcome> {
        Box::pin(async move {
            Ok(LoadFileJobOutcome {
                result: self.file_loader.load()?,
                path: self.path,
                source_crs: self.source_crs,
            })
        })
    }

    fn spawn(self, commands: &mut bevy::ecs::system::Commands) {
        let (outcome_tx, outcome_recv) = async_channel::unbounded::<JobOutcomePayload>();
        let (progress_tx, progress_recv) = async_channel::unbounded::<Progress>();

        let job_name = self.name();
        let in_progress_job = houtu_jobs::InProgressJob {
            name: job_name.clone(),
            progress: 0,
            progress_recv,
            outcome_recv,
        };

        bevy::tasks::AsyncComputeTaskPool::get()
            .spawn(async move {
                let instant = instant::Instant::now();
                bevy::log::info!("Starting job '{}'", job_name);
                let outcome = self.perform(Context { progress_tx }).await;
                bevy::log::info!("Completed job '{}' in {:?}", job_name, instant.elapsed());
                if let Err(e) = outcome_tx
                    .send(JobOutcomePayload {
                        job_outcome_type_id: std::any::TypeId::of::<Self>(),
                        job_outcome: Box::new(outcome),
                    })
                    .await
                {
                    bevy::log::error!(
                        "Failed to send result from job {} back to main thread: {:?}",
                        job_name,
                        e
                    );
                }
            })
            .detach();

        commands.spawn(in_progress_job);
    }
}
