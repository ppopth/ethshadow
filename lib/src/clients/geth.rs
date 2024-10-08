use crate::clients::ENGINE_API_PORT;
use crate::clients::{Client, JSON_RPC_PORT};
use crate::config::shadow::Process;
use crate::error::Error;
use crate::node::{Node, SimulationContext};
use crate::validators::Validator;
use crate::CowStr;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

const PORT: &str = "21000";

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Geth {
    pub executable: CowStr,
}

impl Default for Geth {
    fn default() -> Self {
        Self {
            executable: "geth".into(),
        }
    }
}

#[typetag::deserialize(name = "geth")]
impl Client for Geth {
    fn add_to_node(
        &self,
        node: &Node,
        ctx: &mut SimulationContext,
        _validators: &[Validator],
    ) -> Result<Process, Error> {
        let genesis_file = ctx.metadata_path().join("genesis.json");
        let genesis_file = genesis_file.to_str().ok_or(Error::NonUTF8Path)?;

        let dir = node.dir().join("geth");
        let dir = dir.to_str().ok_or(Error::NonUTF8Path)?;

        let status = Command::new(self.executable.as_ref())
            .arg("init")
            .arg("--datadir")
            .arg(dir)
            .arg(genesis_file)
            .stdout(Stdio::null())
            .spawn()?
            .wait()?;
        if !status.success() {
            return Err(Error::ChildProcessFailure("geth init".to_string()));
        }

        ctx.add_el_http_endpoint(format!("http://{}:{JSON_RPC_PORT}", node.ip()));

        Ok(Process {
            path: self.executable.clone(),
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
                --ipcdisable \
                --log.file {dir}/geth.log \
                --syncmode full",
                ctx.jwt_path().to_str().ok_or(Error::NonUTF8Path)?,
                ctx.el_bootnode_enodes().join(","),
                node.ip(),
            ),
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "5s".into(),
        })
    }
}
