use crate::clients::Client;
use crate::clients::CommonParams;
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::ValidatorSet;
use crate::Error;
use serde::{Deserialize, Serialize};
use serde_yaml::to_writer;
use std::collections::HashMap;
use std::fs::File;

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Prometheus {
    #[serde(flatten)]
    common: CommonParams,
}

#[derive(Serialize)]
struct PrometheusYaml<'a> {
    scrape_configs: Vec<ScrapeConfig<'a>>,
}

#[derive(Serialize)]
struct ScrapeConfig<'a> {
    job_name: String,
    scrape_interval: String,
    static_configs: Vec<StaticConfig<'a>>,
}

#[derive(Serialize)]
struct StaticConfig<'a> {
    targets: &'a [String],
    labels: HashMap<&'a str, &'a str>,
}

#[typetag::deserialize(name = "prometheus")]
impl Client for Prometheus {
    fn add_to_node(
        &self,
        node: &NodeInfo,
        ctx: &mut SimulationContext,
        _vs: &ValidatorSet,
    ) -> Result<Process, Error> {
        let dir = node.dir().join("prometheus");
        let config_file = node.dir().join("prometheus.yaml");

        let config = PrometheusYaml {
            scrape_configs: vec![ScrapeConfig {
                job_name: "lighthouses".to_string(),
                scrape_interval: "15s".to_string(),
                static_configs: ctx
                    .cl_monitoring_endpoints()
                    .iter()
                    .map(|((location, reliability), targets)| StaticConfig {
                        targets,
                        labels: [("location", *location), ("reliability", *reliability)]
                            .into_iter()
                            .collect(),
                    })
                    .collect(),
            }],
        };

        to_writer(File::create_new(&config_file)?, &config)?;

        Ok(Process {
            path: self.common.executable_or("prometheus"),
            args: format!(
                "--storage.tsdb.path={} --config.file={} {}",
                dir.to_str().ok_or(Error::NonUTF8Path)?,
                config_file.to_str().ok_or(Error::NonUTF8Path)?,
                self.common.arguments(""),
            ),
            environment: HashMap::default(),
            expected_final_state: "running".into(),
            start_time: "10s".into(),
        })
    }
}
