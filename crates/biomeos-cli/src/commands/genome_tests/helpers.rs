// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::get_genome_storage_dir_with;
use super::super::*;
use biomeos_genomebin_v3::Arch;
use std::path::{Path, PathBuf};

#[test]
fn test_extract_genome_name_from_path() {
    assert_eq!(
        extract_genome_name_from_path(Path::new("/usr/bin/beardog")),
        "beardog"
    );
    assert_eq!(
        extract_genome_name_from_path(Path::new("tower-x86_64")),
        "tower-x86_64"
    );
    assert_eq!(extract_genome_name_from_path(Path::new("nest")), "nest");
}

#[test]
fn test_extract_genome_name_from_path_empty() {
    assert_eq!(extract_genome_name_from_path(Path::new("")), "genome");
}

#[test]
fn test_extract_genome_name_from_path_dotfile() {
    assert_eq!(
        extract_genome_name_from_path(Path::new("/tmp/.hidden")),
        ".hidden"
    );
}

#[test]
fn test_parse_arch_valid() {
    assert!(matches!(parse_arch("x86_64").unwrap(), Arch::X86_64));
    assert!(matches!(parse_arch("aarch64").unwrap(), Arch::Aarch64));
    assert!(matches!(parse_arch("arm").unwrap(), Arch::Arm));
    assert!(matches!(parse_arch("riscv64").unwrap(), Arch::Riscv64));
}

#[test]
fn test_parse_arch_invalid() {
    let result = parse_arch("invalid");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid architecture"));
    assert!(err.to_string().contains("x86_64"));
}

#[test]
fn test_parse_arch_empty() {
    let result = parse_arch("");
    assert!(result.is_err());
}

#[test]
fn test_parse_arch_case_sensitive() {
    assert!(parse_arch("X86_64").is_err());
    assert!(parse_arch("AARCH64").is_err());
}

#[test]
fn test_get_genome_storage_dir_with_xdg() {
    let dir = get_genome_storage_dir_with(Some("/tmp/xdg_test"));
    assert_eq!(dir, PathBuf::from("/tmp/xdg_test/biomeos/genomes"));
}

#[test]
fn test_extract_genome_name_nested_path() {
    assert_eq!(
        extract_genome_name_from_path(Path::new("/opt/primals/tower-bin")),
        "tower-bin"
    );
}

#[test]
fn test_parse_arch_riscv64() {
    assert!(matches!(parse_arch("riscv64").unwrap(), Arch::Riscv64));
}

#[test]
fn test_create_args_default_arch_string() {
    let args = CreateArgs {
        binary: PathBuf::from("/b"),
        output: PathBuf::from("/o"),
        arch: "x86_64".to_string(),
        name: None,
        version: None,
        description: None,
    };
    assert_eq!(args.arch, "x86_64");
}

#[test]
fn test_compose_args_nucleus_type_default() {
    let args = ComposeArgs {
        name: "n".to_string(),
        nucleus_type: "TOWER".to_string(),
        genomes: vec![],
        output: PathBuf::from("/o"),
    };
    assert_eq!(args.nucleus_type, "TOWER");
}
