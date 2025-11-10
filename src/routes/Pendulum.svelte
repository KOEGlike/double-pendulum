<script lang="ts">
	import { T } from '@threlte/core';
	import { invoke, Channel } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { BufferGeometry } from 'three';
	import type { PendulumState } from './pendulum_state';

	const channel = new Channel<PendulumState>();
	channel.onmessage = (data) => {
		pendulumState = data;

		pendulumState.bobs = pendulumState.bobs.map(({ position, ...rest }) => ({
			position: { x: position.x / 100, y: position.y / 100 },
			...rest
		}));
	};
	let pendulumState = $state<PendulumState | null>(null);
	//$inspect(pendulumState).with(console.log);

	onMount(() => {
		invoke('pendulum_state', { channel }).catch((e) =>
			console.error('pendulum_state invoke failed:', e)
		);
	});
</script>

{#if pendulumState}
	{console.log('state in not null')}

	<!-- origin -->
	<T.Mesh position={[0, 0, 0]}>
		<T.SphereGeometry args={[0.2, 16, 16]} />
		<T.MeshStandardMaterial color="orange" />
	</T.Mesh>

	{#each pendulumState.bobs as bob, index}
		<!-- bob -->
		<T.Mesh position={[bob.position.x, bob.position.y, 0]}>
			<T.SphereGeometry args={[0.15 * Math.cbrt((3 * bob.mass) / (4 * Math.PI)), 16, 16]} />
			<T.MeshStandardMaterial color="orange" />
		</T.Mesh>

		{#if index === 0}
			<!-- line from origin to first bob -->
			<T.Line>
				<T.BufferGeometry>
					<T.BufferAttribute
						args={[new Float32Array([0, 0, 0, bob.position.x, bob.position.y, 0]), 3]}
						attach={({ parent, ref }) => {
							(parent as BufferGeometry).setAttribute('position', ref);
							return () => {};
						}}
					/>
				</T.BufferGeometry>
				<T.LineBasicMaterial color="white" />
			</T.Line>
		{:else}
			<!-- line from previous bob to this bob -->
			<T.Line>
				<T.BufferGeometry>
					<T.BufferAttribute
						args={[
							new Float32Array([
								pendulumState.bobs[index - 1].position.x,
								pendulumState.bobs[index - 1].position.y,
								0,
								bob.position.x,
								bob.position.y,
								0
							]),
							3
						]}
						attach={({ parent, ref }) => {
							(parent as BufferGeometry).setAttribute('position', ref);
							return () => {};
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
