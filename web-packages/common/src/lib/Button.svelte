<script lang="ts">
    export let primary = false;
    export let secondary = false;

    export let style: string | undefined;

    $: className = (() => {
        if (primary) return "primary";
        if (secondary) return "secondary";
    })();
</script>

<button class={className} {style} on:click>
    <slot />
</button>

<style>
    button {
        padding: 8px 12px;
        border-radius: 4px;
        border: none;
        outline: none;
        color: var(--gunmetal-800);
        background-color: transparent;
        cursor: pointer;
    }
    .primary {
        background-color: var(--springgreen-300);
    }
    .secondary {
        background-color: var(--deepskyblue-300);
    }
</style>
