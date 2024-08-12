use std::collections::HashMap;
use std::fs::{create_dir, File};
use std::io::Write;

use libsecp256k1::{PublicKey, SecretKey};
use serde::Deserialize;

use crate::clients::{Client, Validator};
use crate::config::shadow::Process;
use crate::error::Error;
use crate::node::{Node, SimulationContext};
use crate::CowStr;

const DISC_PORT: u16 = 30305;

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct GethBootnode {
    pub executable: CowStr,
}

impl Default for GethBootnode {
    fn default() -> Self {
        Self {
            executable: "bootnode".into(),
        }
    }
}

#[typetag::deserialize(name = "geth_bootnode")]
impl Client for GethBootnode {
    fn add_to_node(
        &self,
        node: &Node,
        ctx: &mut SimulationContext,
        _validators: &[Validator],
    ) -> Result<Process, Error> {
        let dir = node.dir().join("geth_bootnode");
        create_dir(&dir)?;

        let node_key = SecretKey::random(ctx.rng());
        let pub_key = PublicKey::from_secret_key(&node_key);

        let node_key = hex::encode(node_key.serialize());
        let pub_key = hex::encode(&pub_key.serialize()[1..]);

        let key_file = dir.join("boot.key");
        let mut file = File::create_new(&key_file)?;
        file.write_all(node_key.as_bytes())?;
        let key_file = key_file.to_str().ok_or(Error::NonUTF8Path)?;

        let ip = node.ip();

        ctx.add_el_bootnode_enode(format!("enode://{pub_key}@{ip}:0?discport={DISC_PORT}"));

        Ok(Process {
            path: self.executable.clone(),
            args: format!(
                "-nodekey \"{key_file}\" \
                -verbosity 5 \
                -addr :{DISC_PORT} \
                -nat extip:{ip}"
            ),
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "0s".into(),
        })
    }
}
