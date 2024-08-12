use crate::error::Error;
use crate::CowStr;
use serde::Serialize;
use serde_yaml::{to_value, Mapping};
use std::collections::HashMap;

macro_rules! get_as {
    ($map:expr, $string:literal, $accessor:ident) => {
        $map.get($string).map(|v| {
            v.$accessor()
                .ok_or_else(|| Error::ExpectedOtherType($string.to_string()))
        })
    };
}

/// A light wrapper around a yaml mapping representing the root of a shadow config with some useful
/// setters and getters
///
/// The reason why we do not do this "properly" by defining a serde struct is that we want to
/// support all the options, and that is too complex. Unfortunately, Shadow does not expose
/// its config structs in a separate crate, and compiling all of Shadow for that is not nice.
pub struct ShadowConfig(pub Mapping);

#[derive(Serialize, Debug, Clone)]
pub struct Host {
    pub ip_addr: String,
    pub network_node_id: u64,
    pub processes: Vec<Process>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Process {
    pub path: CowStr,
    pub args: String,
    pub environment: HashMap<CowStr, CowStr>,
    pub expected_final_state: CowStr,
    pub start_time: CowStr,
}

impl ShadowConfig {
    fn general(&self) -> Result<Option<&Mapping>, Error> {
        get_as!(self.0, "general", as_mapping).transpose()
    }

    pub fn seed(&self) -> Result<u64, Error> {
        self.general()?
            .and_then(|g| get_as!(g, "seed", as_u64))
            .unwrap_or(Ok(1))
    }

    pub fn add_host(&mut self, hostname: String, host: &Host) -> Result<(), Error> {
        let hosts = self
            .0
            .entry("hosts".into())
            .or_insert_with(|| Mapping::new().into())
            .as_mapping_mut()
            .ok_or_else(|| Error::ExpectedOtherType("hosts".to_string()))?;
        if hosts
            .insert(hostname.clone().into(), to_value(host)?)
            .is_some()
        {
            Err(Error::NameConflict(hostname))
        } else {
            Ok(())
        }
    }

    pub fn set_network(&mut self, gml: String, use_shortest_path: bool) -> Result<(), Error> {
        let mut network = Mapping::new();
        network.insert("use_shortest_path".into(), use_shortest_path.into());
        let mut graph = Mapping::new();
        graph.insert("type".into(), "gml".into());
        graph.insert("inline".into(), gml.into());
        network.insert("graph".into(), graph.into());
        if self.0.insert("network".into(), network.into()).is_some() {
            Err(Error::ExistingNetwork)
        } else {
            Ok(())
        }
    }
}
