// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use biomeos_genomebin_v3::{Arch, GenomeBinBuilder};
use std::path::PathBuf;

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
    let result = execute(args);
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
    let result = execute(args);
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
    let result = execute(args);
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
    let result = execute(args);
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
    let result = execute(args);
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
    let result = execute(args);
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
    let result = execute(exec_args);
    assert!(result.is_ok());
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
    let err = execute(args).expect_err("should fail: no native binary");
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
    let result = execute(args);
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
    let result = execute(exec_args);
    assert!(result.is_ok());
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
    assert!(execute(args).is_err());
}
