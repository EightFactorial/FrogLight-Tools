use std::ops::RangeInclusive;

use super::report::ParsedBlockEntry;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BlockAttributeData {
    pub name: String,
    pub blockstate_ids: RangeInclusive<u32>,
    pub default_state: u32,
    pub attributes: Vec<BlockAttributeAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct BlockAttributeAttribute {
    pub name: String,
    pub values: Vec<String>,
}

impl BlockAttributeData {
    pub(crate) fn from_parsed(name: &str, entry: &ParsedBlockEntry) -> Self {
        Self {
            name: name.to_string(),
            blockstate_ids: entry.range(),
            default_state: entry.default(),
            attributes: entry
                .properties
                .iter()
                .map(|(name, values)| BlockAttributeAttribute {
                    name: name.to_string(),
                    values: values.clone(),
                })
                .collect(),
        }
    }
}

impl PartialOrd for BlockAttributeData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}
impl Ord for BlockAttributeData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.blockstate_ids.start().cmp(other.blockstate_ids.start())
    }
}
