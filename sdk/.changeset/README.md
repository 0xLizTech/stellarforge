# Changesets

Run `npm run changeset` from `sdk/` to record a user-facing SDK change. Select
`@stellarforge-protocol/sdk` and the appropriate semver bump, then summarize
the change for the generated changelog. Commit the new Markdown file with the
change it describes.

When a changeset reaches `main`, the Changesets workflow opens or updates a
Version Packages pull request. Review and merge that PR to update
`sdk/package.json` and `sdk/CHANGELOG.md`. Publishing remains a separate
release action; see [the release guide](../../docs/RELEASING.md).
