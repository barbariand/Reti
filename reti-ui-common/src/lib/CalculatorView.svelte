<script>
    import CalculatorRow from "./CalculatorRow.svelte";
</script>

<div>
    <div class="rows">
        <CalculatorRow />
        <CalculatorRow />
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
