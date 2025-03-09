use crate::config::ethshadow::{Genesis, DEFAULT_MNEMONIC};
use crate::utils::log_and_wait;
use crate::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use users::get_current_uid;

pub const GENESIS_FORK_VERSION: &str = "0x10000000";

pub fn write_config(
    genesis: &Genesis,
    num_validators: usize,
    mut output_path: PathBuf,
) -> Result<(), Error> {
    output_path.push("values.env");
    let mut file = BufWriter::new(File::create_new(output_path)?);
    let file = &mut file;

    export(
        file,
        "PRESET_BASE",
        genesis.preset_base.as_deref().unwrap_or("mainnet"),
    )?;
    export(file, "CHAIN_ID", genesis.chain_id.unwrap_or(1337))?;
    export(
        file,
        "DEPOSIT_CONTRACT_ADDRESS",
        genesis
            .deposit_contract_address
            .as_deref()
            .unwrap_or("0x4242424242424242424242424242424242424242"),
    )?;
    export(
        file,
        "EL_AND_CL_MNEMONIC",
        genesis.mnemonic.as_deref().unwrap_or(DEFAULT_MNEMONIC),
    )?;
    export(file, "CL_EXEC_BLOCK", "0")?;
    export(file, "SLOT_DURATION_IN_SECONDS", "12")?;
    export(
        file,
        "DEPOSIT_CONTRACT_BLOCK",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )?;
    export(file, "NUMBER_OF_VALIDATORS", num_validators)?;
    export(file, "GENESIS_FORK_VERSION", "0x10000000")?;
    export(file, "ALTAIR_FORK_VERSION", "0x20000000")?;
    export(file, "BELLATRIX_FORK_VERSION", "0x30000000")?;
    export(file, "CAPELLA_FORK_VERSION", "0x40000000")?;
    export(
        file,
        "CAPELLA_FORK_EPOCH",
        genesis.capella_epoch.unwrap_or(0),
    )?;
    export(file, "DENEB_FORK_VERSION", "0x50000000")?;
    export(file, "DENEB_FORK_EPOCH", genesis.deneb_epoch.unwrap_or(0))?;
    export(file, "ELECTRA_FORK_VERSION", "0x60000000")?;
    export(
        file,
        "ELECTRA_FORK_EPOCH",
        genesis.electra_epoch.unwrap_or(9_999_999),
    )?;
    export(file, "FULU_FORK_VERSION", "0x70000000")?;
    export(
        file,
        "FULU_FORK_EPOCH",
        genesis.fulu_epoch.unwrap_or(9_999_999),
    )?;
    export(file, "EIP7594_FORK_VERSION", "0x80000000")?;
    export(
        file,
        "EIP7594_FORK_EPOCH",
        genesis.eip7594_epoch.unwrap_or(99_999_999),
    )?;
    export(file, "WITHDRAWAL_TYPE", "0x00")?;
    export(
        file,
        "WITHDRAWAL_ADDRESS",
        genesis
            .withdrawal_address
            .as_deref()
            .unwrap_or("0xf97e180c050e5Ab072211Ad2C213Eb5AEE4DF134"),
    )?;
    export(file, "GENESIS_TIMESTAMP", "946684800")?;
    export(file, "GENESIS_DELAY", genesis.delay.unwrap_or(300))?;
    export(
        file,
        "GENESIS_GASLIMIT",
        genesis.gaslimit.unwrap_or(25_000_000),
    )?;
    export_optional(
        file,
        "MAX_PER_EPOCH_ACTIVATION_CHURN_LIMIT",
        genesis.max_per_epoch_activation_churn_limit,
    )?;
    export_optional(file, "CHURN_LIMIT_QUOTIENT", genesis.churn_limit_quotient)?;
    export_optional(file, "EJECTION_BALANCE", genesis.ejection_balance)?;
    export_optional(file, "ETH1_FOLLOW_DISTANCE", genesis.eth1_follow_distance)?;
    export_optional(
        file,
        "MIN_VALIDATOR_WITHDRAWABILITY_DELAY",
        genesis.min_validator_withdrawability_delay,
    )?;
    export_optional(
        file,
        "SHARD_COMMITTEE_PERIOD",
        genesis.shard_committee_period,
    )?;
    export_optional(file, "SAMPLES_PER_SLOT", genesis.samples_per_slot)?;
    export_optional(file, "CUSTODY_REQUIREMENT", genesis.custody_requirement)?;
    export_optional(
        file,
        "DATA_COLUMN_SIDECAR_SUBNET_COUNT",
        genesis.data_column_sidecar_subnet_count,
    )?;
    export_optional(file, "MAX_BLOBS_PER_BLOCK", genesis.max_blobs_per_block)?;

    if let Some(premine) = genesis.premine.as_ref() {
        export(
            file,
            "EL_PREMINE_ADDRS",
            format!(
                "{{{}}}",
                premine
                    .iter()
                    .map(|(addr, amount)| format!("\\\"{addr}\\\": \\\"{amount}\\\""))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        )?;
    }
    //export(file, "ADDITIONAL_PRELOADED_CONTRACTS", )?;

    Ok(())
}

fn export<W: Write, V: Display>(file: &mut W, key: &str, value: V) -> std::io::Result<()> {
    writeln!(file, "export {key}=\"{value}\"")
}

fn export_optional<W: Write, V: Display>(
    file: &mut W,
    key: &str,
    value: Option<V>,
) -> std::io::Result<()> {
    if let Some(value) = value {
        export(file, key, value)
    } else {
        Ok(())
    }
}

pub fn generate(image_name: &str, output_path: &Path) -> Result<(), Error> {
    let mut data_mount = output_path.as_os_str().to_owned();
    data_mount.push(":/data");
    let mut config_mount = output_path.as_os_str().to_owned();
    config_mount.push("/values.env:/config/values.env");
    let status = log_and_wait(
        Command::new("docker")
            .args(["run", "--rm", "-i", "-u"])
            .arg(get_current_uid().to_string())
            .arg("-v")
            .arg(data_mount)
            .arg("-v")
            .arg(config_mount)
            .arg(image_name)
            .arg("all"),
    )?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::ChildProcessFailure(image_name.to_string()))
    }
}
