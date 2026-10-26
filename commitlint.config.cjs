// Commitlint configuration enforcing the Conventional Commits style that
// CONTRIBUTING.md documents ("Commit Style" section). The CI job in
// .github/workflows/commitlint.yml runs this against every commit in a pull
// request, so a non-conforming subject line fails the check.
module.exports = {
  extends: ["@commitlint/config-conventional"],
  rules: {
    // Types, verbatim from CONTRIBUTING.md.
    "type-enum": [
      2,
      "always",
      ["feat", "fix", "docs", "test", "refactor", "chore", "perf"],
    ],
    // Scopes documented in CONTRIBUTING.md, plus the contract crates that have
    // since been added to the workspace (common, vault) and the conventional
    // "deps"/"release" scopes used by automation. A scope is optional, but when
    // present it must be one of these.
    "scope-enum": [
      2,
      "always",
      [
        "rwa-asset",
        "registry",
        "compliance",
        "governance",
        "oracle-adapter",
        "vault",
        "common",
        "sdk",
        "ci",
        "docs",
        "deps",
        "release",
      ],
    ],
    // CONTRIBUTING.md: "Keep the subject line under 72 characters."
    "header-max-length": [2, "always", 72],
  },
  // Ignore bot-generated merge commits, which do not follow the convention.
  ignores: [(message) => /^Merge /.test(message)],
};
