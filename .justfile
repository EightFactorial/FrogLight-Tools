#!/usr/bin/env just --justfile

# List available recipes
default:
  @just --list

# ---- Test Recipes ----

# Run all tests and doc-tests
all-tests: (deny) (clippy) (tests) (doc-tests)

# Run cargo deny
deny:
  cargo deny check

# Run clippy
clippy:
  cargo clippy --workspace

# Run all tests
tests: (fetch-nextest)
  cargo nextest run --workspace

# Run all doc-tests
doc-tests:
  cargo test --doc --workspace

# ---- Tool Recipes ----

# Re-run `just` without the `tools` argument
tools args="": (fetch-tools)
  @just {{args}}

# ---- Fetch Recipes ----

# Fetch `froglight-tools` submodule if not present
[private]
fetch-tools:
  @if [ ! -f tools/.justfile ]; then git submodule update; fi

# Fetch `nextest` if not present
[private]
fetch-nextest:
  @-cargo nextest --version > /dev/null 2>&1
  @if [ $? -ne 0 ]; then cargo install nextest; fi
