use std::{io};

use houtu_jobs::{
    AsyncReturn, Context, Job,
};
pub struct CreateVerticeJob {}
pub struct CreateVerticeJobOutcome {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}

impl Job for CreateVerticeJob {
    type Outcome = Result<CreateVerticeJobOutcome, Error>;
    fn name(&self) -> String {
        format!("create vertice ",)
    }
    fn perform(self, _context: Context) -> AsyncReturn<Self::Outcome> {
        Box::pin(async move {
            let fetch = async { Ok(CreateVerticeJobOutcome {}) };
            #[cfg(not(target_arch = "wasm32"))]
            {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                runtime.block_on(fetch)
            }
            #[cfg(target_arch = "wasm32")]
            {
                fetch.await
            }
        })
    }
    // fn spawn(self, commands: &mut bevy::ecs::system::Commands) {}
}
