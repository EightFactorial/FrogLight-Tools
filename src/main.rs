#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use froglight_dependency::{
    container::{DependencyContainer, SharedDependencies},
    dependency::{minecraft::DataGenerator, vineflower::DecompiledJar},
    version::Version,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    froglight_extract::logging();

    let version = Version::new_snapshot(25, 6, 'a').unwrap();
    let dependencies = SharedDependencies::from_rust_env();

    {
        let mut deps = dependencies.write().await;

        deps.get_or_retrieve::<DecompiledJar>().await?;
        deps.scoped_fut::<DecompiledJar, anyhow::Result<()>>(
            async |jar: &mut DecompiledJar, deps: &mut DependencyContainer| {
                jar.get_client(&version, deps).await?;
                Ok(())
            },
        )
        .await?;

        deps.get_or_retrieve::<DataGenerator>().await?;
        deps.scoped_fut::<DataGenerator, anyhow::Result<()>>(
            async |gen: &mut DataGenerator, deps: &mut DependencyContainer| {
                gen.get_version(&version, deps).await?;
                Ok(())
            },
        )
        .await?;
    }

    Ok(())
}
