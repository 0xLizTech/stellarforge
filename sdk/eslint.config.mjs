// Flat ESLint config for the StellarForge SDK.
//
// CONTRIBUTING.md requires strict TypeScript with no `any` types and JSDoc on
// every exported symbol. The rules below encode the parts of that policy ESLint
// can check mechanically; `npm run typecheck` covers the type system itself.
import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    // Build output and vendored dependencies are never linted.
    ignores: ["dist/**", "node_modules/**"],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.ts"],
    rules: {
      // CONTRIBUTING.md: "Strict TypeScript - no `any` types."
      "@typescript-eslint/no-explicit-any": "error",
      // Unused code is a review smell; allow a leading underscore as the
      // conventional "intentionally unused" marker for arguments.
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
    },
  },
);
