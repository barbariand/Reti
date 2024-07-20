<script>
    import CalculatorRow from "./CalculatorRow.svelte";
    import { init_wasm, RetiJS } from "../wasm/reti_js.js";
    init_wasm();
    const reti = new RetiJS();
</script>

<div>
    <div class="rows">
        <CalculatorRow {reti} />
        <CalculatorRow {reti} />
    </div>
</div>

<style>
    .rows {
        display: flex;
        flex-direction: column;
        gap: 10px;
        max-width: 600px;
    }
</style>
