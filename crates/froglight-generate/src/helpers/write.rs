use std::{io::SeekFrom, path::Path, process::Stdio};

use quote::ToTokens;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    process::Command,
    task::JoinHandle,
};

use super::tag_file_contents;

/// Write tokens to a file.
///
/// This will overwrite the file if it already exists.
pub(crate) async fn write_tokens_to_file(tokens: impl ToTokens, path: &Path) -> anyhow::Result<()> {
    write_to_file(tokens_to_string(tokens)?, path).await
}

/// Append tokens to a file.
pub(crate) async fn append_tokens_to_file(
    tokens: impl ToTokens,
    file: &mut File,
) -> anyhow::Result<()> {
    append_to_file(tokens_to_string(tokens)?, file).await
}

/// Convert tokens to a string.
///
/// Formats tokens with `prettyplease`
pub(crate) fn tokens_to_string(tokens: impl ToTokens) -> anyhow::Result<String> {
    let parsed = syn::parse2(tokens.into_token_stream())?;
    Ok(prettyplease::unparse(&parsed))
}

/// Write code to a file.
///
/// This will overwrite the file if it already exists.
pub(crate) async fn write_to_file(output: String, path: &Path) -> anyhow::Result<()> {
    let mut file = File::create(path).await?;
    let tagged = tag_file_contents(output);
    append_to_file(tagged, &mut file).await
}

/// Append code to a file.
///
/// Formats output with `rustfmt` before writing.
pub(crate) async fn append_to_file(output: String, file: &mut File) -> anyhow::Result<()> {
    let formatted = format_file_contents(output).await?;
    file.write_all(&formatted.into_bytes()).await.map_err(Into::into)
}

/// Format a file using `rustfmt`.
pub(crate) async fn format_file(file: &mut File) -> anyhow::Result<()> {
    let mut contents = String::new();
    file.seek(SeekFrom::Start(0)).await?;
    file.read_to_string(&mut contents).await?;

    let formatted = format_file_contents(contents).await?;
    file.seek(SeekFrom::Start(0)).await?;
    file.set_len(0).await?;
    file.write_all(formatted.as_bytes()).await.map_err(Into::into)
}

/// Format a string using `rustfmt`.
pub(crate) async fn format_file_contents(input: String) -> anyhow::Result<String> {
    // Skip formatting if the input is empty
    if input.is_empty() {
        return Ok(input);
    }

    let mut command = Command::new("rustfmt");
    command.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = command.spawn()?;
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let handle: JoinHandle<anyhow::Result<()>> =
        tokio::spawn(async move { stdin.write_all(input.as_bytes()).await.map_err(Into::into) });

    let mut output = Vec::new();
    tokio::io::copy(&mut stdout, &mut output).await?;

    let status = child.wait().await?;
    handle.await??;

    if status.success() {
        Ok(String::from_utf8(output)?)
    } else {
        Err(anyhow::anyhow!("rustfmt failed"))
    }
}
