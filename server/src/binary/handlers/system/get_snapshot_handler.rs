use crate::binary::sender::Sender;
use crate::streaming::session::Session;
use crate::streaming::systems::system::System;
use iggy::error::Error;
use iggy::system::get_snapshot::GetSnapshot;
use tokio::io::AsyncReadExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

pub async fn handle(
    command: &GetSnapshot,
    sender: &mut dyn Sender,
    session: &Session,
    system: Arc<RwLock<System>>,
) -> Result<(), Error> {
    debug!("session: {session}, command: {command}");
    let system = system.read().await;

    let snapshot_file_name = system.create_snapshot_file().await?;

    let mut file = match tokio::fs::File::open(snapshot_file_name.clone()).await {
        Ok(file) => file,
        Err(err) => {
          return Err(iggy::error::Error::IoError(err));
        }
    };
    
    let metadata = tokio::fs::metadata(&snapshot_file_name).await.unwrap();
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer).await.unwrap();

    sender.send_ok_response(buffer.as_slice()).await?;
    Ok(())
}
