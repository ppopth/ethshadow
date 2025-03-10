use crate::clients::{Client, ValidatorDemand};
use crate::config::ethshadow::{Node, DEFAULT_GENESIS_GEN_IMAGE, DEFAULT_MNEMONIC};
use crate::config::EthShadowConfig;
use crate::utils::log_and_wait;
use crate::Error;
use log::info;
use std::path::{Path, PathBuf};
use std::process::Command;
use users::get_current_uid;

pub struct ValidatorManager {
    image_name: String,
    mnemonic: String,
    output_path: PathBuf,
    validator_count: usize,
    val_for_each_any: usize,
    remainder: usize,
    already_assigned: usize,
    idx: usize,
}
impl ValidatorManager {
    pub fn new(
        config: &EthShadowConfig,
        nodes: &[Node],
        output_path: &Path,
    ) -> Result<ValidatorManager, Error> {
        let validator_count;
        let mut val_for_each_any = 0;
        let mut remainder = 0;

        let mut requested = 0;
        let mut anys = 0;
        for node in nodes {
            let count = node.count;
            for client in &node.clients {
                match client.validator_demand() {
                    ValidatorDemand::Count(val_count) => requested += val_count * count,
                    ValidatorDemand::Any => anys += count,
                    ValidatorDemand::None => {}
                }
            }
        }

        if let Some(validators) = config.validators {
            let Some(remaining) = validators.checked_sub(requested) else {
                return Err(Error::MoreValidatorsRequested(validators, requested));
            };
            validator_count = validators;
            if anys != 0 {
                val_for_each_any = remaining / anys;
                remainder = remaining % anys;
            } else if remaining != 0 {
                return Err(Error::LeftoverValidators);
            }
        } else {
            if anys != 0 {
                return Err(Error::MissingValidatorCount);
            }
            validator_count = requested;
        };

        let image_name = config
            .genesis
            .generator_image
            .as_deref()
            .unwrap_or(DEFAULT_GENESIS_GEN_IMAGE)
            .into();
        let mnemonic = config
            .genesis
            .mnemonic
            .as_deref()
            .unwrap_or(DEFAULT_MNEMONIC)
            .into();

        Ok(ValidatorManager {
            image_name,
            mnemonic,
            output_path: output_path.to_path_buf(),
            validator_count,
            val_for_each_any,
            remainder,
            already_assigned: 0,
            idx: 0,
        })
    }

    pub fn assign(&mut self, client: &dyn Client) -> Result<ValidatorSet, Error> {
        let count = match client.validator_demand() {
            ValidatorDemand::None => 0,
            ValidatorDemand::Any => {
                if self.remainder > 0 {
                    self.remainder -= 1;
                    self.val_for_each_any + 1
                } else {
                    self.val_for_each_any
                }
            }
            ValidatorDemand::Count(count) => count,
        };
        if count == 0 {
            return Ok(ValidatorSet::default());
        }
        let start = self.already_assigned;
        let end = start + count;

        let mut data_mount = self.output_path.as_os_str().to_owned();
        data_mount.push(":/data");
        info!(
            "Generating validator of index from {} to {}",
            start,
            end - 1
        );
        let status = log_and_wait(
            Command::new("docker")
                .args(["run", "--rm", "-i", "-u"])
                .arg(get_current_uid().to_string())
                .arg("-v")
                .arg(&data_mount)
                .arg("--entrypoint=eth2-val-tools")
                .arg(&self.image_name)
                .arg("keystores")
                .arg("--insecure")
                .arg("--out-loc")
                .arg(format!("/data/validator_keys_{}", self.idx))
                .arg("--source-mnemonic")
                .arg(&self.mnemonic)
                .arg("--source-min")
                .arg(start.to_string())
                .arg("--source-max")
                .arg(end.to_string()),
        )?;
        if !status.success() {
            return Err(Error::ChildProcessFailure(self.image_name.to_string()));
        }
        self.already_assigned = end;
        let base_path = self
            .output_path
            .join(format!("validator_keys_{}", self.idx));

        self.idx += 1;
        Ok(ValidatorSet {
            base_path,
            count: end - start,
        })
    }

    pub fn total_count(&self) -> usize {
        self.validator_count
    }
}

#[derive(Default)]
pub struct ValidatorSet {
    base_path: PathBuf,
    count: usize,
}

impl ValidatorSet {
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    pub fn count(&self) -> usize {
        self.count
    }
}
