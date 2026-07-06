import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import reactCompiler from "eslint-plugin-react-compiler";
import tseslint from "typescript-eslint";
import { globalIgnores } from "eslint/config";
import relay from "eslint-plugin-relay";
import betterTailwindcss from "eslint-plugin-better-tailwindcss";

export default tseslint.config([
  globalIgnores(["dist"]),
  {
    files: ["**/*.{ts,tsx}"],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs.flat["recommended-latest"],
      reactRefresh.configs.vite,
      reactCompiler.configs.recommended,
    ],
    languageOptions: {
      ecmaVersion: 2022,
      globals: globals.browser,
    },
  },
  {
    plugins: { relay },
    rules: relay.configs["ts-recommended"].rules,
  },
  {
    // Mirror the Tailwind CSS IntelliSense extension's diagnostics headlessly so
    // they fail CI (class conflicts + unknown/typo'd classes). The entry point
    // teaches the plugin about the custom @theme colors defined in app.css.
    files: ["**/*.{ts,tsx}"],
    plugins: { "better-tailwindcss": betterTailwindcss },
    rules: betterTailwindcss.configs["correctness-error"].rules,
    settings: {
      "better-tailwindcss": {
        entryPoint: "src/app.css",
      },
    },
  },
]);
