#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
name = "install-texlive-bundle"
version = "0.1.0"
edition = "2024"

[dependencies]
tectonic = { git = "https://github.com/tectonic-typesetting/tectonic.git", default-features = false, features = ["geturl-reqwest"] }
---

fn main() -> tectonic::errors::Result<()> {
    let mut status = tectonic::status::NoopStatusBackend::default();
    let mut bundle = tectonic::config::PersistentConfig::open(false)?.default_bundle(false)?;

    bundle.all_files().iter().for_each(|name| {
        match bundle.input_open_name(name, &mut status).must_exist() {
            Ok(_) => println!("Cached {}", name),
            Err(e) => eprintln!("Failed {}: {}", name, e),
        }
    });

    Ok(())
}
