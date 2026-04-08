// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use std::path::Path;

use super::BootableMediaBuilder;

impl BootableMediaBuilder {
    pub(crate) fn copy_directory(src: &Path, dest: &Path) -> Result<()> {
        std::fs::create_dir_all(dest)
            .with_context(|| format!("Failed to create directory: {}", dest.display()))?;

        for entry in std::fs::read_dir(src)
            .with_context(|| format!("Failed to read directory: {}", src.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().context("Invalid file name")?;
            let dest_path = dest.join(file_name);

            if path.is_dir() {
                Self::copy_directory(&path, &dest_path)?;
            } else {
                std::fs::copy(&path, &dest_path).with_context(|| {
                    format!(
                        "Failed to copy {} to {}",
                        path.display(),
                        dest_path.display()
                    )
                })?;
            }
        }

        Ok(())
    }
}
