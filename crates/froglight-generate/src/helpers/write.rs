#![allow(dead_code)]

use std::{path::Path, process::Stdio};

use quote::ToTokens;
use tokio::{fs::File, io::AsyncWriteExt, process::Command, task::JoinHandle};

use super::update_file_tag;

/// Write tokens to a file.
///
/// This will overwrite the file if it already exists.
pub(crate) async fn write_tokens_to_file(tokens: impl ToTokens, path: &Path) -> anyhow::Result<()> {
    let mut file = File::create(path).await?;
    append_tokens_to_file(tokens, &mut file).await?;
    update_file_tag(&mut file).await
}

/// Write code to a file.
///
/// This will overwrite the file if it already exists.
pub(crate) async fn write_to_file(output: String, path: &Path) -> anyhow::Result<()> {
    let mut file = File::create(path).await?;
    append_to_file(output, &mut file).await?;
    update_file_tag(&mut file).await
}

/// Append tokens to a file.
pub(crate) async fn append_tokens_to_file(
    tokens: impl ToTokens,
    file: &mut File,
) -> anyhow::Result<()> {
    let parsed = syn::parse2(tokens.into_token_stream())?;
    let unparsed = prettyplease::unparse(&parsed);
    append_to_file(unparsed, file).await
}

/// Append code to a file.
pub(crate) async fn append_to_file(output: String, file: &mut File) -> anyhow::Result<()> {
    let formatted = format(output).await?;
    file.write_all(&formatted.into_bytes()).await.map_err(Into::into)
}

/// Format a string using `rustfmt`.
pub(crate) async fn format(input: String) -> anyhow::Result<String> {
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
