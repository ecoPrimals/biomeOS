// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

fn main() {
    let timestamp = std::process::Command::new("date")
        .arg("--utc")
        .arg("+%Y-%m-%dT%H:%M:%SZ")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string());

    println!("cargo::rustc-env=BIOMEOS_BUILD_TIMESTAMP={timestamp}");
    println!("cargo::rerun-if-changed=build.rs");
}
