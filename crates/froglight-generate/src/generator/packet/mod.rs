use crate::{cli::CliArgs, datamap::DataMap};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketGenerator;

impl PacketGenerator {
    #[expect(clippy::unused_async)]
    pub async fn generate(_datamap: &DataMap, _args: &CliArgs) -> anyhow::Result<()> { Ok(()) }
}
