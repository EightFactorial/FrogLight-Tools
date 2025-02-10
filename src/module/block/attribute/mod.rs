#![allow(clippy::module_inception, dead_code)]

use std::{collections::HashSet, ops::RangeInclusive, sync::Arc};

use attribute::BlockAttributeAttribute;
use convert_case::{Case, Casing};
use froglight_dependency::{
    container::{Dependency, DependencyContainer},
    version::Version,
};
use froglight_extract::module::ExtractModule;

mod attribute;
pub(crate) use attribute::BlockAttributeData;

mod report;
pub(crate) use report::BlockReports;

use crate::ToolConfig;

#[derive(Clone, PartialEq, Eq, Dependency)]
#[dep(retrieve = BlockAttributes::generate)]
pub(crate) struct BlockAttributes(pub Arc<HashSet<BlockAttributeAttribute>>);

impl BlockAttributes {
    // TODO: Create an enum representation that can get retrieved from
    // `BlockAttributes` with properly formatted values.

    /// Iterate over all versions and add all unique attributes to the set.
    async fn generate(deps: &mut DependencyContainer) -> anyhow::Result<Self> {
        let mut attributes = HashSet::new();

        deps.get_or_retrieve::<BlockReports>().await?;
        deps.scoped_fut::<BlockReports, anyhow::Result<()>>(
            async |reports: &mut BlockReports, deps| {
                for version in deps.get::<ToolConfig>().unwrap().versions.clone() {
                    for (name, entry) in &reports.get_version(&version, deps).await?.0 {
                        attributes.extend(BlockAttributeData::from_parsed(name, entry).attributes);
                    }
                }
                Ok(())
            },
        )
        .await?;

        Ok(Self(Arc::new(attributes)))
    }

    pub(crate) fn as_enum_name(&self, attr: &BlockAttributeAttribute) -> String {
        self.as_enum_name_internal(attr, &EnumType::from_attribute(attr))
    }

    fn as_enum_name_internal(
        &self,
        attr: &BlockAttributeAttribute,
        enum_type: &EnumType,
    ) -> String {
        match enum_type {
            EnumType::Bool => format!("{}Bool", attr.name.to_case(Case::Pascal)),
            EnumType::IntRange(range) => {
                format!("{}Int{}To{}", attr.name.to_case(Case::Pascal), range.start(), range.end())
            }
            EnumType::Enum(values) => {
                if self.0.iter().filter(|a| a.name == attr.name).count() == 1 {
                    format!("{}Enum", attr.name.to_case(Case::Pascal))
                } else {
                    format!(
                        "{}Enum_{}",
                        attr.name.to_case(Case::Pascal),
                        values
                            .iter()
                            .map(|v| v.to_case(Case::Pascal))
                            .collect::<Vec<_>>()
                            .join("_"),
                    )
                }
            }
        }
    }

    pub(crate) fn as_enum(&self, attr: &BlockAttributeAttribute) -> String {
        let enum_type = EnumType::from_attribute(attr);
        let ident = self.as_enum_name_internal(attr, &enum_type);

        match enum_type {
            EnumType::Bool => format!("pub enum {ident} {{ True, False }}"),
            EnumType::IntRange(range) => {
                format!(
                    "pub enum {ident} {{ {} }}",
                    range.into_iter().map(|i| format!("_{i}")).collect::<Vec<_>>().join(", ")
                )
            }
            EnumType::Enum(values) => {
                format!(
                    "pub enum {ident} {{ {} }}",
                    values.iter().map(|v| v.to_case(Case::Pascal)).collect::<Vec<_>>().join(", ")
                )
            }
        }
    }
}

enum EnumType<'a> {
    Bool,
    IntRange(RangeInclusive<u8>),
    Enum(&'a [String]),
}

impl<'a> EnumType<'a> {
    fn from_attribute(attr: &'a BlockAttributeAttribute) -> Self {
        if attr.values.len() == 2 && attr.values[0..2] == ["true", "false"] {
            Self::Bool
        } else if let Ok(integers) =
            attr.values.iter().map(|v| v.parse::<u8>()).collect::<Result<Vec<_>, _>>()
        {
            Self::IntRange(*integers.iter().min().unwrap()..=*integers.iter().max().unwrap())
        } else {
            Self::Enum(attr.values.as_slice())
        }
    }
}
