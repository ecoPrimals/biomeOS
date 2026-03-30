// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Remote acquisition: GitHub API, HTTPS downloads, cache paths, checksums.

use anyhow::Result;
use biomeos_types::SystemPaths;
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// GitHub API: release payload (subset).
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub(crate) tag_name: String,
    pub(crate) assets: Vec<GitHubAsset>,
}

/// GitHub API: release asset (subset).
#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub(crate) name: String,
    pub(crate) browser_download_url: String,
}

pub fn registry_cache_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("BIOMEOS_REGISTRY_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let paths = SystemPaths::new_lazy();
    Ok(paths.data_dir().join("registry"))
}

pub fn sanitize_cache_component(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn cached_download_path(
    url: &str,
    org: &str,
    repo: &str,
    tag: &str,
    asset: &str,
) -> Result<PathBuf> {
    use sha2::{Digest, Sha256};
    let base = registry_cache_dir()?;
    let digest_hex = format!("{:x}", Sha256::digest(url.as_bytes()));
    let name = if org.is_empty() && repo.is_empty() {
        format!("url_{digest_hex}")
    } else {
        let short = digest_hex
            .get(..8)
            .ok_or_else(|| anyhow::anyhow!("digest string unexpectedly short"))?;
        format!(
            "{}_{}_{}_{}_{}",
            sanitize_cache_component(org),
            sanitize_cache_component(repo),
            sanitize_cache_component(tag),
            sanitize_cache_component(asset),
            short
        )
    };
    Ok(base.join(name))
}

pub async fn github_latest_release(org: &str, repo: &str) -> Result<GitHubRelease> {
    let api_url = format!("https://api.github.com/repos/{org}/{repo}/releases/latest");
    github_api_get(&api_url).await
}

pub async fn github_release_for_tag(org: &str, repo: &str, tag: &str) -> Result<GitHubRelease> {
    let enc = percent_encode_github_tag(tag);
    let api_url = format!("https://api.github.com/repos/{org}/{repo}/releases/tags/{enc}");
    github_api_get(&api_url).await
}

fn percent_encode_github_tag(tag: &str) -> String {
    tag.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

async fn github_api_get(api_url: &str) -> Result<GitHubRelease> {
    let body = curl_fetch_https(api_url).await?;
    let release: GitHubRelease = serde_json::from_slice(&body)
        .map_err(|e| anyhow::anyhow!("Failed to parse GitHub API JSON from {api_url}: {e}"))?;
    Ok(release)
}

/// HTTPS GET via `curl` (pure-Rust binary; avoids linking TLS stacks).
async fn curl_fetch_https(url: &str) -> Result<Vec<u8>> {
    let mut cmd = tokio::process::Command::new("curl");
    cmd.arg("-fsSL")
        .arg("--max-time")
        .arg("120")
        .arg("-H")
        .arg("User-Agent: biomeos-primal-registry/0.1")
        .arg("-H")
        .arg("Accept: application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            cmd.arg("-H").arg(format!("Authorization: Bearer {token}"));
        }
    }
    cmd.arg(url);
    let output = cmd.output().await.map_err(|e| {
        anyhow::anyhow!("Failed to run curl for HTTPS (install curl or set PATH): {e}")
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("curl HTTPS request failed: {stderr}"));
    }
    Ok(output.stdout)
}

async fn curl_download_to_file(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut cmd = tokio::process::Command::new("curl");
    cmd.arg("-fsSL")
        .arg("--max-time")
        .arg("600")
        .arg("-L")
        .arg("-H")
        .arg("User-Agent: biomeos-primal-registry/0.1");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            cmd.arg("-H").arg(format!("Authorization: Bearer {token}"));
        }
    }
    cmd.arg("-o").arg(dest.as_os_str()).arg(url);
    let status = cmd.status().await.map_err(|e| {
        anyhow::anyhow!("Failed to run curl for download (install curl or set PATH): {e}")
    })?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "curl download failed with status {:?}",
            status.code()
        ));
    }
    Ok(())
}

async fn http_get_bytes(url: &str) -> Result<Vec<u8>> {
    let client = Client::builder(TokioExecutor::new()).build_http::<Empty<Bytes>>();
    let uri = url
        .parse::<hyper::Uri>()
        .map_err(|e| anyhow::anyhow!("invalid URL: {e}"))?;
    let res = client
        .get(uri)
        .await
        .map_err(|e| anyhow::anyhow!("HTTP client error: {e}"))?;
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("HTTP {}", res.status()));
    }
    let body = res.into_body();
    let collected = body
        .collect()
        .await
        .map_err(|e| anyhow::anyhow!("read body: {e}"))?;
    Ok(collected.to_bytes().to_vec())
}

pub async fn download_url_to_path_with_verify(
    url: &str,
    dest: &Path,
    checksum_hex: Option<&str>,
) -> Result<()> {
    let parsed = url::Url::parse(url)?;
    match parsed.scheme() {
        "https" => curl_download_to_file(url, dest).await?,
        "http" => {
            let bytes = http_get_bytes(url).await?;
            if let Some(parent) = dest.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(dest, &bytes).await?;
        }
        other => {
            return Err(anyhow::anyhow!("unsupported URL scheme: {other}"));
        }
    }
    set_executable_unix(dest)?;
    verify_checksum_optional(dest, checksum_hex).await?;
    Ok(())
}

/// chmod +x on Unix; no-op on other platforms.
fn set_executable_unix(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

async fn verify_checksum_optional(path: &Path, expected: Option<&str>) -> Result<()> {
    let Some(exp) = expected else {
        return Ok(());
    };
    let exp = exp.trim().to_lowercase();
    let exp = exp.strip_prefix("sha256:").map_or(exp.as_str(), str::trim);
    let actual = compute_checksum_file(path).await?;
    if actual != exp {
        return Err(anyhow::anyhow!(
            "SHA256 mismatch: expected {exp}, got {actual}"
        ));
    }
    Ok(())
}

pub async fn compute_checksum_file(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let contents = tokio::fs::read(path).await?;
    let hash = Sha256::digest(&contents);
    Ok(format!("{hash:x}"))
}

pub fn is_skippable_non_binary_asset(name: &str) -> bool {
    let n = name.to_lowercase();
    let ext_matches = |ext: &str| {
        std::path::Path::new(&n)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
    };
    ext_matches("sha256")
        || ext_matches("asc")
        || ext_matches("sig")
        || ext_matches("txt")
        || ext_matches("md")
        || ext_matches("json")
        || ext_matches("yml")
        || ext_matches("yaml")
}

pub fn asset_matches_platform(name: &str) -> bool {
    let n = name.to_lowercase();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let os_ok = match os {
        "linux" => {
            n.contains("linux")
                || n.contains("unknown-linux")
                || n.contains("gnu")
                || n.contains("musl")
        }
        "macos" => {
            n.contains("darwin") || n.contains("macos") || n.contains("apple") || n.contains("osx")
        }
        "windows" => {
            n.contains("windows")
                || n.contains("win")
                || n.contains("msvc")
                || std::path::Path::new(&n)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
        }
        _ => true,
    };

    if !os_ok {
        return false;
    }

    match arch {
        "x86_64" => {
            n.contains("x86_64") || n.contains("amd64") || n.contains("x64") || n.contains("x86-64")
        }
        "aarch64" => n.contains("aarch64") || n.contains("arm64"),
        other => n.contains(&other.to_lowercase()),
    }
}

pub fn extract_version_from_output(text: &str) -> Option<String> {
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    for tok in line.split_whitespace() {
        let t = tok.trim_matches(|c| c == 'v' || c == '(' || c == ')' || c == '[' || c == ']');
        if t.chars().next().is_some_and(|c| c.is_ascii_digit()) && t.contains('.') {
            return Some(t.to_string());
        }
    }
    if line.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return Some(line.to_string());
    }
    None
}
