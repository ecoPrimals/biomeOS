#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
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
        binary: binary.clone(),
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
