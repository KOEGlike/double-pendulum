<script lang="ts">
	let { children } = $props();
</script>

{@render children()}

<style>
	:global(body) {
		margin: 0;
	}

	:global(html) {
		height: -webkit-fill-available;
	}
</style>
