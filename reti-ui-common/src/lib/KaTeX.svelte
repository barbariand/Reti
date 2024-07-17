<script lang="ts">
    import katex from "katex";

    export let latex: string;
    export let display = false;

    $: html = katex.renderToString(latex, {
        displayMode: display,
    });
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -->
<span data-latex={latex} aria-label={latex}>{@html html}</span>
