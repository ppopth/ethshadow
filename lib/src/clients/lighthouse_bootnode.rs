use crate::clients::CommonParams;
use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::process::Command;

use crate::clients::{Client, ValidatorSet};
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::utils::log_and_wait;
use crate::Error;
use crate::{genesis, CowStr};

const PORT: &str = "4011";

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct LighthouseBootnode {
    #[serde(flatten)]
    pub common: CommonParams,
    pub lcli_executable: CowStr,
}

impl Default for LighthouseBootnode {
    fn default() -> Self {
        Self {
            common: CommonParams::default(),
            lcli_executable: "lcli".into(),
        }
    }
}

#[typetag::deserialize(name = "lighthouse_bootnode")]
impl Client for LighthouseBootnode {
    fn add_to_node(
        &self,
        node: &NodeInfo,
        ctx: &mut SimulationContext,
        _vs: &ValidatorSet,
    ) -> Result<Process, Error> {
        let dir = node.dir().join("lighthouse_bootnode");
        debug!("Calling lcli generate-bootnode-enr");
        let status = log_and_wait(
            Command::new(self.lcli_executable.as_ref())
                .arg("generate-bootnode-enr")
                .arg("--testnet-dir")
                .arg(ctx.metadata_path())
                .arg("--ip")
                .arg(node.ip().to_string())
                .arg("--udp-port")
                .arg(PORT)
                .arg("--tcp-port")
                .arg(PORT)
                .arg("--genesis-fork-version")
                .arg(genesis::GENESIS_FORK_VERSION)
                .arg("--output-dir")
                .arg(&dir),
        )?;
        if !status.success() {
            return Err(Error::ChildProcessFailure(
                "lcli generate-bootnode-enr".to_string(),
            ));
        }

        let enr_path = dir.join("enr.dat");
        let enr = read_to_string(enr_path)?;
        ctx.add_cl_bootnode_enr(enr);

        Ok(Process {
            path: self.common.executable_or("lighthouse"),
            args: format!(
                "--testnet-dir \"{}\" \
                boot_node \
                --port {PORT} \
                --network-dir {} {}",
                ctx.metadata_path().to_str().ok_or(Error::NonUTF8Path)?,
                dir.to_str().ok_or(Error::NonUTF8Path)?,
                self.common.arguments("--disable-packet-filter"),
            ),
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "0s".into(),
        })
    }
}
