// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Test fixtures for synthetic genomeBin files.

use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::Write;
use tar::{Builder, Header};

/// Minimal genomeBin: marker + gzipped tar with `{arch}/stub` executable script.
pub(crate) fn write_genome_bin_with_arch_dir(path: &std::path::Path, arch_dir: &str) {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut tar = Builder::new(&mut encoder);
        let script =
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo \"stub 1.0.0\"; fi\nexit 0\n";
        let mut header = Header::new_gnu();
        let inner_path = format!("{arch_dir}/stub");
        header.set_path(&inner_path).expect("path");
        header.set_size(script.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append(&header, &script[..]).expect("append");
        tar.finish().expect("finish");
    }
    let compressed = encoder.finish().expect("gz finish");
    let mut f = File::create(path).expect("create genome");
    f.write_all(b"stub-genome-header\n__ARCHIVE_START__\n")
        .expect("marker");
    f.write_all(&compressed).expect("gz");
}
