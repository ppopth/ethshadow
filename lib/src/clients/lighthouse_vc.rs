use crate::clients::BEACON_API_PORT;
use crate::clients::{Client, ValidatorDemand};
use crate::config::shadow::Process;
use crate::error::Error;
use crate::node::{Node, SimulationContext};
use crate::validators::Validator;
use crate::CowStr;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::fs::create_dir;

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct LighthouseValidatorClient {
    pub executable: CowStr,
    pub validators: Option<u64>,
}

impl Default for LighthouseValidatorClient {
    fn default() -> Self {
        Self {
            executable: "lighthouse".into(),
            validators: None,
        }
    }
}

#[typetag::deserialize(name = "lighthouse_vc")]
impl Client for LighthouseValidatorClient {
    fn add_to_node(
        &self,
        node: &Node,
        ctx: &mut SimulationContext,
        validators: &[Validator],
    ) -> Result<Process, Error> {
        let dir = node.dir().join("lighthouse");
        let dir_str = dir.to_str().ok_or(Error::NonUTF8Path)?;
        if !dir.exists() {
            create_dir(&dir)?;
        }

        let secrets_dest = dir.join("secrets");
        if !secrets_dest.exists() {
            create_dir(&secrets_dest)?;
        }
        let validators_dest = dir.join("validators");
        if !validators_dest.exists() {
            create_dir(&validators_dest)?;
        }

        for validator in validators {
            let key = validator.key();
            fs::rename(
                validator.base_path().join("secrets").join(key),
                secrets_dest.join(key),
            )?;
            fs::rename(
                validator.base_path().join("keys").join(key),
                validators_dest.join(key),
            )?;
        }

        Ok(Process {
            path: self.executable.clone(),
            args: format!(
                "--testnet-dir \"{}\" \
                validator_client \
                --datadir \"{dir_str}\" \
                --beacon-nodes http://localhost:{BEACON_API_PORT} \
                --suggested-fee-recipient 0xf97e180c050e5Ab072211Ad2C213Eb5AEE4DF134 \
                --init-slashing-protection",
                ctx.metadata_path().to_str().ok_or(Error::NonUTF8Path)?,
            ),
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "5s".into(),
        })
    }

    fn validator_demand(&self) -> ValidatorDemand {
        match self.validators {
            None => ValidatorDemand::Any,
            Some(num) => ValidatorDemand::Count(num),
        }
    }
}
