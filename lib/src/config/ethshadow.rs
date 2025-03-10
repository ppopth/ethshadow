use crate::clients::geth::Geth;
use crate::clients::geth_bootnode::GethBootnode;
use crate::clients::lighthouse::Lighthouse;
use crate::clients::lighthouse_bootnode::LighthouseBootnode;
use crate::clients::lighthouse_vc::LighthouseValidatorClient;
use crate::clients::prometheus::Prometheus;
use crate::clients::prysm::Prysm;
use crate::clients::reth::Reth;
use crate::clients::Client;
use crate::config::one_or_many::OneOrMany;
use crate::error::Error;
use crate::CowStr;
use humantime_serde::Serde as HumanReadable;
use itertools::Itertools;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

/// Options contained in the configuration file.
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct EthShadowConfig {
    #[serde(deserialize_with = "deserialize_nodes")]
    nodes: Vec<SugaredNode>,
    pub locations: HashMap<CowStr, Location>,
    pub reliabilities: HashMap<CowStr, Reliability>,
    pub validators: Option<usize>,
    pub clients: HashMap<CowStr, Box<dyn Client>>,
    #[serde(default = "default_clients")]
    pub default_clients: HashMap<CowStr, CowStr>,
    pub genesis: Genesis,
    pub topology: Topology,
    pub shadow_path: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum NodeConfig {
    Simple(usize),
    Detailed(Vec<SugaredNode>),
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig::Simple(4)
    }
}

fn deserialize_nodes<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<SugaredNode>, D::Error> {
    Ok(match NodeConfig::deserialize(d)? {
        NodeConfig::Simple(count) => vec![
            SugaredNode {
                locations: OneOrMany::One("europe".into()),
                reliabilities: OneOrMany::One("reliable".into()),
                clients: default_boot_clients(),
                count: NodeCount::TotalCount(1),
                tag: Some("boot".into()),
            },
            SugaredNode {
                locations: OneOrMany::One("europe".into()),
                reliabilities: OneOrMany::One("reliable".into()),
                clients: HashMap::new(),
                count: NodeCount::TotalCount(count),
                tag: None,
            },
        ],
        NodeConfig::Detailed(vec) => vec,
    })
}

#[derive(Deserialize, Clone, Debug)]
struct SugaredNode {
    #[serde(alias = "location")]
    pub locations: OneOrMany<String>,
    #[serde(alias = "reliability")]
    pub reliabilities: OneOrMany<String>,
    #[serde(default)]
    pub clients: HashMap<String, OneOrMany<String>>,
    #[serde(default)]
    pub count: NodeCount,
    #[serde(default)]
    pub tag: Option<String>,
}

impl SugaredNode {
    fn combinations(&self) -> usize {
        self.locations.len()
            * self.reliabilities.len()
            * self.clients.values().map(OneOrMany::len).product::<usize>()
    }

    fn count_per_combination(&self) -> Result<usize, Error> {
        match self.count {
            NodeCount::CountPerCombination(count) => Ok(count),
            NodeCount::TotalCount(count) => {
                if count % self.combinations() == 0 {
                    Ok(count / self.combinations())
                } else {
                    Err(Error::InconsistentCount(count, self.combinations()))
                }
            }
        }
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
enum NodeCount {
    #[serde(rename = "per_combination")]
    CountPerCombination(usize),
    #[serde(rename = "total")]
    TotalCount(usize),
}

impl Default for NodeCount {
    fn default() -> Self {
        NodeCount::CountPerCombination(1)
    }
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct Location {
    pub latency_to: HashMap<CowStr, HumanReadable<Duration>>,
    pub packet_loss_to: HashMap<CowStr, f32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Reliability {
    pub added_latency: HumanReadable<Duration>,
    pub added_packet_loss: f32,
    pub bandwidth_up: CowStr,
    pub bandwidth_down: CowStr,
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Genesis {
    pub generator_image: Option<String>,
    pub preset_base: Option<String>,
    pub chain_id: Option<u64>,
    pub deposit_contract_address: Option<String>,
    pub mnemonic: Option<String>,
    pub capella_epoch: Option<u64>,
    pub deneb_epoch: Option<u64>,
    pub electra_epoch: Option<u64>,
    pub fulu_epoch: Option<u64>,
    pub eip7594_epoch: Option<u64>,
    pub withdrawal_address: Option<String>,
    pub delay: Option<u64>,
    pub gaslimit: Option<u64>,
    pub max_per_epoch_activation_churn_limit: Option<u64>,
    pub churn_limit_quotient: Option<u64>,
    pub ejection_balance: Option<u64>,
    pub eth1_follow_distance: Option<u64>,
    pub min_validator_withdrawability_delay: Option<u64>,
    pub shard_committee_period: Option<u64>,
    pub samples_per_slot: Option<u64>,
    pub custody_requirement: Option<u64>,
    pub data_column_sidecar_subnet_count: Option<u64>,
    pub max_blobs_per_block: Option<u64>,
    pub premine: Option<HashMap<String, String>>,
}

#[derive(Default, Deserialize, Clone, Debug)]
pub enum Topology {
    #[default]
    Simple,
    Clustered(Vec<Cluster>),
}

#[derive(Deserialize, Clone, Debug)]
pub struct Cluster {
    pub gateway_latency: u64,
    pub cluster_latencies: Vec<u64>,
}

pub fn default_clients() -> HashMap<CowStr, CowStr> {
    [
        ("el".into(), "geth".into()),
        ("cl".into(), "lighthouse".into()),
        ("vc".into(), "lighthouse_vc".into()),
    ]
    .into_iter()
    .collect()
}

pub fn default_boot_clients() -> HashMap<String, OneOrMany<String>> {
    [
        ("el".into(), OneOrMany::One("geth_bootnode".into())),
        ("cl".into(), OneOrMany::One("lighthouse_bootnode".into())),
    ]
    .into_iter()
    .collect()
}

impl EthShadowConfig {
    pub fn add_default_builtins(&mut self) {
        self.add_builtin_location(
            "australia",
            [
                ("australia", 2, 0.0),
                ("east_asia", 110, 0.0),
                ("europe", 165, 0.0),
                ("na_west", 110, 0.0),
                ("na_east", 150, 0.0),
                ("south_america", 190, 0.0),
                ("south_africa", 220, 0.0),
                ("west_asia", 180, 0.0),
            ],
        );
        self.add_builtin_location(
            "east_asia",
            [
                ("australia", 110, 0.0),
                ("east_asia", 4, 0.0),
                ("europe", 125, 0.0),
                ("na_west", 100, 0.0),
                ("na_east", 140, 0.0),
                ("south_america", 175, 0.0),
                ("south_africa", 175, 0.0),
                ("west_asia", 110, 0.0),
            ],
        );
        self.add_builtin_location(
            "europe",
            [
                ("australia", 165, 0.0),
                ("east_asia", 125, 0.0),
                ("europe", 2, 0.0),
                ("na_west", 110, 0.0),
                ("na_east", 70, 0.0),
                ("south_america", 140, 0.0),
                ("south_africa", 95, 0.0),
                ("west_asia", 60, 0.0),
            ],
        );
        self.add_builtin_location(
            "na_west",
            [
                ("australia", 110, 0.0),
                ("east_asia", 100, 0.0),
                ("europe", 110, 0.0),
                ("na_west", 2, 0.0),
                ("na_east", 60, 0.0),
                ("south_america", 100, 0.0),
                ("south_africa", 160, 0.0),
                ("west_asia", 150, 0.0),
            ],
        );
        self.add_builtin_location(
            "na_east",
            [
                ("australia", 150, 0.0),
                ("east_asia", 140, 0.0),
                ("europe", 70, 0.0),
                ("na_west", 60, 0.0),
                ("na_east", 2, 0.0),
                ("south_america", 100, 0.0),
                ("south_africa", 130, 0.0),
                ("west_asia", 110, 0.0),
            ],
        );
        self.add_builtin_location(
            "south_america",
            [
                ("australia", 190, 0.0),
                ("east_asia", 175, 0.0),
                ("europe", 140, 0.0),
                ("na_west", 100, 0.0),
                ("na_east", 100, 0.0),
                ("south_america", 7, 0.0),
                ("south_africa", 195, 0.0),
                ("west_asia", 145, 0.0),
            ],
        );
        self.add_builtin_location(
            "south_africa",
            [
                ("australia", 220, 0.0),
                ("east_asia", 175, 0.0),
                ("europe", 95, 0.0),
                ("na_west", 160, 0.0),
                ("na_east", 130, 0.0),
                ("south_america", 190, 0.0),
                ("south_africa", 7, 0.0),
                ("west_asia", 110, 0.0),
            ],
        );
        self.add_builtin_location(
            "west_asia",
            [
                ("australia", 180, 0.0),
                ("east_asia", 110, 0.0),
                ("europe", 60, 0.0),
                ("na_west", 150, 0.0),
                ("na_east", 110, 0.0),
                ("south_america", 145, 0.0),
                ("south_africa", 110, 0.0),
                ("west_asia", 5, 0.0),
            ],
        );
        self.add_builtin_reliability(
            "reliable",
            Reliability {
                added_latency: Duration::ZERO.into(),
                added_packet_loss: 0.0,
                bandwidth_up: "1 Gbit".into(),
                bandwidth_down: "1 Gbit".into(),
            },
        );
        self.add_builtin_reliability(
            "home",
            Reliability {
                added_latency: Duration::from_millis(20).into(),
                added_packet_loss: 0.001,
                bandwidth_up: "50 Mbit".into(),
                bandwidth_down: "50 Mbit".into(),
            },
        );
        self.add_builtin_reliability(
            "laggy",
            Reliability {
                added_latency: Duration::from_millis(300).into(),
                added_packet_loss: 0.05,
                bandwidth_up: "50 Mbit".into(),
                bandwidth_down: "50 Mbit".into(),
            },
        );
        self.add_builtin_reliability(
            "constrained",
            Reliability {
                added_latency: Duration::from_millis(20).into(),
                added_packet_loss: 0.001,
                bandwidth_up: "5 Mbit".into(),
                bandwidth_down: "5 Mbit".into(),
            },
        );
        self.add_builtin_reliability(
            "bad",
            Reliability {
                added_latency: Duration::from_millis(500).into(),
                added_packet_loss: 0.2,
                bandwidth_up: "2 Mbit".into(),
                bandwidth_down: "2 Mbit".into(),
            },
        );
        self.add_builtin_client("geth_bootnode", GethBootnode::default());
        self.add_builtin_client("lighthouse_bootnode", LighthouseBootnode::default());
        self.add_builtin_client("geth", Geth::default());
        self.add_builtin_client("reth", Reth::default());
        self.add_builtin_client("prysm", Prysm::default());
        self.add_builtin_client("lighthouse", Lighthouse::default());
        self.add_builtin_client("lighthouse_vc", LighthouseValidatorClient::default());
        self.add_builtin_client("prometheus", Prometheus::default());
    }

    pub fn add_builtin_location<const N: usize>(
        &mut self,
        name: &'static str,
        params: [(&'static str, u64, f32); N],
    ) {
        let location = self.locations.entry(name.into()).or_default();
        for (other, latency, loss) in params {
            location
                .latency_to
                .entry(other.into())
                .or_insert_with(|| HumanReadable::from(Duration::from_millis(latency)));
            location.packet_loss_to.entry(other.into()).or_insert(loss);
        }
    }

    pub fn add_builtin_reliability(&mut self, name: &'static str, params: Reliability) {
        self.reliabilities.entry(name.into()).or_insert(params);
    }

    pub fn add_builtin_client<C: Client + 'static>(&mut self, name: &'static str, client: C) {
        self.clients.entry(name.into()).or_insert(Box::new(client));
    }

    pub fn minimum_latency(&self) -> Duration {
        self.locations
            .values()
            .flat_map(|loc| loc.latency_to.values())
            .map(|latency| latency.into_inner())
            .min()
            .expect("latencies should be specified at this point")
    }

    pub fn desugar_nodes(&self) -> Result<Vec<Node>, Error> {
        let mut result = vec![];

        for node in &self.nodes {
            let clients: Vec<Vec<_>> = if !node.clients.is_empty() {
                node.clients
                    .values()
                    .map(|clients| {
                        clients
                            .iter()
                            .map(|client| {
                                self.clients
                                    .get(client.as_str())
                                    .map(AsRef::as_ref)
                                    .ok_or_else(|| Error::UnknownClient(client.clone()))
                            })
                            .try_collect()
                    })
                    .try_collect()?
            } else {
                self.default_clients
                    .values()
                    .map(|client| {
                        self.clients
                            .get(client)
                            .map(|b| vec![b.as_ref()])
                            .ok_or_else(|| Error::UnknownClient(client.to_string()))
                    })
                    .try_collect()?
            };
            for location in &node.locations {
                for reliability in &node.reliabilities {
                    for clients in clients
                        .iter()
                        .map(|vec| vec.iter().copied())
                        .multi_cartesian_product()
                    {
                        result.push(Node {
                            location,
                            reliability,
                            clients,
                            count: node.count_per_combination()?,
                            tag: node.tag.as_deref(),
                        });
                    }
                }
            }
        }

        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub struct Node<'a> {
    pub location: &'a str,
    pub reliability: &'a str,
    pub clients: Vec<&'a dyn Client>,
    pub count: usize,
    pub tag: Option<&'a str>,
}

pub const DEFAULT_GENESIS_GEN_IMAGE: &str = "ethpandaops/ethereum-genesis-generator:3.7.0";
pub const DEFAULT_MNEMONIC: &str = "\
iron oxygen will win \
iron oxygen will win \
iron oxygen will win \
iron oxygen will win \
iron oxygen will win \
iron oxygen will toe";
