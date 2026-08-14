<!--
Thanks for contributing to StellarForge! Please fill this out and check the boxes.
Read CONTRIBUTING.md first if you haven't already.
-->

## What changed

<!-- Describe the change and why. -->

## Why

<!-- The motivation. Link the issue this closes, e.g. Closes #123. -->

## How to test

<!-- Steps for a reviewer to reproduce and verify. -->

## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --all --features testutils` passes
- [ ] (SDK) `npm run typecheck` and `npm test` pass
- [ ] New public functions have both a positive and a negative test
- [ ] Docs updated (README / ADR / CHANGELOG) where relevant

## Type

- [ ] feat
- [ ] fix
- [ ] docs
- [ ] test
- [ ] refactor
- [ ] chore
