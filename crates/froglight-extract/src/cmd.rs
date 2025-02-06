//! TODO

use std::{collections::HashMap, path::PathBuf, sync::Once};

use clap::Parser;
use froglight_dependency::{container::SharedDependencies, version::Version};
use tokio::runtime::Builder;

use crate::module::{ExtractModule, JsonModule, JsonOutput};

/// The default pre-configured entry point.
///
/// If a `JsonOutput` is present it will be serialized to the console or a file.
///
/// # Example
/// ```rust
/// /// Import the function in `main.rs`.
/// pub use froglight_extract::main;
/// ```
#[expect(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn main() -> anyhow::Result<()> {
    #[cfg(feature = "logging")]
    logging();

    // Parse the command line arguments.
    let mut args = ExtractArgs::parse();

    // Build the runtime and run the `extract` function.
    let runtime =
        Builder::new_multi_thread().enable_all().build().expect("Failed building the Runtime");

    runtime.block_on(async move {
        // If no modules are provided, default to running the `JsonModule`.
        if args.modules.is_empty() {
            args.modules.push(JsonModule::MODULE_NAME.to_string());
        }

        let dependencies = SharedDependencies::from_rust_env();
        extract(args.version, &args.modules, dependencies.clone()).await?;

        // If the `JsonOutput` is present, serialize it to the console or a file.
        if let Some(output) = dependencies.write().await.take::<JsonOutput>() {
            let json = serde_json::to_string_pretty(&output.0)?;
            if let Some(path) = args.output {
                // Write the JSON to the output file.
                tokio::fs::write(path, json).await?;
            } else {
                // Print the JSON to the console.
                println!("{json}");
            }
        }

        Ok(())
    })
}

/// The [`froglight_extract::main`](main) command line arguments.
#[derive(Parser)]
pub struct ExtractArgs {
    /// The version to extract.
    #[clap(short, long)]
    pub version: Version,

    /// The extract modules to run.
    #[clap(short, long)]
    pub modules: Vec<String>,

    /// The path to the output file.
    ///
    /// If `None`, the result will be logged to the console.
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}

/// Initialize logging with the default environment filter.
///
/// This function will only ever run once, even if called multiple times.
///
/// See [`EnvFilter::from_default_env`](tracing_subscriber::EnvFilter::from_default_env)
/// for more information.
#[cfg(feature = "logging")]
pub fn logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    static LOGGING: Once = Once::new();
    LOGGING.call_once(|| {
        let filter = EnvFilter::from_default_env();
        fmt().with_env_filter(filter).with_writer(std::io::stderr).init();
    });
}

/// The extract function.
///
/// Useful if you want to use the extracted data in your own code.
///
/// ```rust,ignore
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let version = Version::new_release(1, 21, 4);
///     let data = froglight_extract::extract(version, ..).await?;
///     // etc...
/// }
/// ```
#[allow(clippy::missing_errors_doc, clippy::unused_async)]
pub async fn extract(
    version: Version,
    modules: &[String],
    dependencies: SharedDependencies,
) -> anyhow::Result<()> {
    // Collect the `ExtractModule`s into a map.
    let module_map: HashMap<&str, &ExtractModule> =
        ExtractModule::collect().map(|m| (m.name(), m)).collect();

    // Iterate over the specified modules and run them.
    for module in modules {
        if let Some(extract) = module_map.get(module.as_str()) {
            tracing::info!("Running module \"{module}\"");
            extract.run(&version, &mut *dependencies.write().await)?;
        } else {
            tracing::error!("Unknown module \"{module}\"");
        }
    }

    Ok(())
}
