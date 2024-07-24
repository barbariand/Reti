<script lang="ts">
    import KaTeX from "./KaTeX.svelte";
    import { RetiJS } from "../wasm/reti_js";

    export let reti: RetiJS;
    export let rowNumber: number;

    function parse_secure(text: string): string {
        if (text == "") {
            return "";
        }
        try {
            return reti.parse(text);
        } catch (e: unknown) {
            if (typeof e === "string") {
                return e.toString();
            } else {
                console.error(e);
                return "unknown error occurred, check console";
            }
        }
    }
    let latex = "";
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
                <KaTeX display latex={parse_secure(latex)} />
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
</style>
