use std::collections::VecDeque;
use std::fs::File;
use std::io::{Stdin, StdinLock};
use std::net::TcpStream;
use std::process::{ChildStderr, ChildStdout};
use std::sync::Arc;

use serde_yaml::{Mapping, Value};

pub use ethshadow::EthShadowConfig;
pub use shadow::ShadowConfig;

use crate::error::Error;

pub mod ethshadow;
pub mod one_or_many;
pub mod shadow;

pub struct FullConfig {
    pub ethshadow_config: EthShadowConfig,
    pub shadow_config: ShadowConfig,
}

impl TryFrom<Mapping> for FullConfig {
    type Error = Error;

    fn try_from(mut mapping: Mapping) -> Result<Self, Self::Error> {
        let ethshadow_config: EthShadowConfig = mapping
            .remove("ethereum")
            .map(serde_yaml::from_value)
            .transpose()?
            .unwrap_or_default();
        let shadow_config = ShadowConfig(mapping);
        Ok(FullConfig {
            ethshadow_config,
            shadow_config,
        })
    }
}

impl TryFrom<Value> for FullConfig {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let Value::Mapping(mapping) = value else {
            return Err(Error::ExpectedOtherType("<root>".to_string()));
        };
        mapping.try_into()
    }
}

macro_rules! from_readers {
    ($id:ty $(,$tail:ty)* $(,)?) => {
        impl TryFrom<$id> for FullConfig {
            type Error = Error;

            fn try_from(reader: $id) -> Result<Self, Self::Error> {
                serde_yaml::from_reader::<_, Value>(reader)?.try_into()
            }
        }
        impl TryFrom<&mut $id> for FullConfig {
            type Error = Error;

            fn try_from(reader: &mut $id) -> Result<Self, Self::Error> {
                serde_yaml::from_reader::<_, Value>(reader)?.try_into()
            }
        }
        from_readers!($($tail),*);
    };
    () => {};
}
from_readers!(
    File,
    TcpStream,
    ChildStderr,
    ChildStdout,
    Arc<File>,
    Stdin,
    StdinLock<'_>,
    VecDeque<u8>,
);

impl TryFrom<&str> for FullConfig {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        serde_yaml::from_str::<Value>(s)?.try_into()
    }
}

impl TryFrom<&[u8]> for FullConfig {
    type Error = Error;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        serde_yaml::from_slice::<Value>(s)?.try_into()
    }
}
