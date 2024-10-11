use crate::config::ethshadow::Topology;
use crate::config::EthShadowConfig;
use crate::gml::{Gml, NetworkNode};
use crate::Error;
use std::collections::BTreeMap;

pub struct GeneratedNetworkGraph<'a> {
    pub gml: String,
    pub network_graph: Box<dyn NetworkGraph + 'a>,
}

pub fn generate_network_graph(config: &EthShadowConfig) -> Result<GeneratedNetworkGraph, Error> {
    match &config.topology {
        Topology::Simple => SimpleNetworkGraph::generate(config),
        Topology::Clustered(_clusters) => todo!(),
    }
}

pub trait NetworkGraph {
    fn assign_network_node(
        &mut self,
        location: &str,
        reliability: &str,
    ) -> Result<NetworkNode, Error>;
}

pub struct SimpleNetworkGraph<'a> {
    nodes: BTreeMap<&'a str, BTreeMap<&'a str, NetworkNode>>,
}

impl SimpleNetworkGraph<'_> {
    pub fn generate(config: &EthShadowConfig) -> Result<GeneratedNetworkGraph, Error> {
        let mut network_graph = Box::new(SimpleNetworkGraph {
            nodes: BTreeMap::new(),
        });
        let mut gml = String::new();
        let mut gml_builder = Gml::new(&mut gml, true)?;
        for location_name in config.locations.keys() {
            for (reliability_name, reliability) in config.reliabilities.iter() {
                let node = gml_builder.add_node(
                    &reliability.bandwidth_up,
                    &reliability.bandwidth_down,
                    Some(&format!("{location_name}-{reliability_name}")),
                )?;
                network_graph
                    .nodes
                    .entry(location_name)
                    .or_default()
                    .insert(reliability_name, node);
            }
        }
        for (src_location_name, src_location) in config.locations.iter() {
            for (src_reliability_name, src_reliability) in config.reliabilities.iter() {
                let src_node =
                    network_graph.get_network_node(src_location_name, src_reliability_name)?;
                for dest_location_name in config.locations.keys() {
                    for (dest_reliability_name, dest_reliability) in config.reliabilities.iter() {
                        let dest_node = network_graph
                            .get_network_node(dest_location_name, dest_reliability_name)?;
                        let mut latency = src_location
                            .latency_to
                            .get(dest_location_name)
                            .ok_or_else(|| {
                                Error::MissingInfoForDestination(
                                    src_location_name.to_string(),
                                    dest_location_name.to_string(),
                                )
                            })?
                            .into_inner();
                        let mut packet_loss = *src_location
                            .packet_loss_to
                            .get(dest_location_name)
                            .ok_or_else(|| {
                                Error::MissingInfoForDestination(
                                    src_location_name.to_string(),
                                    dest_location_name.to_string(),
                                )
                            })?;
                        latency += src_reliability.added_latency.into_inner()
                            + dest_reliability.added_latency.into_inner();
                        packet_loss +=
                            src_reliability.added_packet_loss + dest_reliability.added_packet_loss;
                        gml_builder.add_edge(
                            src_node,
                            dest_node,
                            latency,
                            packet_loss,
                            Some(&format!(
                                "{src_location_name}-{src_reliability_name} to \
                                {dest_location_name}-{dest_reliability_name}"
                            )),
                        )?
                    }
                }
            }
        }
        gml_builder.finish()?;
        Ok(GeneratedNetworkGraph { gml, network_graph })
    }

    fn get_network_node(&self, location: &str, reliability: &str) -> Result<NetworkNode, Error> {
        Ok(*self
            .nodes
            .get(location)
            .ok_or_else(|| Error::UnknownLocation(location.to_string()))?
            .get(reliability)
            .ok_or_else(|| Error::UnknownReliability(reliability.to_string()))?)
    }
}

impl NetworkGraph for SimpleNetworkGraph<'_> {
    fn assign_network_node(
        &mut self,
        location: &str,
        reliability: &str,
    ) -> Result<NetworkNode, Error> {
        self.get_network_node(location, reliability)
    }
}
