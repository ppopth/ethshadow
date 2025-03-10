use crate::clients::CommonParams;
use crate::clients::ENGINE_API_PORT;
use crate::clients::{Client, JSON_RPC_PORT};
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::ValidatorSet;
use crate::Error;
use serde::Deserialize;
use std::collections::HashMap;

const PORT: &str = "21000";

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Reth {
    #[serde(flatten)]
    pub common: CommonParams,
}

#[typetag::deserialize(name = "reth")]
impl Client for Reth {
    fn add_to_node(
        &self,
        node: &NodeInfo,
        ctx: &mut SimulationContext,
        _vs: &ValidatorSet,
    ) -> Result<Process, Error> {
        let genesis_file = ctx.metadata_path().join("genesis.json");
        let genesis_file = genesis_file.to_str().ok_or(Error::NonUTF8Path)?;

        let dir = node.dir().join("reth");
        let dir = dir.to_str().ok_or(Error::NonUTF8Path)?;

        ctx.add_el_http_endpoint(format!("http://{}:{JSON_RPC_PORT}", node.ip()));

        Ok(Process {
            path: self.common.executable_or("reth"),
            args: format!(
                "node \
                --chain {genesis_file} \
                --datadir {dir} \
                --authrpc.port {ENGINE_API_PORT} \
                --authrpc.jwtsecret {} \
                --http \
                --http.addr 0.0.0.0 \
                --http.port {JSON_RPC_PORT} \
                --http.api eth,rpc,web3 \
                --port {PORT} \
                --bootnodes {} \
                --nat extip:{} \
                --log.file.directory {dir} {}",
                ctx.jwt_path().to_str().ok_or(Error::NonUTF8Path)?,
                ctx.el_bootnode_enodes().join(","),
                node.ip(),
                self.common.arguments("--ipcdisable"),
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
