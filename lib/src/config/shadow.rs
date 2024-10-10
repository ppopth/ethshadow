use crate::error::Error;
use crate::CowStr;
use serde::Serialize;
use serde_yaml::mapping::IterMut;
use serde_yaml::{to_value, Mapping, Value};
use std::collections::HashMap;
use std::time::Duration;

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
    pub fn general_mut(&mut self) -> Result<&mut Mapping, Error> {
        self.0
            .entry("general".into())
            .or_insert(Mapping::new().into())
            .as_mapping_mut()
            .ok_or_else(|| Error::ExpectedOtherType("general".into()))
    }

    pub fn experimental_mut(&mut self) -> Result<&mut Mapping, Error> {
        self.0
            .entry("experimental".into())
            .or_insert(Mapping::new().into())
            .as_mapping_mut()
            .ok_or_else(|| Error::ExpectedOtherType("experimental".into()))
    }

    pub fn seed(&self) -> u64 {
        self.0
            .get("general")
            .and_then(Value::as_mapping)
            .and_then(|m| m.get("seed"))
            .and_then(Value::as_u64)
            .unwrap_or(1)
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

    pub fn hosts_mut(&mut self) -> Result<HostsMut, Error> {
        Ok(HostsMut {
            hosts: self
                .0
                .get_mut("hosts")
                .map(|h| {
                    h.as_mapping_mut()
                        .ok_or_else(|| Error::ExpectedOtherType("hosts".to_string()))
                })
                .transpose()?
                .map(|h| h.iter_mut()),
        })
    }

    pub fn apply_defaults(&mut self, runahead: Duration) -> Result<(), Error> {
        let general = self.general_mut()?;
        general
            .entry("model_unblocked_syscall_latency".into())
            .or_insert(Value::Bool(true));
        general
            .entry("heartbeat_interval".into())
            .or_insert_with(|| Value::String("1m".into()));
        let experimental = self.experimental_mut()?;
        // todo: warn user if they override runahead with a value larger than the min latency
        experimental
            .entry("runahead".into())
            .or_insert_with(|| format!("{}ns", runahead.as_nanos()).into());
        experimental
            .entry("use_memory_manager".into())
            .or_insert(Value::Bool(true));
        experimental
            .entry("host_heartbeat_interval".into())
            .or_insert_with(|| Value::String("1m".into()));
        Ok(())
    }
}

pub struct HostsMut<'a> {
    hosts: Option<IterMut<'a>>,
}

impl<'a> Iterator for HostsMut<'a> {
    type Item = Result<UntypedHost<&'a mut Mapping>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.hosts
            .as_mut()
            .and_then(|hosts| hosts.next())
            .map(|(_, host)| {
                host.as_mapping_mut()
                    .ok_or_else(|| Error::ExpectedOtherType("host".to_string()))
                    .map(UntypedHost)
            })
    }
}

pub struct UntypedHost<T>(T);

impl<'a> UntypedHost<&'a mut Mapping> {
    pub fn network_node_id_mut(&mut self) -> Option<&mut Value> {
        self.0.get_mut("network_node_id")
    }
}
