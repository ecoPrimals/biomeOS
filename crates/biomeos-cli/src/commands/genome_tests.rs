// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::get_genome_storage_dir;
use super::*;
use biomeos_genomebin_v3::{Arch, GenomeBinBuilder};
use serial_test::serial;
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
#[serial]
fn test_get_genome_storage_dir_with_xdg() {
    use biomeos_test_utils::TestEnvGuard;
    let _guard = TestEnvGuard::new("XDG_DATA_HOME", Some("/tmp/xdg_test"));
    let dir = get_genome_storage_dir();
    assert_eq!(dir, PathBuf::from("/tmp/xdg_test/biomeos/genomes"));
}

#[test]
#[ignore = "modifies env vars; run with --ignored"]
fn test_get_genome_storage_dir_home_fallback() {
    use biomeos_test_utils::remove_test_env;
    let _ = std::env::var("XDG_DATA_HOME").ok();
    remove_test_env("XDG_DATA_HOME");
    let dir = get_genome_storage_dir();
    assert!(dir.to_string_lossy().contains("genomes"));
}

#[test]
fn test_handle_genome_create_binary_not_found() {
    let args = CreateArgs {
        binary: PathBuf::from("/nonexistent/beardog-xyz"),
        output: PathBuf::from("/tmp/out.json"),
        arch: "x86_64".to_string(),
        name: None,
        version: None,
        description: None,
    };
    let result = handle_genome_create(args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Binary not found"));
}

#[test]
fn test_handle_genome_verify_path_not_found() {
    let args = VerifyArgs {
        path: PathBuf::from("/nonexistent/genome-xyz.json"),
    };
    let result = handle_genome_verify(&args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("GenomeBin not found"));
}

#[test]
fn test_handle_genome_verify_invalid_json() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("corrupt.json");
    std::fs::write(&path, b"{ not valid json").expect("write");
    let args = VerifyArgs { path };
    let result = handle_genome_verify(&args);
    assert!(result.is_err());
}

#[test]
fn test_handle_genome_compose_empty_genomes() {
    let args = ComposeArgs {
        name: "tower".to_string(),
        nucleus_type: "TOWER".to_string(),
        genomes: vec![],
        output: PathBuf::from("/tmp/out.json"),
    };
    let result = handle_genome_compose(&args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("No genomes provided"));
}

#[test]
fn test_handle_genome_compose_nonexistent_genome() {
    let args = ComposeArgs {
        name: "tower".to_string(),
        nucleus_type: "TOWER".to_string(),
        genomes: vec![PathBuf::from("/nonexistent/genome.json")],
        output: PathBuf::from("/tmp/out.json"),
    };
    let result = handle_genome_compose(&args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("GenomeBin not found"));
}

#[test]
fn test_handle_genome_list_nonexistent_storage() {
    use biomeos_test_utils::TestEnvGuard;
    let _guard = TestEnvGuard::new("XDG_DATA_HOME", Some("/nonexistent_biomeos_test_xyz"));
    let result = handle_genome_list();
    assert!(result.is_ok());
}

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

#[test]
fn test_handle_genome_create_success_with_temp_binary() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("fake-binary");
    std::fs::write(&binary, b"#!/bin/sh\necho test").expect("write binary");
    let output = temp.path().join("genome.json");

    let args = CreateArgs {
        binary,
        output: output.clone(),
        arch: "x86_64".to_string(),
        name: Some("test-genome".to_string()),
        version: Some("1.0.0".to_string()),
        description: Some("Test genome".to_string()),
    };
    let result = handle_genome_create(args);
    assert!(result.is_ok(), "create should succeed: {:?}", result.err());
    assert!(output.exists(), "output genome should exist");
}

#[test]
fn test_handle_genome_create_uses_name_from_binary_when_not_provided() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("my-primal");
    std::fs::write(&binary, b"binary").expect("write");
    let output = temp.path().join("out.json");

    let args = CreateArgs {
        binary,
        output,
        arch: "x86_64".to_string(),
        name: None,
        version: None,
        description: None,
    };
    let result = handle_genome_create(args);
    assert!(result.is_ok());
}

#[test]
fn test_handle_genome_create_invalid_arch() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("b");
    std::fs::write(&binary, b"x").expect("write");

    let args = CreateArgs {
        binary,
        output: temp.path().join("out.json"),
        arch: "invalid-arch".to_string(),
        name: None,
        version: None,
        description: None,
    };
    let result = handle_genome_create(args);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid architecture")
    );
}

#[tokio::test]
async fn test_execute_build_binary_not_found() {
    let args = GenomeArgs {
        command: GenomeCommand::Build {
            binary: PathBuf::from("/nonexistent/binary"),
            output: PathBuf::from("/tmp/out.json"),
            arch: "x86_64".to_string(),
            name: None,
            version: None,
            description: None,
        },
    };
    let result = execute(args).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Binary not found"));
}

#[tokio::test]
async fn test_execute_build_invalid_arch() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("b");
    std::fs::write(&binary, b"x").expect("write");

    let args = GenomeArgs {
        command: GenomeCommand::Build {
            binary,
            output: temp.path().join("out.json"),
            arch: "badarch".to_string(),
            name: None,
            version: None,
            description: None,
        },
    };
    let result = execute(args).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid architecture")
    );
}

#[tokio::test]
async fn test_execute_verify_path_not_found() {
    let args = GenomeArgs {
        command: GenomeCommand::Verify {
            path: PathBuf::from("/nonexistent/genome.json"),
        },
    };
    let result = execute(args).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("GenomeBin not found")
    );
}

#[tokio::test]
async fn test_execute_extract_genome_not_found() {
    let temp = tempfile::tempdir().expect("temp dir");
    let args = GenomeArgs {
        command: GenomeCommand::Extract {
            genome: PathBuf::from("/nonexistent/genome.json"),
            output: temp.path().to_path_buf(),
        },
    };
    let result = execute(args).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("GenomeBin not found")
    );
}

#[tokio::test]
async fn test_execute_info_path_not_found() {
    let args = GenomeArgs {
        command: GenomeCommand::Info {
            path: PathBuf::from("/nonexistent/genome.json"),
        },
    };
    let result = execute(args).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("GenomeBin not found")
    );
}

#[tokio::test]
async fn test_execute_build_success() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("primal");
    std::fs::write(&binary, b"#!/bin/sh\necho ok").expect("write");
    let output = temp.path().join("genome.json");

    let args = GenomeArgs {
        command: GenomeCommand::Build {
            binary,
            output: output.clone(),
            arch: "x86_64".to_string(),
            name: Some("test".to_string()),
            version: Some("1.0".to_string()),
            description: Some("desc".to_string()),
        },
    };
    let result = execute(args).await;
    assert!(result.is_ok());
    assert!(output.exists());
}

#[tokio::test]
async fn test_execute_verify_success() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("b");
    std::fs::write(&binary, b"x").expect("write");
    let output = temp.path().join("g.json");

    let args = CreateArgs {
        binary,
        output: output.clone(),
        arch: "x86_64".to_string(),
        name: Some("vtest".to_string()),
        version: None,
        description: None,
    };
    handle_genome_create(args).expect("create");

    let exec_args = GenomeArgs {
        command: GenomeCommand::Verify { path: output },
    };
    let result = execute(exec_args).await;
    assert!(result.is_ok());
}

#[test]
fn test_handle_genome_list_with_genomes_under_xdg() {
    let data = tempfile::tempdir().expect("tempdir");
    let _guard = biomeos_test_utils::TestEnvGuard::set(
        "XDG_DATA_HOME",
        data.path().to_str().expect("utf8 path"),
    );
    let store = get_genome_storage_dir();
    std::fs::create_dir_all(&store).expect("mkdir genomes");

    let binary = data.path().join("list-bin");
    std::fs::write(&binary, b"x").expect("binary");
    let out = store.join("listed-genome.json");

    let args = CreateArgs {
        binary,
        output: out,
        arch: "x86_64".to_string(),
        name: Some("listed-genome".to_string()),
        version: None,
        description: None,
    };
    handle_genome_create(args).expect("create in storage dir");

    let result = handle_genome_list();
    assert!(result.is_ok(), "{:?}", result.err());
}

#[tokio::test]
async fn test_execute_extract_no_binary_for_foreign_arch() {
    let temp = tempfile::tempdir().expect("tempdir");
    let binary = temp.path().join("foreign-bin");
    std::fs::write(&binary, b"x").expect("write");

    let foreign = if cfg!(target_arch = "x86_64") {
        Arch::Aarch64
    } else {
        Arch::X86_64
    };

    let mut builder = GenomeBinBuilder::new("foreign-only");
    builder = builder.add_binary(foreign, &binary);
    let genome = builder.build().expect("build");
    let gpath = temp.path().join("foreign.json");
    genome.save(&gpath).expect("save");

    let out_dir = temp.path().join("extract-out");
    std::fs::create_dir_all(&out_dir).expect("out");

    let args = GenomeArgs {
        command: GenomeCommand::Extract {
            genome: gpath,
            output: out_dir,
        },
    };
    let err = execute(args)
        .await
        .expect_err("should fail: no native binary");
    let msg = err.to_string();
    assert!(
        msg.contains("No binary") || msg.contains("architecture"),
        "{msg}"
    );
}

#[tokio::test]
async fn test_execute_verify_invalid_json() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("bad.json");
    std::fs::write(&path, b"[1,2,3").expect("write");

    let args = GenomeArgs {
        command: GenomeCommand::Verify { path },
    };
    let result = execute(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_info_success() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("b");
    std::fs::write(&binary, b"x").expect("write");
    let output = temp.path().join("g.json");

    let args = CreateArgs {
        binary,
        output: output.clone(),
        arch: "x86_64".to_string(),
        name: Some("info-test".to_string()),
        version: Some("2.0".to_string()),
        description: Some("Info test genome".to_string()),
    };
    handle_genome_create(args).expect("create");

    let exec_args = GenomeArgs {
        command: GenomeCommand::Info { path: output },
    };
    let result = execute(exec_args).await;
    assert!(result.is_ok());
}

#[test]
fn test_handle_genome_self_replicate() {
    let result = handle_genome_self_replicate();
    assert!(result.is_ok());
}

#[test]
fn test_handle_genome_compose_two_genomes_success() {
    let temp = tempfile::tempdir().expect("tempdir");
    let b1 = temp.path().join("b1");
    let b2 = temp.path().join("b2");
    std::fs::write(&b1, b"bin1").unwrap();
    std::fs::write(&b2, b"bin2").unwrap();

    let g1 = temp.path().join("g1.json");
    let g2 = temp.path().join("g2.json");
    let out = temp.path().join("composed.json");

    // TOWER composition requires manifest names `beardog` and `songbird`
    handle_genome_create(CreateArgs {
        binary: b1,
        output: g1.clone(),
        arch: "x86_64".to_string(),
        name: Some("beardog".to_string()),
        version: None,
        description: None,
    })
    .unwrap();
    handle_genome_create(CreateArgs {
        binary: b2,
        output: g2.clone(),
        arch: "x86_64".to_string(),
        name: Some("songbird".to_string()),
        version: None,
        description: None,
    })
    .unwrap();

    let args = ComposeArgs {
        name: "tower".to_string(),
        nucleus_type: "TOWER".to_string(),
        genomes: vec![g1, g2],
        output: out.clone(),
    };
    assert!(handle_genome_compose(&args).is_ok());
    assert!(out.exists());
}

#[tokio::test]
async fn test_execute_verify_checksum_failure() {
    let temp = tempfile::tempdir().expect("tempdir");
    let binary = temp.path().join("b");
    std::fs::write(&binary, b"x").unwrap();
    let path = temp.path().join("tampered.json");
    handle_genome_create(CreateArgs {
        binary,
        output: path.clone(),
        arch: "x86_64".to_string(),
        name: Some("tamper".to_string()),
        version: None,
        description: None,
    })
    .unwrap();
    let mut raw = std::fs::read_to_string(&path).unwrap();
    raw.push_str("\n\"corrupt\":true\n");
    std::fs::write(&path, raw).unwrap();

    let args = GenomeArgs {
        command: GenomeCommand::Verify { path },
    };
    assert!(execute(args).await.is_err());
}

#[test]
fn test_get_genome_storage_dir_fallback_home() {
    let _xdg = biomeos_test_utils::TestEnvGuard::remove("XDG_DATA_HOME");
    let dir = get_genome_storage_dir();
    assert!(dir.to_string_lossy().contains("genomes"));
}
