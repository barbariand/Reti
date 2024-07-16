<script lang="ts">
    let count: number = 0;

    const increment = () => {
        count += 1;
    };

    console.log("hello");
</script>

<button on:click={increment}>
    count is {count}
</button>

<style>
    button {
        color: red;
    }
</style>
