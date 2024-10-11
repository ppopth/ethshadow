use crate::config::ethshadow::DEFAULT_GENESIS_GEN_IMAGE;
use crate::config::FullConfig;
use crate::error::Error;
use crate::network_graph::{generate_network_graph, GeneratedNetworkGraph};
use crate::node::NodeManager;
use crate::validators::ValidatorManager;
use log::{debug, info};
use serde_yaml::Value;
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::fs::{create_dir, File};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

mod clients;
mod config;
mod error;
mod genesis;
mod gml;
mod network_graph;
mod node;
mod utils;
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
    debug!("Reading config file");
    // get the config and extend it with our supported builtins
    let FullConfig {
        mut ethshadow_config,
        mut shadow_config,
    } = config.try_into()?;
    ethshadow_config.add_default_builtins();
    shadow_config.apply_defaults(ethshadow_config.minimum_latency())?;

    debug!("Creating output directory");
    create_dir(&output_path).map_err(|e| match e.kind() {
        ErrorKind::AlreadyExists => Error::OutputFolderExists,
        _ => Error::Io(e),
    })?;
    output_path = output_path.canonicalize()?;
    let dir_path = output_path.clone().into_os_string();

    debug!("Desugaring node config");
    let nodes = ethshadow_config.desugar_nodes()?;

    debug!("Computing validators");
    let validators = ValidatorManager::new(&ethshadow_config, &nodes, &output_path)?;

    info!("Generating genesis information");
    genesis::write_config(
        &ethshadow_config.genesis,
        validators.total_count(),
        output_path.clone(),
    )?;
    genesis::generate(
        ethshadow_config
            .genesis
            .generator_image
            .as_deref()
            .unwrap_or(DEFAULT_GENESIS_GEN_IMAGE),
        dir_path,
    )?;

    debug!("Generating network graph");
    let GeneratedNetworkGraph {
        gml,
        mut network_graph,
    } = generate_network_graph(&ethshadow_config)?;
    shadow_config.set_network(gml, false)?;

    // postprocessing given shadow config values: overwrite string network ids
    for host in shadow_config.hosts_mut()? {
        if let Some(node_id) = host?.network_node_id_mut() {
            if let Some((location, reliability)) = node_id.as_str().and_then(|s| s.split_once("-"))
            {
                let node = network_graph.assign_network_node(location, reliability)?;
                *node_id = Value::Number(node.id().into());
            } else {
                return Err(Error::ExpectedOtherType("network_node_id".to_string()));
            }
        }
    }

    info!("Generating nodes");
    let mut node_manager = NodeManager::new(
        output_path.clone(),
        &nodes,
        &mut shadow_config,
        network_graph,
        validators,
    );
    node_manager.generate_nodes()?;

    info!("Writing finished configuration");
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
