// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

/// Boot target type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootTarget {
    /// ISO image for optical media or virtual machines.
    Iso,
    /// USB flash drive bootable image.
    Usb,
}
