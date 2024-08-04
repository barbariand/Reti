<script>
    import Button from "./Button.svelte";
    import CalculatorRow from "./CalculatorRow.svelte";
    import { init_wasm, RetiJS } from "reti-js";
    init_wasm();
    const reti = new RetiJS();
    let rows = 2;
</script>

<div class="calculator-view">
    <div class="rows">
        {#each { length: rows } as _, index}
            <CalculatorRow {reti} rowNumber={index + 1} />
        {/each}
        <Button primary on:click={() => rows++} style="align-self: center;">
            Add row
        </Button>
    </div>
</div>

<style>
    .calculator-view {
        display: flex;
        flex-grow: 1;
        padding: 20px 28px 20px 5px;
        max-width: 600px;
        background-color: white;
    }
    .rows {
        display: flex;
        flex-direction: column;
        gap: 10px;
        max-width: 600px;
        flex-grow: 1;
    }
</style>
