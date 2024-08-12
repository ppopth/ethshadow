use crate::clients::Client;
use crate::clients::{BEACON_API_PORT, CL_PROMETHEUS_PORT, ENGINE_API_PORT};
use crate::config::shadow::Process;
use crate::error::Error;
use crate::node::{Node, SimulationContext};
use crate::validators::Validator;
use crate::CowStr;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;

const PORT: &str = "31000";

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Lighthouse {
    pub executable: CowStr,
}

impl Default for Lighthouse {
    fn default() -> Self {
        Self {
            executable: "lighthouse".into(),
        }
    }
}

#[typetag::deserialize(name = "lighthouse")]
impl Client for Lighthouse {
    fn add_to_node(
        &self,
        node: &Node,
        ctx: &mut SimulationContext,
        _validators: &[Validator],
    ) -> Result<Process, Error> {
        let dir = node.dir().join("lighthouse");
        let dir = dir.to_str().ok_or(Error::NonUTF8Path)?;

        let ip = node.ip();

        ctx.add_cl_http_endpoint(format!("{ip}:{BEACON_API_PORT}"));
        ctx.add_cl_monitoring_endpoint(format!("{ip}:{CL_PROMETHEUS_PORT}"));

        Ok(Process {
            path: self.executable.clone(),
            args: format!(
                "--testnet-dir \"{}\" \
                beacon_node \
                --datadir \"{dir}\" \
                --execution-endpoint http://localhost:{ENGINE_API_PORT} \
                --execution-jwt \"{}\" \
                --boot-nodes {} \
                --port {PORT} \
                --enr-address {ip} \
                --enr-udp-port {PORT} \
                --enr-tcp-port {PORT} \
                --http \
                --http-port {BEACON_API_PORT} \
                --disable-quic \
                --disable-upnp \
                --disable-packet-filter \
                --metrics-address 0.0.0.0 \
                --metrics-port {CL_PROMETHEUS_PORT} \
                --metrics",
                ctx.metadata_path().to_str().ok_or(Error::NonUTF8Path)?,
                ctx.jwt_path().to_str().ok_or(Error::NonUTF8Path)?,
                ctx.cl_bootnode_enrs().join(","),
            ),
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: format!("{}ms", ctx.rng().gen_range(5000..10000)).into(),
        })
    }
}
