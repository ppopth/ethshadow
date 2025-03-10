use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::ValidatorSet;
use crate::CowStr;
use crate::Error;
use serde::Deserialize;
use std::fmt::Debug;

const ENGINE_API_PORT: &str = "21001";
const JSON_RPC_PORT: &str = "22001";
const BEACON_API_PORT: &str = "31001";
const CL_PROMETHEUS_PORT: &str = "32001";

pub mod blobssss;
pub mod geth;
pub mod geth_bootnode;
pub mod lighthouse;
pub mod lighthouse_bootnode;
pub mod lighthouse_vc;
pub mod prometheus;
pub mod prysm;
pub mod reth;

pub enum ValidatorDemand {
    /// We do not need validator keys. The validator slice will be empty.
    None,
    /// We want validators, but the user does not care about the amount. If we can't get any,
    /// the validator slice will be empty.
    Any,
    /// We want validators, and the slice will have exactly this amount of elements. Generation
    /// fails if we can not satisfy this.
    Count(usize),
}

#[typetag::deserialize(tag = "type")]
pub trait Client: Debug {
    fn add_to_node<'a>(
        &self,
        node: &NodeInfo<'a>,
        ctx: &mut SimulationContext<'a>,
        validators: &ValidatorSet,
    ) -> Result<Process, Error>;

    fn validator_demand(&self) -> ValidatorDemand {
        ValidatorDemand::None
    }

    fn is_cl_client(&self) -> bool {
        false
    }
    fn is_el_client(&self) -> bool {
        false
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct CommonParams {
    pub executable: String,
    pub extra_args: String,
    pub use_recommended_args: bool,
}

impl Default for CommonParams {
    fn default() -> Self {
        CommonParams {
            executable: String::new(),
            extra_args: String::new(),
            use_recommended_args: true,
        }
    }
}

impl CommonParams {
    pub fn executable_or(&self, default: &'static str) -> CowStr {
        if self.executable.is_empty() {
            default.into()
        } else {
            self.executable.clone().into()
        }
    }

    pub fn arguments(&self, recommended: &'static str) -> String {
        if self.use_recommended_args && !recommended.is_empty() {
            format!("{} {}", recommended, self.extra_args)
        } else {
            self.extra_args.clone()
        }
    }
}
