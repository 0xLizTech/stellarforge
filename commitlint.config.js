module.exports = {
  extends: ["@commitlint/config-conventional"],
  rules: {
    "type-enum": [
      2,
      "always",
      ["feat", "fix", "docs", "test", "refactor", "chore", "perf"],
    ],
    "scope-enum": [
      2,
      "always",
      [
        "rwa-asset",
        "registry",
        "compliance",
        "governance",
        "oracle-adapter",
        "sdk",
        "ci",
        "docs",
        "readme",
      ],
    ],
    "header-max-length": [2, "always", 72],
  },
};
