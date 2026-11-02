# Deployment and CI/CD Guide

Rusty Chain Kit includes an enterprise-grade Continuous Integration and Continuous Deployment (CI/CD) automation pipeline using GitHub Actions.

---

## 1. CI/CD Architecture

```
[ Git Push / Pull Request ]
             |
             +---> [ `rust.yml` ]: Build & 202 Integration Tests
             +---> [ `lint.yml` ]: Format Check (`cargo fmt`) & Clippy Linter
             +---> [ `docs.yml` ]: API Documentation Build (`cargo doc`)
             +---> [ `audit.yml` ]: Dependency Security Audit (`cargo audit`)
             +---> [ `codeql.yml` ]: CodeQL Static Security Analysis
             |
[ On Version Tag `v*` ]
             |
             +---> [ `release.yml` ]: Build & Publish Multi-Platform Binaries
             +---> [ `docker.yml` ]: Build & Push Container Image to GHCR
```

---

## 2. Automated Workflow Suites

### 1. Primary CI (`rust.yml`)
- Executes on every push and pull request to `main`.
- Compiles debug binaries with `dtolnay/rust-toolchain@stable`.
- Employs `Swatinem/rust-cache@v2` for cross-run build caching.
- Executes all **202 integration tests** across 8 independent test suites.

### 2. Code Quality & Formatting Gate (`lint.yml`)
- Enforces strict rustfmt styling across all modules via `cargo fmt --check`.
- Runs `cargo clippy --all-targets --all-features` to enforce idiomatic Rust practices.

### 3. API Documentation Verification (`docs.yml`)
- Compiles project documentation via `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"`.
- Archives generated HTML documentation as a downloadable workflow artifact.

### 4. Dependency Security Audit (`audit.yml`)
- Periodically scans all 518+ crate dependencies against the RustSec Advisory Database.
- Configured via `.cargo/audit.toml` to enforce compliance and report new vulnerabilities.

### 5. Multi-Platform Release Packaging (`release.yml`)
- Triggers on version tags (`v*`).
- Compiles stripped, optimized release binaries across a 3-platform matrix:
  - `x86_64-unknown-linux-gnu` (Linux x86_64)
  - `aarch64-apple-darwin` (macOS Apple Silicon)
  - `x86_64-apple-darwin` (macOS Intel)
- Packages binaries into compressed `.tar.gz` archives and publishes them directly to the GitHub Release page with generated changelogs.

### 6. Container Image Publisher (`docker.yml`)
- Builds multi-stage production Docker images and publishes to GitHub Container Registry (`ghcr.io/aetherx86/rusty-chain-kit`).
- Tags images with `latest`, branch names, and semver tags (`v0.1.1-beta`).

---

## 3. Publishing a New Release

To publish a new official release of Rusty Chain Kit:

1. **Update Version**:
   Update `version` in [Cargo.toml](file:///Users/sushikim/Documents/Paradigm/rusty-chain-kit/Cargo.toml#L3) (e.g., `0.1.2`).
2. **Commit and Tag**:
   ```bash
   git commit -am "chore: Release v0.1.2"
   git tag v0.1.2
   git push origin main --tags
   ```
3. **Automated Publishing**:
   GitHub Actions will automatically build the release binaries, upload the archives to GitHub Releases, and push the updated Docker container image to GHCR.
