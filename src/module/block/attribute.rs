#![allow(dead_code)]

use std::{collections::HashMap, ops::Range, path::Path};

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockAttributeBundle(pub(crate) Vec<BlockAttributes>);

impl BlockAttributeBundle {
    pub(crate) async fn parse(path: &Path) -> anyhow::Result<Self> {
        let path = path.join("reports/blocks.json");
        tracing::debug!("Parsing `blocks.json` from \"{}\"", path.display());

        let contents = tokio::fs::read_to_string(path).await?;
        let parsed = serde_json::from_str::<ParsedBlockReport>(&contents)?;

        Ok(Self::from_parsed(parsed))
    }

    fn from_parsed(parsed: ParsedBlockReport) -> Self {
        let mut bundle = Vec::new();

        let mut next_state = Some(0);
        while let Some(next) = next_state.as_mut() {
            if let Some((name, state)) = Self::get_next(*next, &parsed) {
                bundle.push(BlockAttributes {
                    name: name.to_string(),
                    blockstate_ids: state.range(),
                    default_state: state.default() - state.range().start,
                    attributes: BlockAttributes::attributes(state),
                });
                *next = state.range().end + 1;
            } else {
                next_state = None;
            }
        }

        tracing::debug!("PARSED: {bundle:#?}");
        Self(bundle)
    }

    fn get_next(index: u32, parsed: &ParsedBlockReport) -> Option<(&String, &ParsedBlockEntry)> {
        parsed.0.iter().find(|(_, entry)| entry.states.iter().any(|state| state.id == index))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockAttributes {
    pub name: String,
    pub blockstate_ids: Range<u32>,
    pub default_state: u32,
    pub attributes: Vec<BlockAttribute>,
}

impl BlockAttributes {
    fn attributes(parsed: &ParsedBlockEntry) -> Vec<BlockAttribute> {
        parsed
            .properties
            .iter()
            .map(|(name, values)| BlockAttribute { name: name.to_string(), values: values.clone() })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BlockAttribute {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct ParsedBlockReport(HashMap<String, ParsedBlockEntry>);

#[derive(Deserialize)]
struct ParsedBlockEntry {
    #[serde(default)]
    properties: IndexMap<String, Vec<String>>,
    states: Vec<ParsedBlockState>,
}

impl ParsedBlockEntry {
    pub(crate) fn default(&self) -> u32 {
        self.states.iter().find(|state| state.default).map_or_else(
            || panic!("BlockAttributeBundle: Unable to find default state"),
            |state| state.id,
        )
    }

    pub(crate) fn range(&self) -> Range<u32> {
        Range {
            start: self.states.iter().map(|state| state.id).min().unwrap(),
            end: self.states.iter().map(|state| state.id).max().unwrap(),
        }
    }
}

#[derive(Deserialize)]
struct ParsedBlockState {
    id: u32,
    #[serde(default)]
    default: bool,
}
