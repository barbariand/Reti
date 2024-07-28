/**
 * @see https://prettier.io/docs/en/configuration.html
 * @type {import("prettier").Config}
 */
export default {
    trailingComma: "all",
    semi: true,
    singleQuote: false,
    printWidth: 80,
    tabWidth: 4,
    svelteIndentScriptAndStyle: true,
    plugins: ["prettier-plugin-svelte"],
};
