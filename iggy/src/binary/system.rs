use crate::binary::binary_client::BinaryClient;
use crate::binary::{fail_if_not_authenticated, mapper};
use crate::bytes_serializable::BytesSerializable;
use crate::command::{
    GET_CLIENTS_CODE, GET_CLIENT_CODE, GET_ME_CODE, GET_SNAPSHOT_CODE, GET_STATS_CODE, PING_CODE,
};
use crate::error::Error;
use crate::models::client_info::{ClientInfo, ClientInfoDetails};
use crate::models::stats::Stats;
use crate::system::get_client::GetClient;
use crate::system::get_clients::GetClients;
use crate::system::get_me::GetMe;
use crate::system::get_snapshot::GetSnapshot;
use crate::system::get_stats::GetStats;
use crate::system::ping::Ping;

pub async fn get_stats(client: &dyn BinaryClient, command: &GetStats) -> Result<Stats, Error> {
    fail_if_not_authenticated(client).await?;
    let response = client
        .send_with_response(GET_STATS_CODE, &command.as_bytes())
        .await?;
    mapper::map_stats(&response)
}

pub async fn get_snapshot(client: &dyn BinaryClient, command: &GetSnapshot) -> Result<(), Error> {
    fail_if_not_authenticated(client).await?;
    client
        .send_with_response(GET_SNAPSHOT_CODE, &command.as_bytes())
        .await?;
    Ok(())
}

pub async fn get_me(
    client: &dyn BinaryClient,
    command: &GetMe,
) -> Result<ClientInfoDetails, Error> {
    fail_if_not_authenticated(client).await?;
    let response = client
        .send_with_response(GET_ME_CODE, &command.as_bytes())
        .await?;
    mapper::map_client(&response)
}

pub async fn get_client(
    client: &dyn BinaryClient,
    command: &GetClient,
) -> Result<ClientInfoDetails, Error> {
    fail_if_not_authenticated(client).await?;
    let response = client
        .send_with_response(GET_CLIENT_CODE, &command.as_bytes())
        .await?;
    mapper::map_client(&response)
}

pub async fn get_clients(
    client: &dyn BinaryClient,
    command: &GetClients,
) -> Result<Vec<ClientInfo>, Error> {
    fail_if_not_authenticated(client).await?;
    let response = client
        .send_with_response(GET_CLIENTS_CODE, &command.as_bytes())
        .await?;
    mapper::map_clients(&response)
}

pub async fn ping(client: &dyn BinaryClient, command: &Ping) -> Result<(), Error> {
    client
        .send_with_response(PING_CODE, &command.as_bytes())
        .await?;
    Ok(())
}
