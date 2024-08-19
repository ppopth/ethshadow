use crate::clients::ValidatorDemand;
use crate::config::ethshadow::{
    default_boot_clients, default_client_stack, Node, NodeConfig, NodeCount,
};
use crate::config::one_or_many::OneOrMany;
use crate::config::FullConfig;
use crate::error::Error;
use crate::error::Error::{LeftoverValidators, MoreValidatorsRequested};
use crate::network_graph::generate_network_graph;
use crate::node::{NodeManager, SimulationContext};
use itertools::Itertools;
use rand::prelude::*;
use serde_yaml::Value;
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::fs::{create_dir, File};
use std::path::PathBuf;
use std::process::Command;

mod clients;
mod config;
mod error;
mod genesis;
mod gml;
mod network_graph;
mod node;
mod validators;

type CowStr = Cow<'static, str>;

pub struct ShadowInvocation {
    command: Command,
    args: Vec<OsString>,
}

impl ShadowInvocation {
    fn new<S: AsRef<OsStr>>(executable: S, args: Vec<OsString>) -> ShadowInvocation {
        ShadowInvocation {
            command: Command::new(executable),
            args,
        }
    }

    pub fn with_user_args<I, S>(&mut self, user_args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(user_args);
    }

    pub fn command(mut self) -> Command {
        self.command.args(self.args);
        self.command
    }
}

pub fn generate<T: TryInto<FullConfig, Error = Error>>(
    config: T,
    mut output_path: PathBuf,
) -> Result<ShadowInvocation, Error> {
    // get the config and extend it with our supported builtins
    let FullConfig {
        mut ethshadow_config,
        mut shadow_config,
    } = config.try_into()?;
    ethshadow_config.add_default_builtins();

    create_dir(&output_path)?;
    output_path = output_path.canonicalize()?;
    let dir_path = output_path.clone().into_os_string();

    // generate genesis
    genesis::write_config(&ethshadow_config, output_path.clone())?;
    genesis::generate(
        ethshadow_config
            .genesis
            .generator_image
            .as_deref()
            .unwrap_or("ethpandaops/ethereum-genesis-generator:3.3.5"),
        dir_path,
    )?;

    // generate network graph
    let network_graph = generate_network_graph(&ethshadow_config)?;
    shadow_config.set_network(network_graph.gml, false)?;

    // postprocessing given shadow config values: overwrite string network ids
    for host in shadow_config.hosts_mut()? {
        if let Some(node_id) = host?.network_node_id_mut() {
            if let Some((location, reliability)) = node_id.as_str().and_then(|s| s.split_once("-"))
            {
                let node = network_graph
                    .nodes
                    .get(&(location, reliability))
                    .ok_or_else(|| {
                        Error::UnknownLocationReliability(
                            location.to_string(),
                            reliability.to_string(),
                        )
                    })?;
                *node_id = Value::Number(node.id().into());
            } else {
                return Err(Error::ExpectedOtherType("network_node_id".to_string()));
            }
        }
    }

    // generate nodes TODO clean up and organize...
    let nodes = match &ethshadow_config.nodes {
        NodeConfig::Simple(count) => {
            // user does NOT care, let's put em all in europe
            &vec![
                Node {
                    locations: OneOrMany::One("europe".into()),
                    reliabilities: OneOrMany::One("reliable".into()),
                    clients: default_boot_clients(),
                    count: NodeCount::TotalCount(1),
                    tag: Some("boot".into()),
                },
                Node {
                    locations: OneOrMany::One("europe".into()),
                    reliabilities: OneOrMany::One("reliable".into()),
                    clients: default_client_stack(),
                    count: NodeCount::TotalCount(*count),
                    tag: None,
                },
            ]
        }
        NodeConfig::Detailed(nodes) => nodes,
    };
    let (val_for_each_any, get_one_val_extra, total_val) = match ethshadow_config.validators {
        Some(validators) => {
            let mut requested = 0;
            let mut anys = 0;
            for node in nodes {
                let count = node.count()?;
                for client_category in node.clients.values() {
                    let count = count / client_category.len() as u64;
                    for client in client_category {
                        let client = ethshadow_config
                            .clients
                            .get(client.as_str())
                            .ok_or_else(|| Error::UnknownClient(client.to_string()))?;
                        match client.validator_demand() {
                            ValidatorDemand::Count(val_count) => requested += val_count * count,
                            ValidatorDemand::Any | ValidatorDemand::AnyNonZero => anys += count,
                            ValidatorDemand::None => {}
                        }
                    }
                }
            }
            let Some(remaining) = validators.checked_sub(requested) else {
                return Err(MoreValidatorsRequested(validators, requested));
            };
            if anys != 0 {
                (remaining / anys, remaining % anys, validators)
            } else if remaining != 0 {
                return Err(LeftoverValidators);
            } else {
                (0, 0, validators)
            }
        }
        None => {
            let mut total = 0;
            for node in nodes {
                let count = node.count()?;
                for client_category in node.clients.values() {
                    let count = count / client_category.len() as u64;
                    for client in client_category {
                        let client = ethshadow_config
                            .clients
                            .get(client.as_str())
                            .ok_or_else(|| Error::UnknownClient(client.to_string()))?;
                        if let ValidatorDemand::Count(val_count) = client.validator_demand() {
                            total += val_count * count;
                        }
                    }
                }
            }
            (0, 0, total) // maybe (1, 0) is preferable? :thinking:
        }
    };
    let (validator_dir, validators) = validators::generate(
        ethshadow_config
            .genesis
            .generator_image
            .as_deref()
            .unwrap_or("ethpandaops/ethereum-genesis-generator:3.3.5"),
        &output_path,
        ethshadow_config
            .genesis
            .mnemonic
            .as_deref()
            .unwrap_or(genesis::DEFAULT_MNEMONIC),
        total_val,
    )?;
    let mut val_idx = 0;
    let mut validators = |count: u64| {
        let end = val_idx + (count as usize);
        let slice = &validators[val_idx..end];
        val_idx = end;
        slice
    };
    let rng = StdRng::seed_from_u64(shadow_config.seed()?);
    let ctx = SimulationContext::new(
        rng,
        output_path.join("metadata"),
        validator_dir,
        output_path.join("jwt/jwtsecret"),
    );
    let mut node_manager = NodeManager::new(
        ctx,
        output_path.clone(),
        &mut shadow_config,
        network_graph.nodes,
    );
    let mut encountered_anys = 0;
    for node in nodes {
        for location in &node.locations {
            for reliability in &node.reliabilities {
                for client_stack in node
                    .clients
                    .values()
                    .map(|clients| {
                        clients.into_iter().map(|client| {
                            ethshadow_config
                                .clients
                                .get(client.as_str())
                                .ok_or_else(|| client.to_string())
                        })
                    })
                    .multi_cartesian_product()
                {
                    for _ in 0..(node.count()? / node.combinations()) {
                        let client_stack: Vec<_> = client_stack
                            .iter()
                            .filter_map(|client| match client {
                                Ok(client) => match client.validator_demand() {
                                    ValidatorDemand::None => Some(Ok((client.as_ref(), 0))),
                                    demand @ ValidatorDemand::Any
                                    | demand @ ValidatorDemand::AnyNonZero => {
                                        let mut val_count = val_for_each_any;
                                        if encountered_anys < get_one_val_extra {
                                            val_count += 1;
                                        }
                                        encountered_anys += 1;
                                        if val_count > 0 || matches!(demand, ValidatorDemand::Any) {
                                            Some(Ok((client.as_ref(), val_count)))
                                        } else {
                                            None
                                        }
                                    }
                                    ValidatorDemand::Count(count) => {
                                        Some(Ok((client.as_ref(), count)))
                                    }
                                },
                                Err(client) => Some(Err(Error::UnknownClient(client.clone()))),
                            })
                            .try_collect()?;
                        let client_stack: Vec<_> = client_stack
                            .iter()
                            .map(|(client, val_count)| (*client, validators(*val_count)))
                            .collect();
                        node_manager.gen_node(
                            node.tag.as_deref().unwrap_or(""),
                            &client_stack,
                            location,
                            reliability,
                        )?;
                    }
                }
            }
        }
    }

    // write modified shadow.yaml to disk
    output_path.push("shadow.yaml");
    serde_yaml::to_writer(File::create_new(&output_path)?, &shadow_config.0)?;
    let config_path = output_path.as_os_str().to_owned();
    output_path.pop();

    output_path.push("shadow");
    Ok(ShadowInvocation::new(
        ethshadow_config.shadow_path.as_deref().unwrap_or("shadow"),
        vec!["-d".into(), output_path.into_os_string(), config_path],
    ))
}
