<script lang="ts">
    import KaTeX from "./KaTeX.svelte";
    import { RetiJS } from "reti-js";

    type RetiResult = {
        LaTeX?: string;
        Error?: string;
        // TODO better types for AddedFunction and such.
    };

    export let reti: RetiJS;
    export let rowNumber: number;
    let latex = "";
    $: result = parse(latex);

    function parse(latex: string): RetiResult | null {
        if (latex == "") {
            return null;
        }
        try {
            return reti.parse(latex) as RetiResult;
        } catch (e: unknown) {
            if (typeof e === "string") {
                return { Error: e };
            } else {
                console.error(e);
                return { Error: "Unknown error occurred, check console" };
            }
        }
    }
</script>

<div class="calculator-row">
    <div class="number">({rowNumber})</div>
    <div class="calculator-row-main">
        <div class="input-container">
            <textarea class="input" bind:value={latex} rows="1" />
        </div>
        <div class="math-container">
            <div class="math-input">
                <KaTeX display {latex} />
            </div>
            <div class="math-output">
                {#if result?.LaTeX}
                    <KaTeX display latex={result.LaTeX} />
                {:else if result?.Error}
                    <span class="error">{result.Error}</span>
                {/if}
            </div>
        </div>
    </div>
</div>

<style>
    .calculator-row {
        display: flex;
        gap: 10px;
    }
    .calculator-row-main {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }
    .input-container {
        display: flex;
        justify-content: stretch;
    }
    .input {
        padding: 8px;
        border-radius: 5px;
        border: 1px solid #7dedc4;
        font-family: monospace;
        line-height: 15px;
        font-size: 14px;
        flex-grow: 1;
        resize: vertical;
    }
    .math-container {
        display: flex;
        flex-wrap: wrap;
        justify-content: space-between;
        align-items: center;
    }
    .math-output {
        justify-self: end, flex-end;
    }
    .error {
        color: red;
    }
</style>
