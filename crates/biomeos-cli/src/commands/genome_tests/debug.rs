// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::path::PathBuf;

#[test]
fn test_create_args_debug() {
    let args = CreateArgs {
        binary: PathBuf::from("/tmp/b"),
        output: PathBuf::from("/tmp/o"),
        arch: "x86_64".to_string(),
        name: Some("test".to_string()),
        version: Some("1.0".to_string()),
        description: Some("desc".to_string()),
    };
    let _ = format!("{args:?}");
}

#[test]
fn test_compose_args_debug() {
    let args = ComposeArgs {
        name: "tower".to_string(),
        nucleus_type: "TOWER".to_string(),
        genomes: vec![PathBuf::from("a.json")],
        output: PathBuf::from("/tmp/out.json"),
    };
    let _ = format!("{args:?}");
}

#[test]
fn test_verify_args_debug() {
    let args = VerifyArgs {
        path: PathBuf::from("/tmp/g.json"),
    };
    let _ = format!("{args:?}");
}

#[test]
fn test_genome_args_debug() {
    let args = GenomeArgs {
        command: GenomeCommand::Verify {
            path: PathBuf::from("/tmp/g.json"),
        },
    };
    let _ = format!("{args:?}");
}

#[test]
fn test_genome_command_build_variant_debug() {
    let cmd = GenomeCommand::Build {
        binary: PathBuf::from("/tmp/b"),
        output: PathBuf::from("/tmp/o.json"),
        arch: "x86_64".to_string(),
        name: Some("test".to_string()),
        version: Some("1.0".to_string()),
        description: Some("desc".to_string()),
    };
    let _ = format!("{cmd:?}");
}

#[test]
fn test_genome_command_extract_variant_debug() {
    let cmd = GenomeCommand::Extract {
        genome: PathBuf::from("/tmp/g.json"),
        output: PathBuf::from("/tmp/out"),
    };
    let _ = format!("{cmd:?}");
}

#[test]
fn test_genome_command_info_variant_debug() {
    let cmd = GenomeCommand::Info {
        path: PathBuf::from("/tmp/g.json"),
    };
    let _ = format!("{cmd:?}");
}
