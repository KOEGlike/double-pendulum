<script lang="ts">
	let { children } = $props();
</script>

{@render children()}

<style>
	:global(body) {
		margin: 0;
		height: 100vh;
	}

	:global(html) {
		height: -webkit-fill-available;
	}
</style>
