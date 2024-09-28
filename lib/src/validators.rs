use crate::error::Error;
use itertools::Itertools;
use std::cmp::min;
use std::ffi::OsString;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use users::get_current_uid;

pub fn generate(
    image_name: &str,
    output_path: &Path,
    mnemonic: &str,
    total_val: u64,
) -> Result<(PathBuf, Vec<Validator>), Error> {
    let mut generated = 0;
    let mut data_mount = output_path.as_os_str().to_owned();
    data_mount.push(":/data");
    while generated < total_val {
        let status = Command::new("docker")
            .args(["run", "--rm", "-i", "-u"])
            .arg(get_current_uid().to_string())
            .arg("-v")
            .arg(&data_mount)
            .arg("--entrypoint=eth2-val-tools")
            .arg(image_name)
            .arg("keystores")
            .arg("--insecure")
            .arg("--out-loc")
            .arg("/data/validator_keys")
            .arg("--source-mnemonic")
            .arg(mnemonic)
            .arg("--source-min")
            .arg(generated.to_string())
            .arg("--source-max")
            .arg(min(generated + 5000, total_val).to_string())
            .spawn()?
            .wait()?;
        if !status.success() {
            return Err(Error::ChildProcessFailure(image_name.to_string()));
        }
        generated += 5000;
    }
    let mut path = output_path.join("validator_keys/keys");
    let validators = read_dir(&path)?
        .map_ok(|e| Validator { key: e.file_name() })
        .try_collect()?;
    path.pop();
    Ok((path, validators))
}

pub struct Validator {
    key: OsString,
}

impl Validator {
    pub fn key(&self) -> &OsString {
        &self.key
    }
}
