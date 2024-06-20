use std::{io::SeekFrom, path::Path};

use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use tracing::trace;

use super::format_file_contents;
use crate::consts::GENERATE_NOTICE;

/// Update the `@generated` tag at the top of a file.
///
/// If there is no tag, one will be added.
pub(crate) async fn update_tag(path: &Path) -> anyhow::Result<()> {
    update_file_tag(&mut OpenOptions::new().read(true).write(true).open(path).await?, path).await
}

/// Update the `@generated` tag at the top of a file.
///
/// If there is no tag, one will be added.
///
/// Requires the file to be opened with both
/// [`read`](OpenOptions::read) and [`write`](OpenOptions::write)
/// permissions.
pub(crate) async fn update_file_tag(file: &mut File, path: &Path) -> anyhow::Result<()> {
    trace!("Updating `@generated` tag for: \"{}\"", path.display());

    // Read the contents of the file.
    let mut contents = String::new();
    file.seek(SeekFrom::Start(0u64)).await?;
    file.read_to_string(&mut contents).await?;

    // Update the tag in the contents.
    let updated = tag_file_contents(contents);
    let formatted = format_file_contents(updated).await?;

    // Write the updated contents back to the file.
    file.seek(SeekFrom::Start(0u64)).await?;
    file.set_len(0).await?;
    file.write_all(formatted.as_bytes()).await?;
    file.sync_data().await.map_err(Into::into)
}

/// Update the `@generated` tag at the top of a file.
///
/// If there is no tag, one will be added.
pub(crate) fn tag_file_contents(contents: String) -> String {
    if let Some(index) = contents.find("//! @generated") {
        // If the @generated tag is found, update it.
        existing_tag(contents, index)
    } else {
        // If the @generated tag is not found, insert it.
        new_tag(contents)
    }
}

/// Update an existing `@generated` tag with the current git hash.
fn existing_tag(mut contents: String, index: usize) -> String {
    // Find the end of the line where the `@generated` tag is.
    let end = contents[index..].find('\n').unwrap_or(contents[index..].len());
    // Replace the existing line with the `@generated` tag with the new one.
    contents.replace_range(index..index + end, GENERATE_NOTICE);
    contents
}

/// Insert a new `@generated` doc comment with the current git hash.
fn new_tag(mut contents: String) -> String {
    // Find the end of the last `//!` comment in the file.
    let mut end = 0usize;
    while let Some(index) = contents[end..].find("//!") {
        end += index.max(1);
    }

    // If there are no comments in the file, add the tag at the top.
    if end == 0 {
        return format!("{GENERATE_NOTICE}\n\n{contents}");
    }

    // Find the end of the last comment.
    end = contents[end - 1..].find('\n').unwrap_or(contents.len());

    // Create a new string to append to the end of the existing comments.
    let mut append = String::new();
    append.push_str("\n//!\n");
    append.push_str(GENERATE_NOTICE);

    // Insert the new `@generated` tag
    contents.replace_range(end..end, &append);
    contents
}

#[test]
fn test_existing_tag() {
    let test_content =
        "//! Updating Existing `@generated` Tag\n//!\n//! @generated `froglight-generate` #0000000";
    let index = test_content.find("//! @generated").unwrap();
    let updated = existing_tag(test_content.to_string(), index);
    assert!(updated.ends_with(GENERATE_NOTICE));
}

#[test]
fn test_new_tag() {
    let test_content = "//! Inserting New `@generated` Tag";
    let updated = new_tag(test_content.to_string());
    assert!(updated.ends_with(GENERATE_NOTICE));
}

#[test]
fn test_untagged() {
    let test_content = "";
    let updated = new_tag(test_content.to_string());
    assert!(updated[..updated.len() - 2].ends_with(GENERATE_NOTICE));
}
