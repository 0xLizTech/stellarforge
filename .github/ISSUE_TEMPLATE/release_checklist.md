---
name: Release checklist
about: Track release evidence, signing, artifact hashes, and changelog review
title: "[release] v"
labels: ""
assignees: ""
---

## Release identity

- Version and tag (`v<x.y.z>`):
- Full source commit SHA:
- Release owner and reviewing maintainer:
- Scope (SDK, contracts, or both):
- Release notes / draft release URL:
- Target network, if a separate deployment is planned:

Use [the release guide](https://github.com/0xLizTech/stellarforge/blob/main/docs/RELEASING.md) and retain links to evidence
below. Creating this issue does not publish packages or deploy contracts. Mark
an inapplicable step with a reason; leave incomplete steps unchecked.

## Source, changelog, and checks

- [ ] The release commit is on `main` and the relevant Contracts CI and SDK CI
      runs passed at that exact SHA. Record run URLs and skipped checks below.
- [ ] [CHANGELOG.md](https://github.com/0xLizTech/stellarforge/blob/main/CHANGELOG.md) has a dated entry for this version,
      including breaking changes and migration guidance when applicable.
- [ ] For an SDK release, `sdk/package.json` and `sdk/package-lock.json` agree
      with the tag after removing its `v` prefix. The version is unused on npm.
- [ ] A maintainer reviewed the release notes, scope, and outstanding blockers.

## Artifacts and hashes

- [ ] For contracts, download `contract-wasm` from the successful Contracts CI
      run for the release SHA. Keep its `.wasm` files together with `SHA256SUMS`.
      In the extracted directory run `sha256sum --check SHA256SUMS` and record
      the output. Reject missing files or mismatches.
- [ ] Record the exact artifact inventory, originating run, and SHA-256 values
      below. Check the final files attached to the release against these hashes.
      CI artifacts expire after seven days; preserve release files before expiry.
- [ ] For an SDK release, record the published npm tarball URL and `dist.integrity`
      (for example, `npm view @stellarforge-protocol/sdk@<version> dist --json`).
      If a tarball is attached separately, record its SHA-256 as well; do not
      substitute the `sdk-dist` directory artifact for the npm package.

| Artifact / package | Source run or URL | SHA-256 or npm integrity | Verification result |
| --- | --- | --- | --- |
| | | | |

## Signing and provenance

The current workflows produce WASM checksums and use npm trusted publishing;
they do not create signed Git tags or detached signatures for WASM assets.

- [ ] Record the release tag's signature status. If signed, verify it with
      `git verify-tag v<x.y.z>` and record the signer fingerprint and result.
      If unsigned, record the maintainer's explicit decision and reason.
- [ ] For separately distributed artifacts, record the maintainer's signing
      decision. If signatures are supplied, identify the signed files, signature
      files, trusted public-key fingerprint, verification command, and result.
      Otherwise disclose that the assets have checksums but no signatures.
- [ ] For an SDK release through `publish-sdk.yml`, verify the npm provenance
      points to this repository, workflow, and source revision. Retain the npm
      package and workflow URLs. Handle the first manual publish exception
      described in the release guide explicitly; do not claim provenance for it.

Never paste signing keys, npm credentials, or account secrets in this issue.

## Maintainer publication and verification

- [ ] For an SDK release, the `npm-publish` environment and npm trusted publisher
      configuration match the release guide. Record any dry-run evidence.
- [ ] A maintainer authorized publication. Publishing a GitHub Release triggers
      the SDK publish workflow; a manual dispatch publishes only when `dry_run`
      is false. Record the environment approval and successful run URL.
- [ ] Verify the published release tag and source SHA, download and check the
      final attachments, and verify the npm version/integrity/provenance when
      applicable. Link the results below.
- [ ] Any contract deployment is tracked and authorized separately, with its
      network, contract IDs, transaction evidence, and deployed WASM hashes.
- [ ] A reviewing maintainer confirmed all applicable steps and resolved the
      blockers before closing this checklist.

## Evidence, exceptions, and blockers

<!-- Link CI runs, changelog entry, signature verification, checksum output,
     published package/release, and any separate deployment issue. Explain each
     skipped or unsigned step and identify the maintainer accepting it. -->
