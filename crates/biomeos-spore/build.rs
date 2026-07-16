// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

fn main() {
    let git_commit = std::env::var("GIT_COMMIT_HASH")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|hash| hash.trim().to_string())
                .filter(|hash| !hash.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo::rustc-env=GIT_COMMIT_HASH={git_commit}");
    println!("cargo::rerun-if-env-changed=GIT_COMMIT_HASH");
    println!("cargo::rerun-if-changed=build.rs");
}
