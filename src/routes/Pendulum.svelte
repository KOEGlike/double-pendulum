<script lang="ts">
	import { T } from '@threlte/core';
	import { invoke, Channel } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { BufferGeometry } from 'three';

	type PendulumState = {
		angles: number[];
		positions: { x: number; y: number }[];
	};

	const channel = new Channel<PendulumState>();
	channel.onmessage = (data) => {
		pendulumState = data;
		pendulumState.positions.unshift({ x: 0, y: 0 });
		pendulumState.positions = pendulumState.positions.map(({ x, y }) => ({ x: x / 20, y: y / 20 }));
	};
	let pendulumState = $state<PendulumState | null>(null);
	$inspect(pendulumState).with(console.log);

	onMount(() => {
		invoke('pendulum_state', { channel }).catch((e) =>
			console.error('pendulum_state invoke failed:', e)
		);
	});
</script>

{#if pendulumState}
	{console.log('state in not null')}

	{#each pendulumState.positions as pos, index}
		<T.Mesh position={[pos.x, pos.y, 0]}>
			<T.SphereGeometry args={[0.2, 16, 16]} />
			<T.MeshStandardMaterial color="orange" />
		</T.Mesh>
		{#if index < pendulumState.positions.length - 1}
			<T.Line>
				<T.BufferGeometry>
					<T.BufferAttribute
						args={[
							new Float32Array([
								pos.x,
								pos.y,
								0,
								pendulumState.positions[index + 1].x,
								pendulumState.positions[index + 1].y,
								0
							]),
							3
						]}
						attach={({ parent, ref }) => {
							(parent as BufferGeometry).setAttribute('position', ref);
							return () => {
								// cleanup function called when ref changes or the component unmounts
								// https://threlte.xyz/docs/reference/core/t#attach
							};
						}}
					/>
				</T.BufferGeometry>
				<T.LineBasicMaterial color="white" />
			</T.Line>
		{/if}
	{/each}
{:else}
	<p>Loading pendulum state...</p>
{/if}
