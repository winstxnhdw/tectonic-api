#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
name = "collect-profiles"
version = "0.1.0"
edition = "2024"

[dependencies.reqwest]
version = "0.13.1"
default-features = false
features = ["blocking", "default-tls", "query"]
---

use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder().http1_only().build()?;
    let binary = std::env::args().nth(1).expect("usage: collect_profiles.rs <binary>");
    let mut server = Command::new(&binary)
        .env("LLVM_PROFILE_FILE", "/target/pgo-profiles/default_%m_%p.profraw")
        .spawn()?;

    loop {
        if client.get("http://localhost:5555/api/health").send().is_ok() {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    for entry in fs::read_dir("scripts/profiles")?.filter_map(Result::ok) {
        if entry.path().extension().is_none_or(|ext| ext != "tex") {
            continue;
        }

        for _ in 0..30 {
            client
                .get("http://localhost:5555/api/v2/pdf")
                .query(&[("source", &fs::read_to_string(entry.path())?)])
                .send()?
                .error_for_status()?;
        }
    }

    // Send SIGTERM (not SIGKILL) so PGO/BOLT instrumentation can flush profile data
    Command::new("/bin/sh")
        .args(["-c", &format!("kill {}", server.id())])
        .status()?;

    server.wait()?;

    Ok(())
}
