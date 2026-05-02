# CI/CD Pipeline

**Last Updated:** May 2, 2026

---

## Workflows

biomeOS uses GitHub Actions with two workflows:

### 1. `ci.yml` — Code Quality Gate

**Triggers:** Push to `main`, pull requests to `main`

**Single job (`check`)** on `ubuntu-latest` with 20-minute timeout:

| Step | Command |
|------|---------|
| **Format** | `cargo fmt --all -- --check` |
| **Clippy** | `cargo clippy --workspace --all-targets -- -D warnings` |
| **Test** | `cargo test --workspace --lib` |

Uses `dtolnay/rust-toolchain@stable` with `rustfmt` + `clippy` components and `Swatinem/rust-cache@v2` for build caching. Concurrency groups cancel in-progress runs on the same ref.

### 2. `notify-plasmidbin.yml` — Binary Distribution Trigger

**Triggers:** Push to `main` only

Sends a `repository-dispatch` event to `ecoPrimals/plasmidBin` with the primal name and commit SHA. plasmidBin's `auto-harvest.yml` rebuilds and distributes the binary. This is the mechanism by which downstream springs (primalSpring, ludoSpring, etc.) receive updated biomeOS binaries.

---

## Local Pre-Push Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib
```

Coverage, security audit, and `cargo deny` are run locally on demand — not in CI (kept lean for fast feedback).

---

## Quality Standards (enforced locally, verified in CI)

| Check | Status |
|-------|--------|
| Formatting | Enforced (`cargo fmt`) |
| Linting | Enforced (`clippy -D warnings`, pedantic+nursery via workspace lints) |
| Tests | 8,076+ passing (0 failures, fully concurrent) |
| Coverage | 90%+ line / function / region (llvm-cov, local) |
| Unsafe code | 0 (`#[forbid(unsafe_code)]` on all crate roots) |
| TODO/FIXME/HACK | 0 in production code |
| C dependencies | 0 |
| File size | 0 production files >800 lines |

---

## References

- **Current Status:** `../../CURRENT_STATUS.md`
- **Contributing:** `../../CONTRIBUTING.md`
- **Standards:** `ecoPrimals/wateringHole/STANDARDS_AND_EXPECTATIONS.md`
