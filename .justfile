#!/usr/bin/env just --justfile

# List available recipes
default:
  @just --list

# Clean build artifacts
clean:
  cargo clean

# ---- Test Recipes ----

# Run all tests and all tool tests
all-tests: (update) (deny) (fmt) (test)

# Run all tests and doc-tests
test: (nextest) (doc-test) 

# Run all tests
nextest: (fetch-nextest)
  cargo nextest run --workspace

# Run all doc-tests
doc-test:
  cargo test --doc --workspace

# ---- Tool Recipes ----

# Run cargo deny
deny:
  cargo deny check

# Run cargo update
update:
  cargo update

# Run clippy
clippy:
  cargo clippy --workspace

# Run cargo fmt
fmt:
  cargo fmt --all

# Run the froglight-extractor with the given command and arguments
# Uses the ./target (./target/froglight) directory as the cache
extract version arg0="" arg1="" arg2="" arg3="":
  cargo run --package froglight-extractor --features=binary --release -- --cache ../target --version {{version}} {{arg0}} {{arg1}} {{arg2}} {{arg3}}

# Run the froglight-generator to generate code dynamically
generate arg0="" arg1="" arg2="" arg3="":
  cargo run --package froglight-generator --release -- --dir ../ {{arg0}} {{arg1}} {{arg2}} {{arg3}}

# Re-run `just` without the `tools` argument
tools args="":
  @just {{args}}

# ---- Fetch Recipes ----

# Fetch `nextest` if not present
[private]
fetch-nextest:
  @-cargo nextest --version > /dev/null 2>&1
  @if [ $? -ne 0 ]; then cargo install nextest; fi
