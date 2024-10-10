use crate::clients::Client;
use crate::clients::{BEACON_API_PORT, CL_PROMETHEUS_PORT, ENGINE_API_PORT};
use crate::config::shadow::Process;
use crate::error::Error;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::Validator;
use crate::CowStr;
use serde::Deserialize;
use std::collections::HashMap;

const PORT: &str = "31000";

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Lighthouse {
    pub executable: CowStr,
    pub extra_args: String,
    pub lower_target_peers: bool,
}

impl Default for Lighthouse {
    fn default() -> Self {
        Self {
            executable: "lighthouse".into(),
            extra_args: "".into(),
            lower_target_peers: true,
        }
    }
}

#[typetag::deserialize(name = "lighthouse")]
impl Client for Lighthouse {
    fn add_to_node<'a>(
        &self,
        node: &NodeInfo<'a>,
        ctx: &mut SimulationContext<'a>,
        _validators: &[Validator],
    ) -> Result<Process, Error> {
        let dir = node.dir().join("lighthouse");
        let dir = dir.to_str().ok_or(Error::NonUTF8Path)?;

        let ip = node.ip();

        ctx.add_cl_http_endpoint(format!("{ip}:{BEACON_API_PORT}"));
        ctx.add_cl_monitoring_endpoint(
            node.location(),
            node.reliability(),
            format!("{ip}:{CL_PROMETHEUS_PORT}"),
        );

        let mut args = format!(
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
                --metrics \
                {} ",
            ctx.metadata_path().to_str().ok_or(Error::NonUTF8Path)?,
            ctx.jwt_path().to_str().ok_or(Error::NonUTF8Path)?,
            ctx.cl_bootnode_enrs().join(","),
            self.extra_args,
        );
        if self.lower_target_peers && ctx.num_cl_clients() <= 100 {
            args.push_str(&format!("--target-peers {}", ctx.num_cl_clients() - 1));
        }

        Ok(Process {
            path: self.executable.clone(),
            args,
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "5s".into(),
        })
    }

    fn is_cl_client(&self) -> bool { true }
}
