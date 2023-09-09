use crate::binary::sender::Sender;
use anyhow::Result;
use iggy::error::Error;
use iggy::streams::create_stream::CreateStream;
use std::sync::Arc;
use streaming::systems::system::System;
use streaming::users::user_context::UserContext;
use tokio::sync::RwLock;
use tracing::trace;

pub async fn handle(
    command: &CreateStream,
    sender: &mut dyn Sender,
    user_context: &UserContext,
    system: Arc<RwLock<System>>,
) -> Result<(), Error> {
    trace!("{}", command);
    let mut system = system.write().await;
    system.permissioner.create_stream(user_context.user_id)?;
    system
        .create_stream(user_context.user_id, command.stream_id, &command.name)
        .await?;
    sender.send_empty_ok_response().await?;
    Ok(())
}
