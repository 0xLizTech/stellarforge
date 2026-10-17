# Releasing `@stellarforge-protocol/sdk`

The SDK is published by `.github/workflows/publish-sdk.yml` using npm trusted
publishing (OIDC). npm accepts a publish only from that workflow, in
`0xLizTech/stellarforge`, running in the `npm-publish` GitHub environment. No
npm token exists, and npm attaches a provenance attestation to each version.

## One-time setup

An owner of the `stellarforge-protocol` npm organization does this once.

1. **Publish the first version by hand.** npm can only configure trusted
   publishing for a package that already exists.

   ```bash
   git checkout main && git pull
   cd sdk && npm ci && npm test && npm run build
   npm publish --access public
   ```

   This version has no provenance attestation. Every later one does.

2. **Configure the trusted publisher.** On npmjs.com, open the package's
   **Settings → Trusted Publisher → GitHub Actions** and enter:

   | Field | Value |
   |---|---|
   | Organization or user | `0xLizTech` |
   | Repository | `stellarforge` |
   | Workflow filename | `publish-sdk.yml` |
   | Environment name | `npm-publish` |

   npm does not validate these when you save, so a typo only shows up as a
   failed publish.

3. **Disallow token publishing.** In **Settings → Publishing access**, choose
   **Require two-factor authentication and disallow tokens**. Trusted publishing
   keeps working; a leaked token no longer can.

4. **Configure required reviewers on the GitHub environment.** In GitHub repository
   **Settings → Environments → npm-publish**:
   - Enable **Required reviewers**.
   - Add maintainers / release leads (`0xLizTech/maintainers`).
   - Every `publish-sdk.yml` execution running in this environment will pause
     until approved by an authorized reviewer.

5. **Tag the published commit** so the version traces back to its source:

   ```bash
   git tag v0.1.0 <commit> && git push origin v0.1.0
   ```

   Push the tag only. Publishing a GitHub Release starts the publish workflow,
   which would fail on a version that already exists.

## Every release

1. In `sdk/`, run `npm version <x.y.z> --no-git-tag-version`, add the release to
   `CHANGELOG.md`, and merge that to `main`.
2. Publish a GitHub Release from `main` tagged `v<x.y.z>`. The workflow refuses
   to publish if the tag and `sdk/package.json` disagree.
3. Approve the `npm-publish` deployment when the workflow run asks for it.

## Reproducing Contract WASM Byte-for-Byte

To independently verify that release WASM binaries match the audited source repository byte-for-byte across machines:

1. **Prerequisites**:
   - Ensure the exact Rust toolchain pinned in `rust-toolchain.toml` is installed (`rustup show`).
   - Install `stellar-cli` version pinned in `.github/workflows/contracts-ci.yml` (`27.1.0`).

2. **Build and Hash**:
   ```bash
   git checkout <release-tag>
   cargo clean
   stellar contract build --manifest-path contracts/rwa-asset/Cargo.toml
   stellar contract build --manifest-path contracts/registry/Cargo.toml
   stellar contract build --manifest-path contracts/compliance/Cargo.toml
   stellar contract build --manifest-path contracts/governance/Cargo.toml
   stellar contract build --manifest-path contracts/oracle-adapter/Cargo.toml

   cd target/wasm32v1-none/release
   sha256sum *.wasm
   ```

3. **Verification**:
   Compare the generated sha256 hashes against `SHA256SUMS` published with the release artifacts. The checksums must match identically across independent machines.
