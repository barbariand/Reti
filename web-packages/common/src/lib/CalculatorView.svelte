<script>
    import CalculatorRow from "./CalculatorRow.svelte";
    import { init_wasm, RetiJS } from "reti-js";
    init_wasm();
    const reti = new RetiJS();
</script>

<div class="calculator-view">
    <div class="rows">
        <CalculatorRow {reti} rowNumber={1} />
        <CalculatorRow {reti} rowNumber={2} />
    </div>
</div>

<style>
    .calculator-view {
        display: flex;
        flex-grow: 1;
        padding: 20px 28px 0px 5px;
    }
    .rows {
        display: flex;
        flex-direction: column;
        gap: 10px;
        max-width: 600px;
        flex-grow: 1;
    }
</style>
