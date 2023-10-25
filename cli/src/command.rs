use crate::{
    consumer_groups, consumer_offsets, messages, partitions, personal_access_tokens, streams,
    system, topics, users,
};
use iggy::client_error::ClientError;
use iggy::clients::client::IggyClient;
use iggy::command::Command;
use iggy::messages::poll_messages::PollMessages;
use std::str::FromStr;
use tracing::info;

pub async fn handle(input: &str, client: &IggyClient) -> Result<(), ClientError> {
    let command = Command::from_str(input).map_err(|_| ClientError::InvalidCommand)?;
    info!("Handling '{}' command...", command);
    match command {
        Command::Ping(payload) => system::ping(&payload, client).await,
        Command::GetStats(payload) => system::get_stats(&payload, client).await,
        Command::GetSnapshot(payload) => system::get_snapshot(&payload, client).await,
        Command::GetMe(payload) => system::get_me(&payload, client).await,
        Command::GetClient(payload) => system::get_client(&payload, client).await,
        Command::GetClients(payload) => system::get_clients(&payload, client).await,
        Command::GetUser(payload) => users::get_user(&payload, client).await,
        Command::GetUsers(payload) => users::get_users(&payload, client).await,
        Command::CreateUser(payload) => users::create_user(&payload, client).await,
        Command::DeleteUser(payload) => users::delete_user(&payload, client).await,
        Command::UpdateUser(payload) => users::update_user(&payload, client).await,
        Command::UpdatePermissions(payload) => users::update_permissions(&payload, client).await,
        Command::ChangePassword(payload) => users::change_password(&payload, client).await,
        Command::LoginUser(payload) => users::login_user(&payload, client).await,
        Command::LogoutUser(payload) => users::logout_user(&payload, client).await,
        Command::GetPersonalAccessTokens(payload) => {
            personal_access_tokens::get_personal_access_tokens(&payload, client).await
        }
        Command::CreatePersonalAccessToken(payload) => {
            personal_access_tokens::create_personal_access_token(&payload, client).await
        }
        Command::DeletePersonalAccessToken(payload) => {
            personal_access_tokens::delete_personal_access_token(&payload, client).await
        }
        Command::LoginWithPersonalAccessToken(payload) => {
            personal_access_tokens::login_with_personal_access_token(&payload, client).await
        }
        Command::SendMessages(mut payload) => messages::send_messages(&mut payload, client).await,
        Command::PollMessages(payload) => {
            let format = match input.split('|').last() {
                Some(format) => match format {
                    "b" | "binary" => Format::Binary,
                    "s" | "string" => Format::String,
                    _ => Format::None,
                },
                None => Format::None,
            };
            let payload = PollMessagesWithFormat { payload, format };
            messages::poll_messages(&payload, client).await
        }
        Command::StoreConsumerOffset(payload) => {
            consumer_offsets::store_consumer_offset(&payload, client).await
        }
        Command::GetConsumerOffset(payload) => {
            consumer_offsets::get_consumer_offset(&payload, client).await
        }
        Command::GetStream(payload) => streams::get_stream(&payload, client).await,
        Command::GetStreams(payload) => streams::get_streams(&payload, client).await,
        Command::CreateStream(payload) => streams::create_stream(&payload, client).await,
        Command::DeleteStream(payload) => streams::delete_stream(&payload, client).await,
        Command::UpdateStream(payload) => streams::update_stream(&payload, client).await,
        Command::GetTopic(payload) => topics::get_topic(&payload, client).await,
        Command::GetTopics(payload) => topics::get_topics(&payload, client).await,
        Command::CreateTopic(payload) => topics::create_topic(&payload, client).await,
        Command::DeleteTopic(payload) => topics::delete_topic(&payload, client).await,
        Command::UpdateTopic(payload) => topics::update_topic(&payload, client).await,
        Command::CreatePartitions(payload) => partitions::create_partitions(&payload, client).await,
        Command::DeletePartitions(payload) => partitions::delete_partitions(&payload, client).await,
        Command::GetConsumerGroup(payload) => {
            consumer_groups::get_consumer_group(&payload, client).await
        }
        Command::GetConsumerGroups(payload) => {
            consumer_groups::get_consumer_groups(&payload, client).await
        }
        Command::CreateConsumerGroup(payload) => {
            consumer_groups::create_consumer_group(&payload, client).await
        }
        Command::DeleteConsumerGroup(payload) => {
            consumer_groups::delete_consumer_group(&payload, client).await
        }
        Command::JoinConsumerGroup(payload) => {
            consumer_groups::join_consumer_group(&payload, client).await
        }
        Command::LeaveConsumerGroup(payload) => {
            consumer_groups::leave_consumer_group(&payload, client).await
        }
    }
}

#[derive(Debug)]
pub struct PollMessagesWithFormat {
    pub payload: PollMessages,
    pub format: Format,
}

#[derive(Debug, PartialEq, Default, Copy, Clone)]
pub enum Format {
    #[default]
    None,
    Binary,
    String,
}
