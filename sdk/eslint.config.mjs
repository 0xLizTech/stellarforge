// Flat ESLint config for the StellarForge SDK.
//
// CONTRIBUTING.md requires strict TypeScript with no `any` types and JSDoc on
// every exported symbol. The rules below encode the parts of that policy ESLint
// can check mechanically; `npm run typecheck` covers the type system itself.
import js from "@eslint/js";
import jsdoc from "eslint-plugin-jsdoc";
import tseslint from "typescript-eslint";

const stellarforgeDocs = {
  rules: {
    "require-exported-value-docs": {
      meta: {
        type: "problem",
        docs: { description: "require TSDoc comments on exported values" },
        schema: [],
        messages: { missing: "Exported value '{{name}}' needs a TSDoc comment." },
      },
      create(context) {
        const sourceCode = context.sourceCode;
        return {
          ExportNamedDeclaration(node) {
            if (node.declaration?.type !== "VariableDeclaration") return;
            const declaration = node.declaration;
            const hasDoc = sourceCode.getCommentsBefore(node).some((comment) =>
              sourceCode.text.slice(comment.range[0], comment.range[0] + 3) === "/**" &&
              sourceCode.text.slice(comment.range[1], node.range[0]).trim() === "",
            );
            if (hasDoc) return;
            for (const value of declaration.declarations) {
              if (value.id.type === "Identifier") {
                context.report({ node: value, messageId: "missing", data: { name: value.id.name } });
              }
            }
          },
        };
      },
    },
  },
};

export default tseslint.config(
  {
    // Build output and vendored dependencies are never linted.
    ignores: ["dist/**", "node_modules/**"],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    plugins: { jsdoc, stellarforge: stellarforgeDocs },
    rules: {
      "jsdoc/require-jsdoc": "error",
      "stellarforge/require-exported-value-docs": "error",
    },
  },
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
      // Public SDK API documentation is part of the published contract.
      "jsdoc/require-jsdoc": [
        "error",
        {
          publicOnly: true,
          require: {
            ClassDeclaration: true,
            FunctionDeclaration: true,
            MethodDefinition: true,
          },
          contexts: [
            "TSInterfaceDeclaration",
            "TSTypeAliasDeclaration",
            "TSEnumDeclaration",
          ],
        },
      ],
      // Descriptions, @param, @returns and @throws are part of the TSDoc
      // convention; the required-comment rule is the blocking completeness gate.
    },
  },
);
