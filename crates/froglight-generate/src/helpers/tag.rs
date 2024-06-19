#![allow(dead_code)]

use std::path::Path;

use crate::consts::GENERATE_NOTICE;

/// Update the `@generated` tag at the top of a file.
///
/// If there is no tag, one will be added.
pub(crate) async fn update_tag(path: &Path) -> anyhow::Result<()> {
    // Read the contents of the file.
    let contents = tokio::fs::read_to_string(path).await?;

    // Write the updated contents back to the file.
    tokio::fs::write(
        path,
        if let Some(index) = contents.find("//! @generated") {
            // If the @generated tag is found, update it.
            update_existing_tag(contents, index)
        } else {
            // If the @generated tag is not found, insert it.
            insert_new_tag(contents)
        },
    )
    .await?;

    Ok(())
}

/// Update an existing `@generated` tag with the current git hash.
fn update_existing_tag(mut contents: String, index: usize) -> String {
    // Find the end of the line where the `@generated` tag is.
    let end = contents[index..].find('\n').unwrap_or(contents[index..].len());
    // Replace the existing line with the `@generated` tag with the new one.
    contents.replace_range(index..index + end, GENERATE_NOTICE);
    contents
}

/// Insert a new `@generated` doc comment with the current git hash.
fn insert_new_tag(mut contents: String) -> String {
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
fn test_update_existing_tag() {
    let test_content =
        "//! Updating Existing `@generated` Tag\n//!\n//! @generated `froglight-generate` #0000000";
    let index = test_content.find("//! @generated").unwrap();
    let updated = update_existing_tag(test_content.to_string(), index);
    assert!(updated.ends_with(GENERATE_NOTICE));
}

#[test]
fn test_insert_new_tag() {
    let test_content = "//! Inserting New `@generated` Tag";
    let updated = insert_new_tag(test_content.to_string());
    assert!(updated.ends_with(GENERATE_NOTICE));
}
