use crate::cli_command::{CliCommand, PRINT_TARGET};
use crate::client::Client;
use crate::personal_access_tokens::delete_personal_access_token::DeletePersonalAccessToken;
use anyhow::Context;
use async_trait::async_trait;
use tracing::{event, Level};

pub struct DeletePersonalAccessTokenCmd {
    delete_token: DeletePersonalAccessToken,
}

impl DeletePersonalAccessTokenCmd {
    pub fn new(name: String) -> Self {
        Self {
            delete_token: DeletePersonalAccessToken { name },
        }
    }
}

#[async_trait]
impl CliCommand for DeletePersonalAccessTokenCmd {
    fn explain(&self) -> String {
        format!(
            "delete personal access tokens with name: {}",
            self.delete_token.name
        )
    }

    async fn execute_cmd(&mut self, client: &dyn Client) -> anyhow::Result<(), anyhow::Error> {
        client
            .delete_personal_access_token(&self.delete_token)
            .await
            .with_context(|| {
                format!(
                    "Problem deleting personal access tokens with name: {}",
                    self.delete_token.name
                )
            })?;

        event!(target: PRINT_TARGET, Level::INFO,
            "Personal access token with name: {} deleted", self.delete_token.name
        );

        Ok(())
    }
}
