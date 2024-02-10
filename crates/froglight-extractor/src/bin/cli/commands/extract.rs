use async_zip::tokio::read::fs::ZipFileReader;
use cafebabe::ClassFile;
use froglight_data::VersionManifest;
use froglight_extractor::jar;
use hashbrown::HashMap;
use tracing::{debug, warn};

use super::Command;

pub(crate) async fn extract(command: &Command, manifest: &VersionManifest) -> serde_json::Value {
    let _classmap = ClassMap::new(command, manifest).await;

    todo!()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassMap {
    classes: HashMap<String, ParsedJar>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ParsedJar {
    data: Vec<u8>,
}

impl ParsedJar {
    fn jar(&self) -> ClassFile<'_> { cafebabe::parse_class(&self.data).unwrap() }
}

impl ClassMap {
    async fn new(command: &Command, manifest: &VersionManifest) -> Self {
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
}

impl std::fmt::Debug for ParsedJar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.jar().fmt(f) }
}
