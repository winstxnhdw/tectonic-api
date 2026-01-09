#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
name = "install-texlive-bundle"
version = "0.1.0"
edition = "2024"

[dependencies.reqwest]
version = "0.13.1"
default-features = false
features = ["blocking", "default-tls"]

[dependencies.tectonic_bundles]
git = "https://github.com/tectonic-typesetting/tectonic.git"
default-features = false
features = ["geturl-reqwest"]

[dependencies.tectonic_engine_xetex]
git = "https://github.com/tectonic-typesetting/tectonic.git"
default-features = false

[dependencies.tectonic_io_base]
git = "https://github.com/tectonic-typesetting/tectonic.git"
default-features = false
---

use std::fs::File;
use std::fs::create_dir_all;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Read;
use std::io::copy;
use tectonic_bundles::Bundle;
use tectonic_bundles::CachableBundle;
use tectonic_io_base::app_dirs;

struct FileIndexEntry {
    name: String,
    offset: u64,
    length: u64,
}

enum IndexParseError {
    MissingName,
    MissingOffset,
    MissingLength,
    InvalidOffset,
    InvalidLength,
}

fn parse_file_index(line: impl AsRef<str>) -> Result<FileIndexEntry, IndexParseError> {
    let mut fields = line.as_ref().split_whitespace();

    let name = fields.next().ok_or(IndexParseError::MissingName)?.to_string();
    let offset: u64 = fields
        .next()
        .ok_or(IndexParseError::MissingOffset)?
        .parse()
        .map_err(|_| IndexParseError::InvalidOffset)?;
    let length: u64 = fields
        .next()
        .ok_or(IndexParseError::MissingLength)?
        .parse()
        .map_err(|_| IndexParseError::InvalidLength)?;

    Ok(FileIndexEntry { name, offset, length })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle_url = tectonic_bundles::get_fallback_bundle_url(tectonic_engine_xetex::FORMAT_SERIAL);
    let mut bundle = tectonic_bundles::itar::ItarBundle::new(bundle_url.clone())?;
    let digest_hash = bundle.get_digest()?.to_string();
    let bundle_cache = app_dirs::get_user_cache_dir("bundles")?;
    let data_root = bundle_cache.join("data").join(&digest_hash);
    let hashes_directory = bundle_cache.join("hashes");
    let index_path = data_root.with_extension("index");

    create_dir_all(&hashes_directory)?;
    create_dir_all(&data_root)?;

    std::io::copy(
        &mut bundle.get_index_reader()?,
        &mut BufWriter::new(File::create(&index_path)?),
    )?;

    std::fs::write(
        hashes_directory.join(app_dirs::app_dirs2::sanitized(&bundle_url)),
        &digest_hash,
    )?;

    let mut stream_position = 0;
    let mut response = reqwest::blocking::Client::builder()
        .http1_only()
        .build()?
        .get(&bundle_url)
        .send()?
        .error_for_status()?;

    std::io::BufReader::new(File::open(&index_path)?)
        .lines()
        .map_while(Result::ok)
        .flat_map(parse_file_index)
        .try_for_each(|index| {
            copy(
                &mut response.by_ref().take(index.offset.saturating_sub(stream_position)),
                &mut std::io::sink(),
            )?;

            stream_position = index.offset;

            copy(
                &mut response.by_ref().take(index.length),
                &mut BufWriter::new(File::create(data_root.join(&index.name))?),
            )?;

            stream_position += index.length;

            Ok(())
        })
}
