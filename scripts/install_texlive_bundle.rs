#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
name = "install-texlive-bundle"
version = "0.1.0"
edition = "2024"

[dependencies.tectonic]
git = "https://github.com/tectonic-typesetting/tectonic.git"
default-features = false
features = ["geturl-reqwest", "external-harfbuzz"]
---

fn main() -> tectonic::errors::Result<()> {
    let mut status = tectonic::status::NoopStatusBackend::default();
    let mut bundle = tectonic::config::PersistentConfig::open(false)?.default_bundle(false)?;

    for name in bundle.all_files() {
        bundle.input_open_name(&name, &mut status).must_exist()?;
    }

    Ok(())
}
