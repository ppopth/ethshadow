use crate::clients::Client;
use crate::config::ethshadow::Node;
use crate::config::shadow::Host;
use crate::config::ShadowConfig;
use crate::network_graph::NetworkGraph;
use crate::validators::ValidatorManager;
use crate::Error;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir, File};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

pub struct NodeManager<'c, 'n> {
    ctx: SimulationContext<'n>,
    base_dir: PathBuf,
    nodes: &'n [Node<'n>],
    shadow_config: &'c mut ShadowConfig,
    network_nodes: Box<dyn NetworkGraph + 'n>,
    validator_manager: ValidatorManager,
    used_ips: HashSet<Ipv4Addr>,
}

pub struct NodeInfo<'a> {
    ip: Ipv4Addr,
    dir: PathBuf,
    location: &'a str,
    reliability: &'a str,
}

impl<'c, 'n> NodeManager<'c, 'n> {
    pub fn new(
        base_dir: PathBuf,
        nodes: &'n [Node<'n>],
        shadow_config: &'c mut ShadowConfig,
        network_nodes: Box<dyn NetworkGraph + 'n>,
        validator_manager: ValidatorManager,
    ) -> Self {
        let mut num_el_clients = 0;
        let mut num_cl_clients = 0;
        for node in nodes {
            for client in &node.clients {
                if client.is_el_client() {
                    num_el_clients += node.count;
                }
                if client.is_cl_client() {
                    num_cl_clients += node.count;
                }
            }
        }
        let rng = StdRng::seed_from_u64(shadow_config.seed());
        let ctx = SimulationContext::new(
            rng,
            base_dir.join("metadata"),
            base_dir.join("jwt/jwtsecret"),
            num_el_clients as usize,
            num_cl_clients as usize,
        );
        Self {
            ctx,
            base_dir,
            nodes,
            shadow_config,
            network_nodes,
            validator_manager,
            used_ips: HashSet::new(),
        }
    }

    fn gen_node(
        &mut self,
        tag: &str,
        clients: &[&dyn Client],
        location: &'n str,
        reliability: &'n str,
    ) -> Result<(), Error> {
        let idx = self.used_ips.len();
        let name = format!("node{idx}{tag}");

        let dir = self.base_dir.join(&name);
        create_dir(&dir)?;

        File::create_new(dir.join(location))?;
        File::create_new(dir.join(reliability))?;

        let mut ip = random_ip(self.ctx.rng());
        while !self.used_ips.insert(ip) {
            ip = random_ip(self.ctx.rng());
        }

        let node = NodeInfo {
            ip,
            dir,
            location,
            reliability,
        };

        let mut host = Host {
            ip_addr: ip.to_string(),
            network_node_id: self
                .network_nodes
                .assign_network_node(location, reliability)?
                .id(),
            processes: vec![],
        };

        for &client in clients {
            let validators = self.validator_manager.assign(client);
            let process = client.add_to_node(&node, &mut self.ctx, validators)?;
            host.processes.push(process);
        }

        self.shadow_config.add_host(name, &host)?;

        Ok(())
    }

    pub fn generate_nodes(&mut self) -> Result<(), Error> {
        for node in self.nodes {
            for _ in 0..node.count {
                self.gen_node(
                    node.tag.unwrap_or(""),
                    &node.clients,
                    node.location,
                    node.reliability,
                )?;
            }
        }
        Ok(())
    }
}

impl<'a> NodeInfo<'a> {
    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    pub fn dir(&self) -> &Path {
        self.dir.as_path()
    }

    pub fn location(&self) -> &'a str {
        self.location
    }

    pub fn reliability(&self) -> &'a str {
        self.reliability
    }
}

// we want to avoid hitting a reserved IP range, as that might invoke special behavior in clients.
// we also want to distribute the addresses as wide as possible, as e.g. `bootnode` has buckets
// for IP ranges. As there are a lot of reserved IP ranges, we don't bother having each possible
// IP, and choose 11-99 for the first octet. The last octet will be chosen within 1-254, as the
// first and last IP of a subnet is special.
fn random_ip<R: Rng>(rng: &mut R) -> Ipv4Addr {
    Ipv4Addr::new(
        rng.gen_range(11..=99),
        rng.gen(),
        rng.gen(),
        rng.gen_range(1..=254),
    )
}

pub struct SimulationContext<'a> {
    rng: StdRng,
    metadata_path: PathBuf,
    jwt_path: PathBuf,
    el_bootnode_enodes: Vec<String>,
    cl_bootnode_enrs: Vec<String>,
    el_http_endpoints: Vec<String>,
    cl_http_endpoints: Vec<String>,
    cl_monitoring_endpoints: HashMap<(&'a str, &'a str), Vec<String>>,
    num_el_clients: usize,
    num_cl_clients: usize,
}

impl<'a> SimulationContext<'a> {
    pub fn new(
        rng: StdRng,
        metadata_path: PathBuf,
        jwt_path: PathBuf,
        num_el_clients: usize,
        num_cl_clients: usize,
    ) -> Self {
        Self {
            rng,
            metadata_path,
            jwt_path,
            el_bootnode_enodes: vec![],
            cl_bootnode_enrs: vec![],
            el_http_endpoints: Vec::with_capacity(num_el_clients),
            cl_http_endpoints: Vec::with_capacity(num_cl_clients),
            cl_monitoring_endpoints: HashMap::with_capacity(num_cl_clients),
            num_el_clients,
            num_cl_clients,
        }
    }

    pub fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    pub fn metadata_path(&self) -> &Path {
        self.metadata_path.as_path()
    }

    pub fn jwt_path(&self) -> &Path {
        self.jwt_path.as_path()
    }

    pub fn el_bootnode_enodes(&self) -> &[String] {
        self.el_bootnode_enodes.as_slice()
    }

    pub fn cl_bootnode_enrs(&self) -> &[String] {
        self.cl_bootnode_enrs.as_slice()
    }

    pub fn el_http_endpoints(&self) -> &[String] {
        self.el_http_endpoints.as_slice()
    }

    pub fn cl_http_endpoints(&self) -> &[String] {
        self.cl_http_endpoints.as_slice()
    }

    pub fn cl_monitoring_endpoints(&self) -> &HashMap<(&str, &str), Vec<String>> {
        &self.cl_monitoring_endpoints
    }

    pub fn add_el_bootnode_enode(&mut self, enode: String) {
        self.el_bootnode_enodes.push(enode);
    }

    pub fn add_cl_bootnode_enr(&mut self, enr: String) {
        self.cl_bootnode_enrs.push(enr);
    }

    pub fn add_el_http_endpoint(&mut self, endpoint: String) {
        self.el_http_endpoints.push(endpoint);
    }

    pub fn add_cl_http_endpoint(&mut self, endpoint: String) {
        self.cl_http_endpoints.push(endpoint);
    }

    pub fn add_cl_monitoring_endpoint(
        &mut self,
        location: &'a str,
        reliability: &'a str,
        endpoint: String,
    ) {
        self.cl_monitoring_endpoints
            .entry((location, reliability))
            .or_default()
            .push(endpoint);
    }

    pub fn num_el_clients(&self) -> usize {
        self.num_el_clients
    }

    pub fn num_cl_clients(&self) -> usize {
        self.num_cl_clients
    }
}
