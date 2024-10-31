//! Extraction Sources

use std::{future::Future, pin::Pin};

use enum_dispatch::enum_dispatch;
use froglight_extract::{bundle::ExtractBundle, sources::Modules as ExtractModules};
use serde::{Deserialize, Serialize};

mod packets;
use packets::Packets;

use crate::bundle::GenerateBundle;

/// Modules that generate code from extracted data
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[enum_dispatch(GenerateModule)]
#[serde(untagged)]
pub enum Modules {
    /// Generate packets
    Packets(Packets),
}

impl Modules {
    /// Default modules to use when none are specified.
    pub const DEFAULT: &'static [Modules] = &[Modules::Packets(Packets)];

    /// Get the required [`ExtractModules`] for this module.
    #[must_use]
    pub fn required(&self) -> &'static [ExtractModules] {
        match self {
            Modules::Packets(_) => <Packets as sealed::GenerateRequired>::REQUIRED,
        }
    }
}

/// Trait for extracting data from a source.
#[enum_dispatch]
pub trait GenerateModule: sealed::GenerateRequired {
    /// Run the generation process.
    fn generate<'a>(
        &'a self,
        generate: &'a GenerateBundle<'_>,
        extract: &'a ExtractBundle,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + Sync + 'a>>;
}

/// Sealed trait to allow an array of required extract modules to be defined.
mod sealed {
    use froglight_extract::sources::Modules;

    pub trait GenerateRequired {
        const REQUIRED: &'static [Modules];
    }

    impl GenerateRequired for super::Modules {
        const REQUIRED: &'static [Modules] = &[];
    }
}

/// Implement `FromStr` for `Modules` to allow parsing from a string.
mod json_workaround {
    use std::str::FromStr;

    use serde::{Deserialize, Serialize};

    use super::Modules;

    #[derive(Serialize, Deserialize)]
    struct ModuleWrapper {
        module: Modules,
    }

    impl FromStr for Modules {
        type Err = anyhow::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match serde_json::from_str::<ModuleWrapper>(&format!("{{\"module\": \"{s}\"}}")) {
                Ok(resolver) => Ok(resolver.module),
                Err(err) => Err(anyhow::anyhow!("Failed to parse module: {err}")),
            }
        }
    }
}
