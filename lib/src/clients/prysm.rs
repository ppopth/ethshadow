use crate::clients::Client;
use crate::clients::CommonParams;
use crate::clients::{BEACON_API_PORT, ENGINE_API_PORT};
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::ValidatorSet;
use crate::Error;
use serde::Deserialize;
use std::collections::HashMap;

const PORT: &str = "31000";

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Prysm {
    #[serde(flatten)]
    pub common: CommonParams,
    pub lower_target_peers: bool,
}

impl Default for Prysm {
    fn default() -> Self {
        Self {
            common: CommonParams::default(),
            lower_target_peers: true,
        }
    }
}

#[typetag::deserialize(name = "prysm")]
impl Client for Prysm {
    fn add_to_node<'a>(
        &self,
        node: &NodeInfo<'a>,
        ctx: &mut SimulationContext<'a>,
        _vs: &ValidatorSet,
    ) -> Result<Process, Error> {
        if self.common.executable.is_empty() {
            return Err(Error::MissingExecutable(String::from("prysm")));
        }
        let dir = node.dir().join("prysm");
        let dir = dir.to_str().ok_or(Error::NonUTF8Path)?;

        let ip = node.ip();

        ctx.add_cl_http_endpoint(format!("{ip}:{BEACON_API_PORT}"));

        let meta_dir = ctx.metadata_path().to_str().ok_or(Error::NonUTF8Path)?;

        let mut args = format!(
            "--chain-config-file \"{meta_dir}/config.yaml\" \
                --accept-terms-of-use \
                --contract-deployment-block 0 \
                --deposit-contract 0x4242424242424242424242424242424242424242 \
                --genesis-state \"{meta_dir}/genesis.ssz\"
                --datadir \"{dir}\" \
                --execution-endpoint http://localhost:{ENGINE_API_PORT} \
                --jwt-secret \"{}\" \
                --bootstrap-node {} \
                --p2p-tcp-port {PORT} \
                --p2p-udp-port {PORT} \
                --p2p-host-ip {ip} \
                --http-port {BEACON_API_PORT} \
                {} ",
            ctx.jwt_path().to_str().ok_or(Error::NonUTF8Path)?,
            ctx.cl_bootnode_enrs().join(" --bootstrap-node "),
            self.common.extra_args,
        );
        if self.lower_target_peers && ctx.num_cl_clients() <= 100 {
            args.push_str(&format!("--p2p-max-peers {}", ctx.num_cl_clients() - 1));
        }

        Ok(Process {
            path: self.common.executable.clone().into(),
            args,
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "5s".into(),
        })
    }

    fn is_cl_client(&self) -> bool {
        true
    }
}
