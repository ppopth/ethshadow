use crate::clients::Client;
use crate::clients::CommonArgs;
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::Validator;
use crate::Error;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Blobssss {
    #[serde(flatten)]
    pub common: CommonArgs,
    pub private_key: String,
    pub min_per_slot: u8,
    pub max_per_slot: u8,
    pub start_time: String,
}

#[typetag::deserialize(name = "blobssss")]
impl Client for Blobssss {
    fn add_to_node(
        &self,
        _node: &NodeInfo,
        ctx: &mut SimulationContext,
        _validators: &[Validator],
    ) -> Result<Process, Error> {
        Ok(Process {
            path: self.common.executable_or("blobssss"),
            args: format!(
                "--min {} --max {} --key {} --rpcs {} {}",
                self.min_per_slot,
                self.max_per_slot,
                self.private_key,
                ctx.el_http_endpoints().iter().join(","),
                self.common.extra_args,
            ),
            environment: HashMap::default(),
            expected_final_state: "running".into(),
            start_time: self.start_time.clone().into(),
        })
    }
}
