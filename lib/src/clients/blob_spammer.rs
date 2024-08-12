use crate::clients::Client;
use crate::config::shadow::Process;
use crate::error::Error;
use crate::node::{Node, SimulationContext};
use crate::validators::Validator;
use itertools::Itertools;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct BlobSpammer {
    executable: String,
    private_key: String,
    throughput: u8,
    start_time: String,
}

#[typetag::deserialize(name = "blob_spammer")]
impl Client for BlobSpammer {
    fn add_to_node(
        &self,
        _node: &Node,
        ctx: &mut SimulationContext,
        _validators: &[Validator],
    ) -> Result<Process, Error> {
        Ok(Process {
            path: self.executable.clone().into(),
            args: format!(
                "combined -t {} -p {} -h {}",
                self.throughput,
                self.private_key,
                ctx.el_http_endpoints().iter().join(" -h "),
            ),
            environment: Default::default(),
            expected_final_state: "running".into(),
            start_time: self.start_time.clone().into(),
        })
    }
}
