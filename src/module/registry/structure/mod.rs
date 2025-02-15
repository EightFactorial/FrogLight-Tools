#![allow(dead_code)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use froglight_dependency::{
    container::{Dependency, DependencyContainer},
    dependency::minecraft::DataGenerator,
    version::Version,
};
use indexmap::{map::Entry, IndexMap};
use serde_json::Value;

#[derive(Clone, Default, Dependency)]
pub(crate) struct DataStructures(HashMap<Version, VersionStructures>);

impl DataStructures {
    pub(crate) fn version(&self, version: &Version) -> Option<&VersionStructures> {
        self.0.get(version)
    }

    pub(crate) async fn get_version(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&VersionStructures> {
        if !self.0.contains_key(version) {
            deps.get_or_retrieve::<DataGenerator>().await?;
            deps.scoped_fut::<DataGenerator, anyhow::Result<()>>(
                async |data: &mut DataGenerator, deps| {
                    let data = data.get_version(version, deps).await?;
                    self.0.insert(version.clone(), VersionStructures::parse(data).await?);
                    Ok(())
                },
            )
            .await?;
        }
        Ok(self.0.get(version).unwrap())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct VersionStructures(pub(crate) IndexMap<PathBuf, RegistryItem>);

impl VersionStructures {
    const BASE_IGNORE: &'static [&'static str] = &[
        "advancement",
        "datapacks",
        "enchantment",
        "loot_table",
        "recipe",
        "trial_spawner",
        "worldgen",
    ];

    async fn parse(path: &Path) -> anyhow::Result<Self> {
        let mut structure = Self::default();
        let path = path.join("data/minecraft");

        let mut dir = tokio::fs::read_dir(&path).await?;
        while let Ok(Some(dir)) = dir.next_entry().await {
            if dir.file_type().await.is_ok_and(|f| f.is_dir()) {
                // Skip ignored base directories
                if Self::BASE_IGNORE.iter().any(|ignored| dir.path().ends_with(ignored)) {
                    continue;
                }

                structure.parse_recursive(&dir.path(), &path).await?;
            }
        }

        Ok(structure)
    }

    async fn parse_recursive(&mut self, path: &Path, base: &Path) -> anyhow::Result<()> {
        let mut item = RegistryItem::default();

        let mut dir = tokio::fs::read_dir(path).await?;
        while let Ok(Some(dir)) = dir.next_entry().await {
            if dir.file_type().await.is_ok_and(|f| f.is_dir()) {
                Box::pin(self.parse_recursive(&dir.path(), base)).await?;
                continue;
            }

            tracing::trace!("Parsing file: \"{}\"", dir.path().display());

            let content = tokio::fs::read_to_string(dir.path()).await?;
            item.extend(RegistryItem::from_value(&serde_json::from_str(&content)?));
        }

        if !item.0.is_empty() {
            self.0.insert(path.strip_prefix(base).unwrap_or(path).to_path_buf(), item);
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct RegistryItem(pub(crate) IndexMap<String, RegistryField>);

impl RegistryItem {
    fn from_value(value: &Value) -> Self {
        let mut fields = IndexMap::new();

        if let Some(map) = value.as_object() {
            for (field_name, field_value) in map {
                fields.insert(field_name.clone(), RegistryField::from_value(field_value));
            }
        } else {
            fields.insert(String::new(), RegistryField::from_value(value));
        }

        Self(fields)
    }

    fn extend(&mut self, other: Self) {
        for (key, value) in other.0 {
            match self.0.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().extend(value);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RegistryField {
    Bool,
    Float,
    Integer,
    Null,
    String,

    Enum(Vec<RegistryField>),
    Item(RegistryItem),
    Vec(Box<RegistryField>),
}

impl RegistryField {
    fn from_value(value: &Value) -> Self {
        match value {
            Value::Null => panic!("Null value in RegistryField!"),
            Value::Bool(_) => RegistryField::Bool,
            Value::Number(n) if n.is_f64() => RegistryField::Float,
            Value::Number(_) => RegistryField::Integer,
            Value::String(_) => RegistryField::String,
            Value::Array(values) => {
                let mut field: Option<Self> = None;
                for value in values {
                    if let Some(item) = field.as_mut() {
                        item.extend(Self::from_value(value));
                    } else {
                        field = Some(Self::from_value(value));
                    }
                }

                Self::Vec(Box::new(field.unwrap_or(RegistryField::Null)))
            }
            map @ Value::Object(..) => RegistryField::Item(RegistryItem::from_value(map)),
        }
    }

    fn extend(&mut self, other: Self) {
        match (self, other) {
            // Replace Null
            (a @ RegistryField::Null, b) => {
                *a = b;
            }

            // Extend Enums
            (a @ RegistryField::Enum(..), RegistryField::Enum(b)) => {
                b.into_iter().for_each(|b| a.extend(b));
            }
            (RegistryField::Enum(a), b) => {
                if !a.contains(&b) {
                    a.push(b);
                }
            }
            (a, mut b @ RegistryField::Enum(..)) => {
                std::mem::swap(a, &mut b);
                a.extend(b);
            }

            // Extend Items/Vecs
            (RegistryField::Item(a), RegistryField::Item(b)) => {
                a.extend(b);
            }
            (RegistryField::Vec(a), RegistryField::Vec(b)) => {
                a.as_mut().extend(*b);
            }

            // Ignore matching types and Null
            (RegistryField::Bool, RegistryField::Bool)
            | (RegistryField::Float, RegistryField::Float)
            | (RegistryField::Integer, RegistryField::Integer)
            | (_, RegistryField::Null)
            | (RegistryField::String, RegistryField::String) => {}

            // Create Enums
            (a, b) => {
                let old_a = std::mem::replace(a, RegistryField::Enum(Vec::new()));
                a.extend(old_a);
                a.extend(b);
            }
        }
    }
}
