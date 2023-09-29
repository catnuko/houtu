//reference:https://github.com/frewsxcv/rgis/blob/main/rgis-network/src/lib.rs

use bevy::prelude::{Handle, Image};
use futures_util::StreamExt;
use std::io;
pub struct FetchedImg {
    pub id: Handle<Image>,
    pub bytes: bytes::Bytes,
}

pub struct FetchImgJob {
    pub url: String,
    pub id: Handle<Image>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}

impl houtu_jobs::Job for FetchImgJob {
    type Outcome = Result<FetchedImg, Error>;

    fn name(&self) -> String {
        "".to_string()
    }

    fn perform(self, ctx: houtu_jobs::Context) -> houtu_jobs::AsyncReturn<Self::Outcome> {
            Box::pin(async move {
                let fetch = async {
                    let response = reqwest::get(self.url).await?;
                    let total_size = response.content_length().unwrap_or(0);
                    let mut bytes_stream = response.bytes_stream();
                    let mut bytes = Vec::<u8>::with_capacity(total_size as usize);

                    while let Some(bytes_chunk) = bytes_stream.next().await {
                        let mut bytes_chunk = Vec::from(bytes_chunk?);
                        bytes.append(&mut bytes_chunk);
                        if total_size > 0 {
                            let _ = ctx
                                .send_progress((bytes.len() / total_size as usize) as u8)
                                .await;
                        }
                    }

                    Ok(FetchedImg {
                        bytes: bytes::Bytes::from(bytes),
                        id: self.id.clone(),
                    })
                };
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
}
