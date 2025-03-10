use crate::clients::CommonParams;
use crate::clients::BEACON_API_PORT;
use crate::clients::{Client, ValidatorDemand};
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::ValidatorSet;
use crate::Error;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::fs::create_dir;

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct LighthouseValidatorClient {
    #[serde(flatten)]
    pub common: CommonParams,
    pub validators: Option<usize>,
}

#[typetag::deserialize(name = "lighthouse_vc")]
impl Client for LighthouseValidatorClient {
    fn add_to_node(
        &self,
        node: &NodeInfo,
        ctx: &mut SimulationContext,
        vs: &ValidatorSet,
    ) -> Result<Process, Error> {
        let dir = node.dir().join("lighthouse");
        let dir_str = dir.to_str().ok_or(Error::NonUTF8Path)?;
        if !dir.exists() {
            create_dir(&dir)?;
        }
        fs::rename(
            vs.base_path().join("secrets"),
            dir.join("secrets"),
        )?;
        fs::rename(
            vs.base_path().join("keys"),
            dir.join("validators"),
        )?;

        Ok(Process {
            path: self.common.executable_or("lighthouse"),
            args: format!(
                "--testnet-dir \"{}\" \
                validator_client \
                --datadir \"{dir_str}\" \
                --beacon-nodes http://localhost:{BEACON_API_PORT} \
                --init-slashing-protection {}",
                ctx.metadata_path().to_str().ok_or(Error::NonUTF8Path)?,
                self.common.arguments(
                    "--suggested-fee-recipient 0xf97e180c050e5Ab072211Ad2C213Eb5AEE4DF134"
                ),
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
