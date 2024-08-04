<script lang="ts">
    import type { ComponentType } from "svelte";
    import KaTeX from "./lib/KaTeX.svelte";
    import CalculatorView from "./lib/CalculatorView.svelte";
    import "./colors.css";

    const components = { KaTeX, CalculatorView };
    let component: ComponentType | null = null;
</script>

<div>
    <header>
        <h1>reti-common-ui component viewer</h1>
        <select bind:value={component}>
            <option value={null}>Select a component</option>
            {#each Object.entries(components) as [name, option]}
                <option value={option}>{name}</option>
            {/each}
        </select>
    </header>
    <div class="content">
        {#if component == null}
            <p>Select a component.</p>
        {:else if component == KaTeX}
            <KaTeX display latex={"\\int_1^2f(x)\\,\\mathrm{d}x"} />
        {:else}
            <svelte:component this={component} />
        {/if}
    </div>
</div>

<style>
    header {
        height: 100px;
    }
    .content {
        display: flex;
        justify-content: center;
        align-items: center;
        min-height: calc(100vh - 100px);
    }
</style>
