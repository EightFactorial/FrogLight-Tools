use froglight_generate::{CliArgs, DataMap};

pub(crate) trait ModuleGenerator {
    async fn generate(datamap: &DataMap, args: &CliArgs) -> anyhow::Result<()>;
}
