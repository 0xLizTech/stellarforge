// ESLint flat config for the SDK. `npm run lint` and the pre-commit hook
// (scripts/pre-commit-eslint.sh) both read this file.
import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default [
  { ignores: ["dist/**", "node_modules/**", "coverage/**"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.ts"],
    rules: {
      // A leading underscore marks a parameter or binding as intentionally unused.
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_", caughtErrors: "none" },
      ],
    },
  },
];
