use crate::clients::CommonParams;
use crate::clients::ENGINE_API_PORT;
use crate::clients::{Client, JSON_RPC_PORT};
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::utils::log_and_wait;
use crate::validators::Validator;
use crate::Error;
use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

const PORT: &str = "21000";

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Geth {
    #[serde(flatten)]
    pub common: CommonParams,
}

#[typetag::deserialize(name = "geth")]
impl Client for Geth {
    fn add_to_node(
        &self,
        node: &NodeInfo,
        ctx: &mut SimulationContext,
        _validators: &[Validator],
    ) -> Result<Process, Error> {
        let genesis_file = ctx.metadata_path().join("genesis.json");
        let genesis_file = genesis_file.to_str().ok_or(Error::NonUTF8Path)?;

        let dir = node.dir().join("geth");
        let dir = dir.to_str().ok_or(Error::NonUTF8Path)?;

        let executable = self.common.executable_or("geth");

        debug!("Calling geth init");
        let status = log_and_wait(
            Command::new(executable.as_ref())
                .arg("init")
                .arg("--datadir")
                .arg(dir)
                .arg(genesis_file),
        )?;
        if !status.success() {
            return Err(Error::ChildProcessFailure("geth init".to_string()));
        }

        ctx.add_el_http_endpoint(format!("http://{}:{JSON_RPC_PORT}", node.ip()));

        Ok(Process {
            path: executable,
            args: format!(
                "--datadir {dir} \
                --authrpc.port {ENGINE_API_PORT} \
                --authrpc.jwtsecret {} \
                --http \
                --http.addr 0.0.0.0 \
                --http.port {JSON_RPC_PORT} \
                --http.api eth,rpc,web3 \
                --port {PORT} \
                --bootnodes {} \
                --nat extip:{} \
                --log.file {dir}/geth.log {}",
                ctx.jwt_path().to_str().ok_or(Error::NonUTF8Path)?,
                ctx.el_bootnode_enodes().join(","),
                node.ip(),
                self.common.arguments("--syncmode full --ipcdisable"),
            ),
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "5s".into(),
        })
    }

    fn is_el_client(&self) -> bool {
        true
    }
}
