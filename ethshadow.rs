#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! ethshadow = { path="lib" }
//! clap = { version = "4.5", features = ["cargo"] }
//! ```

use std::env;
use clap::{arg, command, value_parser};
use ethshadow::generate;
use std::error::Error;
use std::fs::File;
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut matches = command!() // requires `cargo` feature
        .bin_name("ethshadow")
        .arg(arg!(dir: -d [DIR] "Output directory for ethshadow and Shadow")
            .value_parser(value_parser!(PathBuf))
            .default_value("data"))
        .arg(arg!(genonly: --"gen-only" "Generate data dir only, do not invoke Shadow"))
        .arg(arg!(config: <CONFIG> "Configuration file. See CONFIG.md")
            .value_parser(value_parser!(PathBuf)))
        .arg(arg!(shadow_cli: [SHADOW_CLI_OPTION]... "Optional options passed on to Shadow, except \"-d\" and the config")
                .last(true),
        )
        .get_matches();

    let dir = matches
        .remove_one::<PathBuf>("dir")
        .expect("there is a default in place");
    let config = matches.get_one::<PathBuf>("config").expect("required arg");

    let config = File::open(config)?;

    let mut invocation = generate(config, dir)?;

    if !matches.get_flag("genonly") {
        if let Some(user_args) = matches.get_many::<String>("shadow_cli") {
            invocation.with_user_args(user_args);
        }
        // if exec() returns, the call failed!
        Err(invocation.command().exec().into())
    } else {
        Ok(())
    }
}
