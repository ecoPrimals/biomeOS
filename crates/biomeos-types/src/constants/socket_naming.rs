// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Standard socket path builder and filename parser for ecosystem consistency.
//!
//! Convention: `{socket_dir}/{primal_name}-{family_id}.sock`

use std::path::{Path, PathBuf};

/// Standard socket path builder for ecosystem consistency.
///
/// Convention: `{socket_dir}/{primal_name}-{family_id}.sock`
#[must_use]
pub fn primal_socket_path(socket_dir: &Path, primal_name: &str, family_id: &str) -> PathBuf {
    socket_dir.join(format!("{primal_name}-{family_id}.sock"))
}

/// Parse a socket filename back into `(primal_name, family_id)`.
///
/// Uses the last `-` separator so primal names may contain hyphens
/// (e.g. `my-primal-fam1.sock` → `("my-primal", "fam1")`).
#[must_use]
pub fn parse_socket_filename(filename: &str) -> Option<(&str, &str)> {
    let stem = filename.strip_suffix(".sock")?;
    let (name, family) = stem.rsplit_once('-')?;
    Some((name, family))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_naming_primal_socket_path() {
        let dir = Path::new("/run/membrane/sockets");
        let path = primal_socket_path(dir, "beardog", "fam1");
        assert_eq!(path, PathBuf::from("/run/membrane/sockets/beardog-fam1.sock"));
    }

    #[test]
    fn socket_naming_parse_simple() {
        assert_eq!(
            parse_socket_filename("beardog-fam1.sock"),
            Some(("beardog", "fam1"))
        );
    }

    #[test]
    fn socket_naming_parse_hyphenated_primal() {
        assert_eq!(
            parse_socket_filename("my-primal-fam1.sock"),
            Some(("my-primal", "fam1"))
        );
    }

    #[test]
    fn socket_naming_parse_rejects_non_sock() {
        assert_eq!(parse_socket_filename("beardog-fam1.pid"), None);
    }

    #[test]
    fn socket_naming_roundtrip() {
        let dir = Path::new("/tmp/sockets");
        let path = primal_socket_path(dir, "songbird", "abc123");
        let filename = path.file_name().and_then(|n| n.to_str()).expect("filename");
        assert_eq!(parse_socket_filename(filename), Some(("songbird", "abc123")));
    }
}
