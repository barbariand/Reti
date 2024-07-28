<script lang="ts">
    import katex from "katex";
    import "katex/dist/katex.min.css";

    export let latex: string;
    export let display = false;

    function parse(latex: string) {
        try {
            const html = katex.renderToString(latex, {
                displayMode: display,
            });
            return html;
        } catch {
            return null;
        }
    }

    $: html = parse(latex);
</script>

<span data-latex={latex} aria-label={latex}>
    {#if html}
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        {@html html}
    {:else if latex}
        <span class="error">{latex}</span>
    {/if}
</span>

<style>
    .error {
        color: red;
    }
</style>
