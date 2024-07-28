import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import eslintPluginSvelte from "eslint-plugin-svelte";
import globals from "globals";

export default tseslint.config(
    eslint.configs.recommended,
    ...tseslint.configs.recommended,
    ...eslintPluginSvelte.configs["flat/recommended"],
    ...eslintPluginSvelte.configs["flat/prettier"],
    {
        languageOptions: {
            parserOptions: {
                parser: tseslint.parser,
            },
            globals: {
                ...globals.browser,
            },
        },
        ignores: ["**/dist/**"],
        rules: {
            // override/add rules settings here, such as:
            // 'svelte/rule-name': 'error'
        },
    },
);
