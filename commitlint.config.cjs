// Enforces the commit style documented in CONTRIBUTING.md ("Commit Style").
// Self-contained on purpose: no `extends`, so it resolves the same way from the
// repo root, from sdk/ (where the CLI is installed) and inside CI.
//
// When you change the type or scope lists, update CONTRIBUTING.md too; a test
// in sdk/tests/tooling/commitlint.test.ts fails if the two drift apart.

const types = ["feat", "fix", "docs", "test", "refactor", "chore", "perf"];

const scopes = [
  // Documented in CONTRIBUTING.md
  "rwa-asset",
  "registry",
  "compliance",
  "governance",
  "oracle-adapter",
  "sdk",
  "ci",
  "docs",
  // Parts of the repo that are not one of the contracts above
  "common",
  "benchmark",
  "readme",
];

module.exports = {
  // `type(scope)!: subject`, the `!` marks a breaking change.
  parserPreset: {
    parserOpts: {
      headerPattern: /^(\w*)(?:\((.*)\))?!?: (.*)$/,
      headerCorrespondence: ["type", "scope", "subject"],
    },
  },
  rules: {
    "type-enum": [2, "always", types],
    "type-case": [2, "always", "lower-case"],
    "type-empty": [2, "never"],
    "scope-enum": [2, "always", scopes],
    "scope-case": [2, "always", "kebab-case"],
    "subject-empty": [2, "never"],
    "subject-full-stop": [2, "never", "."],
    "header-max-length": [2, "always", 72],
    "body-leading-blank": [2, "always"],
    "footer-leading-blank": [2, "always"],
  },
};
