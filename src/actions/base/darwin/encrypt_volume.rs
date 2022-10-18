use serde::Serialize;
use tokio::process::Command;

use crate::execute_command;

use crate::actions::{Action, ActionDescription, ActionState, Actionable};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct EncryptVolume {
    unit: String,
    action_state: ActionState,
}

impl EncryptVolume {
    #[tracing::instrument(skip_all)]
    pub async fn plan(unit: String) -> Result<Self, EncryptVolumeError> {
        Ok(Self {
            unit,
            action_state: ActionState::Uncompleted,
        })
    }
}

#[async_trait::async_trait]
impl Actionable for EncryptVolume {
    type Error = EncryptVolumeError;

    fn describe_execute(&self) -> Vec<ActionDescription> {
        if self.action_state == ActionState::Completed {
            vec![]
        } else {
            vec![ActionDescription::new(
                "Start the systemd Nix service and socket".to_string(),
                vec![
                    "The `nix` command line tool communicates with a running Nix daemon managed by your init system".to_string()
                ]
            )]
        }
    }

    #[tracing::instrument(skip_all, fields(
        unit = %self.unit,
    ))]
    async fn execute(&mut self) -> Result<(), Self::Error> {
        let Self { unit, action_state } = self;
        if *action_state == ActionState::Completed {
            tracing::trace!("Already completed: Starting systemd unit");
            return Ok(());
        }
        tracing::debug!("Starting systemd unit");

        // TODO(@Hoverbear): Handle proxy vars
        execute_command(
            Command::new("systemctl")
                .arg("enable")
                .arg("--now")
                .arg(format!("{unit}")),
        )
        .await
        .map_err(EncryptVolumeError::Command)?;

        tracing::trace!("Started systemd unit");
        *action_state = ActionState::Completed;
        Ok(())
    }

    fn describe_revert(&self) -> Vec<ActionDescription> {
        if self.action_state == ActionState::Uncompleted {
            vec![]
        } else {
            vec![ActionDescription::new(
                "Stop the systemd Nix service and socket".to_string(),
                vec![
                    "The `nix` command line tool communicates with a running Nix daemon managed by your init system".to_string()
                ]
            )]
        }
    }

    #[tracing::instrument(skip_all, fields(
        unit = %self.unit,
    ))]
    async fn revert(&mut self) -> Result<(), Self::Error> {
        let Self { unit, action_state } = self;
        if *action_state == ActionState::Uncompleted {
            tracing::trace!("Already reverted: Stopping systemd unit");
            return Ok(());
        }
        tracing::debug!("Stopping systemd unit");

        // TODO(@Hoverbear): Handle proxy vars
        execute_command(Command::new("systemctl").arg("stop").arg(format!("{unit}")))
            .await
            .map_err(EncryptVolumeError::Command)?;

        tracing::trace!("Stopped systemd unit");
        *action_state = ActionState::Completed;
        Ok(())
    }
}

impl From<EncryptVolume> for Action {
    fn from(v: EncryptVolume) -> Self {
        Action::DarwinEncryptVolume(v)
    }
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum EncryptVolumeError {
    #[error("Failed to execute command")]
    Command(
        #[source]
        #[serde(serialize_with = "crate::serialize_error_to_display")]
        std::io::Error,
    ),
}
