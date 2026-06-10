<script lang="ts">
  let { children } = $props();
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

{@render children()}
