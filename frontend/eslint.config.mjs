import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";

const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,
  {
    files: ["**/*.{ts,tsx}"],
    rules: {
      // Maintainability guardrails: large files/functions should be split by domain/responsibility.
      "max-lines": ["warn", { max: 800, skipBlankLines: true, skipComments: true }],
      "max-lines-per-function": [
        "warn",
        { max: 180, skipBlankLines: true, skipComments: true, IIFEs: true },
      ],
    },
  },
  // Override default ignores of eslint-config-next.
  globalIgnores([
    // Default ignores of eslint-config-next:
    ".next/**",
    "out/**",
    "build/**",
    "next-env.d.ts",
    // Generated test artifacts:
    "coverage/**",
    "playwright-report/**",
    "test-results/**",
  ]),
]);

export default eslintConfig;
