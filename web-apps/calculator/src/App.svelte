<script lang="ts">
    import CalculatorView from "reti-ui-common/src/lib/CalculatorView.svelte";
</script>

<main>
    <CalculatorView />
</main>

<style>
    :global(body) {
        background-color: #eaeef0;
        padding: 0;
    }
    main {
        max-width: 600px;
        display: flex;
        justify-content: center;
        margin: auto;
        background-color: white;
        border-radius: 5px;
    }
</style>
