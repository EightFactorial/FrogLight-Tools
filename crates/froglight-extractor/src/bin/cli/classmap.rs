use async_zip::tokio::read::fs::ZipFileReader;
use cafebabe::ClassFile;
use froglight_data::VersionManifest;
use froglight_extractor::jar;
use hashbrown::HashMap;
use tracing::{debug, warn};

use crate::commands::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassMap {
    classes: HashMap<String, ParsedJar>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ParsedJar {
    data: Vec<u8>,
}

impl ParsedJar {
    pub(crate) fn parse(&self) -> ClassFile<'_> { cafebabe::parse_class(&self.data).unwrap() }
}

#[allow(dead_code)]
impl ClassMap {
    /// Create a new class map from the given command and manifest.
    pub(crate) async fn new(command: &Command, manifest: &VersionManifest) -> Self {
        let jar_path =
            jar::get_mapped_jar(&command.version, manifest, &command.cache, command.refresh).await;
        let jar = ZipFileReader::new(jar_path).await.expect("Failed to read jar file");

        // This creates a hashmap that's a bit too big, but it's better to overestimate
        // than underestimate
        let file_count = jar.file().entries().len();
        let mut classes = HashMap::with_capacity(file_count);

        for file_index in 0..file_count {
            let mut entry =
                jar.reader_with_entry(file_index).await.expect("Failed to read jar file");

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
            entry.read_to_end_checked(&mut data).await.expect("Failed to read class file");

            // Insert successfully parsed classes into the map
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

        debug!("Parsed {} classes", classes.len());

        Self { classes }
    }

    /// Create an empty class map.
    pub(crate) fn empty() -> Self { Self { classes: HashMap::new() } }

    /// Iterate over the classes in the map.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&String, &ParsedJar)> { self.classes.iter() }

    /// Iterate over the classes in the map.
    pub(crate) fn into_iter(self) -> impl Iterator<Item = (String, ParsedJar)> {
        self.classes.into_iter()
    }

    /// Get a class from the map.
    pub(crate) fn get(&self, key: &str) -> Option<&ParsedJar> { self.classes.get(key) }

    /// Insert a class into the map.
    pub(crate) fn insert(&mut self, key: String, value: ParsedJar) {
        self.classes.insert(key, value);
    }
}

impl std::fmt::Debug for ParsedJar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.parse().fmt(f) }
}
