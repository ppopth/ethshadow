use crate::config::EthShadowConfig;
use crate::error::Error;
use crate::gml::{Gml, NetworkNode};
use std::collections::BTreeMap;

pub struct NetworkGraph<'a> {
    pub gml: String,
    pub nodes: BTreeMap<(&'a str, &'a str), NetworkNode>,
}

pub fn generate_network_graph(config: &EthShadowConfig) -> Result<NetworkGraph, Error> {
    let mut gml = String::new();
    let mut nodes = BTreeMap::<(&str, &str), NetworkNode>::new();
    let mut graph = Gml::new(&mut gml, true)?;
    for location_name in config.locations.keys() {
        for (reliability_name, reliability) in config.reliabilities.iter() {
            let node = graph.add_node(
                &reliability.bandwidth_up,
                &reliability.bandwidth_down,
                Some(&format!("{location_name}-{reliability_name}")),
            )?;
            nodes.insert((location_name, reliability_name), node);
        }
    }
    for (source_location_name, source_location) in config.locations.iter() {
        for (source_reliability_name, source_reliability) in config.reliabilities.iter() {
            let source_node = nodes
                .get(&(source_location_name, source_reliability_name))
                .expect("we JUST added all combinations");
            for dest_location_name in config.locations.keys() {
                for (dest_reliability_name, dest_reliability) in config.reliabilities.iter() {
                    let dest_node = nodes
                        .get(&(dest_location_name, dest_reliability_name))
                        .expect("we JUST added all combinations");
                    let mut latency = source_location
                        .latency_to
                        .get(dest_location_name)
                        .ok_or_else(|| {
                            Error::MissingInfoForDestination(
                                source_location_name.to_string(),
                                dest_location_name.to_string(),
                            )
                        })?
                        .into_inner();
                    let mut packet_loss = *source_location
                        .packet_loss_to
                        .get(dest_location_name)
                        .ok_or_else(|| {
                        Error::MissingInfoForDestination(
                            source_location_name.to_string(),
                            dest_location_name.to_string(),
                        )
                    })?;
                    latency += source_reliability.added_latency.into_inner()
                        + dest_reliability.added_latency.into_inner();
                    packet_loss +=
                        source_reliability.added_packet_loss + dest_reliability.added_packet_loss;
                    graph.add_edge(
                        *source_node,
                        *dest_node,
                        latency,
                        packet_loss,
                        Some(&format!(
                            "{source_location_name}-{source_reliability_name} to \
                                {dest_location_name}-{dest_reliability_name}"
                        )),
                    )?
                }
            }
        }
    }
    graph.finish()?;
    Ok(NetworkGraph { gml, nodes })
}
