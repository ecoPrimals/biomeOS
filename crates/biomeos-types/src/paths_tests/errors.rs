use super::super::*;
use tempfile::tempdir;

#[test]
fn test_path_error_display() {
    let err = PathError::InvalidPath("bad-path".to_string());
    assert!(err.to_string().contains("Invalid path"));
    assert!(err.to_string().contains("bad-path"));
}
#[test]
fn test_empty_primal_id_in_socket() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();
    let socket = paths.primal_socket("");
    assert!(socket.ends_with(".sock"));
}

#[test]
fn test_safe_uid() {
    let uid = safe_uid();
    assert_ne!(uid, 0, "safe_uid should return non-zero value");
}

#[test]
fn test_path_error_create_dir_failed_display() {
    let err = PathError::CreateDirFailed {
        path: "/invalid/path".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    };
    let s = err.to_string();
    assert!(s.contains("Failed to create directory"));
    assert!(s.contains("/invalid/path"));
}

#[test]
fn test_path_error_no_home_dir_display() {
    let err = PathError::NoHomeDir;
    let s = err.to_string();
    assert!(s.to_lowercase().contains("home"));
}

#[test]
fn test_path_error_invalid_path_display() {
    let err = PathError::InvalidPath("bad/path".to_string());
    let s = err.to_string();
    assert!(s.contains("Invalid path"));
    assert!(s.contains("bad/path"));
}

#[test]
fn test_path_error_debug() {
    let err = PathError::NoHomeDir;
    let s = format!("{:?}", err);
    assert!(s.contains("NoHomeDir"));
}
#[test]
fn test_path_error_invalid_path_and_no_home_debug_display() {
    let inv = PathError::InvalidPath("x/y".to_string());
    assert!(inv.to_string().contains("Invalid path"));
    assert!(inv.to_string().contains("x/y"));
    let inv_dbg = format!("{inv:?}");
    assert!(inv_dbg.contains("InvalidPath") || inv_dbg.contains("x/y"));

    let no_home = PathError::NoHomeDir;
    assert_eq!(no_home.to_string(), "Failed to determine home directory");
    assert!(format!("{no_home:?}").contains("NoHomeDir"));
}
