//! A map of classes to their parsed representations.

use std::path::Path;

use async_zip::tokio::read::fs::ZipFileReader;
use cafebabe::ClassFile;
use froglight_data::{Version, VersionManifest};
use hashbrown::HashMap;
use tracing::{trace, warn};

use crate::jar;

/// A map of classes to their parsed representations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassMap {
    classes: HashMap<String, ParsedJar>,
}

/// A parsed jar file.
///
/// This is a wrapper around the raw data of a jar file
/// that can be parsed into a [`ClassFile`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ParsedJar {
    data: Vec<u8>,
}

impl ParsedJar {
    /// Parse the jar file into a [`ClassFile`].
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn parse(&self) -> ClassFile<'_> { cafebabe::parse_class(&self.data).unwrap() }

    /// Parse the jar file into a mutable [`ClassFile`].
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn parse_mut(&mut self) -> ClassFile<'_> { cafebabe::parse_class(&self.data).unwrap() }
}

#[allow(dead_code)]
impl ClassMap {
    /// Create a new class map from the given command and manifest.
    ///
    /// # Errors
    /// - If files cannot be read or downloaded
    pub async fn new(
        version: &Version,
        manifest: &VersionManifest,
        cache: &Path,
        refresh: bool,
    ) -> anyhow::Result<Self> {
        let jar_path = jar::get_mapped_jar(version, manifest, cache, refresh).await?;
        let jar = ZipFileReader::new(jar_path).await?;

        // It's a bit large, but no versions should have more than 10,000 classes
        let mut classes = HashMap::with_capacity(10_000);

        for file_index in 0..jar.file().entries().len() {
            let mut entry = jar.reader_with_entry(file_index).await?;

            // Skip non-class files
            let Ok(filename) = entry.entry().filename().as_str() else {
                continue;
            };
            if !std::path::Path::new(filename)
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("class"))
            {
                continue;
            }

            // Parse the class file
            let mut data = Vec::new();
            entry.read_to_end_checked(&mut data).await?;

            // Insert successfully parsable classes into the map
            //
            // Because parsed classes don't own their data, we can just insert the data
            // directly into the map and then parse it again when we need it
            match cafebabe::parse_class(&data) {
                Err(err) => warn!("Failed to parse class file: `{err}`"),
                Ok(class) => {
                    classes.insert(class.this_class.to_string(), ParsedJar { data });
                }
            }
        }

        trace!("Parsed {} classes", classes.len());

        Ok(Self { classes })
    }

    /// Create an empty class map.
    #[must_use]
    pub fn empty() -> Self { Self { classes: HashMap::new() } }

    /// Iterate over the classes in the map.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ParsedJar)> { self.classes.iter() }

    /// Iterate over the classes in the map.
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> impl Iterator<Item = (String, ParsedJar)> { self.classes.into_iter() }

    /// Get a class from the map.
    #[must_use]
    pub fn get<'a>(&'a self, key: &str) -> Option<ClassFile<'a>> {
        self.classes.get(key).map(ParsedJar::parse)
    }

    /// Get a mutable class from the map.
    #[must_use]
    pub fn get_mut<'a>(&'a mut self, key: &str) -> Option<ClassFile<'a>> {
        self.classes.get_mut(key).map(ParsedJar::parse_mut)
    }

    /// Insert a class into the map.
    pub fn insert(&mut self, key: String, value: ParsedJar) { self.classes.insert(key, value); }

    /// Get the number of classes in the map.
    #[must_use]
    pub fn len(&self) -> usize { self.classes.len() }

    /// Check if the map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool { self.classes.is_empty() }
}

impl std::fmt::Debug for ParsedJar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.parse().fmt(f) }
}
