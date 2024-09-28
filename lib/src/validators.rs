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
    total_val: usize,
) -> Result<Vec<Validator>, Error> {
    let mut validators = Vec::with_capacity(total_val);
    let mut idx = 0;
    let mut data_mount = output_path.as_os_str().to_owned();
    data_mount.push(":/data");
    while validators.len() < total_val {
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
            .arg(format!("/data/validator_keys_{idx}"))
            .arg("--source-mnemonic")
            .arg(mnemonic)
            .arg("--source-min")
            .arg(validators.len().to_string())
            .arg("--source-max")
            .arg(min(validators.len() + 4000, total_val).to_string())
            .spawn()?
            .wait()?;
        if !status.success() {
            return Err(Error::ChildProcessFailure(image_name.to_string()));
        }
        let base_path = output_path.join(format!("validator_keys_{idx}"));
        for validator in read_dir(base_path.join("keys"))?.map_ok(|e| Validator {
            base_path: base_path.clone(),
            key: e.file_name(),
        }) {
            validators.push(validator?);
        }
        idx += 1;
    }
    Ok(validators)
}

pub struct Validator {
    base_path: PathBuf,
    key: OsString,
}

impl Validator {
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    pub fn key(&self) -> &OsString {
        &self.key
    }
}
