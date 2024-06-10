//! Bytecode parsing utilities.

use std::{path::Path, sync::Arc};

use cafebabe::ClassFile;
use hashbrown::HashMap;
use tracing::error;

/// A container for a JAR file.
///
/// Contains a map of paths to class files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JarContainer(HashMap<String, ClassContainer>);

impl JarContainer {
    /// Create a new [`JarContainer`] from a ZIP file reader.
    ///
    /// # Errors
    /// Errors if the ZIP file cannot be read.
    #[allow(clippy::missing_panics_doc)]
    pub async fn new_tokio_fs(
        zip: &async_zip::tokio::read::fs::ZipFileReader,
    ) -> anyhow::Result<Self> {
        // Iterate over all the entries in the ZIP file
        let mut classes = HashMap::new();
        for index in 0..zip.file().entries().len() {
            // Get each class file inside the ZIP file
            let mut entry = zip.reader_with_entry(index).await?;
            if entry.entry().filename().as_str().is_ok_and(|s| {
                Path::new(s).extension().map_or(false, |ext| ext.eq_ignore_ascii_case("class"))
            }) {
                // Prepare a buffer to read the class file into
                let buffer_size =
                    usize::try_from(entry.entry().uncompressed_size()).expect("Class too large");
                let mut buffer = Vec::with_capacity(buffer_size);

                // Read the class file into the buffer
                entry.read_to_end_checked(&mut buffer).await?;
                let buffer = Arc::from(buffer);

                // Verify that the class file can be parsed
                let class_name = entry.entry().filename().clone().into_string().ok();
                match cafebabe::parse_class(&buffer) {
                    Ok(_) => {
                        //  Insert the class file into the map
                        if let Some(class_name) = class_name {
                            classes.insert(
                                class_name.trim_end_matches(".class").to_string(),
                                ClassContainer(buffer),
                            );
                        }
                    }
                    Err(err) => {
                        if let Some(class_name) = class_name {
                            error!("Failed to parse class file \"{class_name}\": {err}");
                        } else {
                            error!("Failed to parse class file: {err}");
                        }
                    }
                }
            }
        }

        Ok(Self(classes))
    }
}

impl std::ops::Deref for JarContainer {
    type Target = HashMap<String, ClassContainer>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for JarContainer {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

/// A container for a class file.
///
/// Only contains the raw bytes of the class file,
/// which can be parsed using [`ClassContainer::parse`].
///
/// Can freely be cloned.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassContainer(Arc<[u8]>);

impl ClassContainer {
    /// Parse the class file.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn parse(&self) -> ClassFile<'_> { cafebabe::parse_class(&self.0).unwrap() }
}
