<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Canvas } from '@threlte/core';
	import Scene from './Scene.svelte';
	import { AsciiRenderer } from '@threlte/extras';
	import Control from './Control.svelte';

	let name = $state('');
	let greetMsg = $state('');

	async function greet(event: Event) {
		event.preventDefault();
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		greetMsg = await invoke('greet', { name });
	}
</script>

<div>
	<Canvas>
		<Scene />
	</Canvas>
	<Control />
</div>

<style>
	div {
		height: 100%;
	}
</style>
