<script lang="ts">
    import CalculatorRow from "./CalculatorRow.svelte";
    import { start, RetiJS } from "reti-js";
    start();
    const reti = new RetiJS();
    let rows = [{ id: 1 }];
    function addRow() {
        console.log("adding row");
        let next_id = rows[rows.length - 1].id + 1;
        rows = [...rows, { id: next_id }];
    }
</script>

<div class="calculator-view">
    <div class="rows">
        {#each rows as row}
            <CalculatorRow {reti} rowNumber={row.id} on_first_input={addRow} />
        {/each}
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
