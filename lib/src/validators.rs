use crate::clients::{Client, ValidatorDemand};
use crate::config::ethshadow::{Node, DEFAULT_GENESIS_GEN_IMAGE, DEFAULT_MNEMONIC};
use crate::config::EthShadowConfig;
use crate::utils::log_and_wait;
use crate::Error;
use itertools::Itertools;
use log::info;
use std::cmp::min;
use std::ffi::OsString;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use users::get_current_uid;

pub struct ValidatorManager {
    validators: Vec<Validator>,
    val_for_each_any: usize,
    remainder: usize,
    already_assigned: usize,
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

        info!("Generating {validator_count} validators");
        let validators = generate(
            config
                .genesis
                .generator_image
                .as_deref()
                .unwrap_or(DEFAULT_GENESIS_GEN_IMAGE),
            output_path,
            config
                .genesis
                .mnemonic
                .as_deref()
                .unwrap_or(DEFAULT_MNEMONIC),
            validator_count,
        )?;

        Ok(ValidatorManager {
            validators,
            val_for_each_any,
            remainder,
            already_assigned: 0,
        })
    }

    pub fn assign(&mut self, client: &dyn Client) -> &[Validator] {
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
        let start = self.already_assigned;
        let end = start + count;
        self.already_assigned = end;
        &self.validators[start..end]
    }

    pub fn total_count(&self) -> usize {
        self.validators.len()
    }
}

fn generate(
    image_name: &str,
    output_path: &Path,
    mnemonic: &str,
    total_val: usize,
) -> Result<Vec<Validator>, Error> {
    let mut validators = Vec::with_capacity(total_val);
    let mut idx = 0;
    let mut data_mount = output_path.as_os_str().to_owned();
    data_mount.push(":/data");
    while validators.len() < total_val {
        info!("Generating validator batch {}", idx + 1);
        let status = log_and_wait(
            Command::new("docker")
                .args(["run", "--rm", "-i", "-u"])
                .arg(get_current_uid().to_string())
                .arg("-v")
                .arg(&data_mount)
                .arg("--entrypoint=eth2-val-tools")
                .arg(image_name)
                .arg("keystores")
                .arg("--insecure")
                .arg("--out-loc")
                .arg(format!("/data/validator_keys_{idx}"))
                .arg("--source-mnemonic")
                .arg(mnemonic)
                .arg("--source-min")
                .arg(validators.len().to_string())
                .arg("--source-max")
                .arg(min(validators.len() + 4000, total_val).to_string()),
        )?;
        if !status.success() {
            return Err(Error::ChildProcessFailure(image_name.to_string()));
        }
        let base_path = output_path.join(format!("validator_keys_{idx}"));
        for validator in read_dir(base_path.join("keys"))?.map_ok(|e| Validator {
            base_path: base_path.clone(),
            key: e.file_name(),
        }) {
            validators.push(validator?);
        }
        idx += 1;
    }
    Ok(validators)
}

pub struct Validator {
    base_path: PathBuf,
    key: OsString,
}

impl Validator {
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    pub fn key(&self) -> &OsString {
        &self.key
    }
}
