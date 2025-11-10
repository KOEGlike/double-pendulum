<script lang="ts">
	import { Channel, invoke } from '@tauri-apps/api/core';
	import type { PendulumState } from './pendulum_state';
	import { onMount } from 'svelte';

	const channel = new Channel<PendulumState>();
	channel.onmessage = (data) => {
		pendulumState = data;
		// keep modify forms array in sync with number of bobs
		ensureModifyFormsLength();
	};
	let pendulumState = $state<PendulumState | null>(null);
	//$inspect(pendulumState).with(console.log);

	// Error and info messages
	let message = $state<string | null>(null);
	function setMessage(m: string | null) {
		message = m;
		if (m) setTimeout(() => (message = null), 2500);
	}

	onMount(() => {
		invoke('pendulum_state', { channel }).catch((e) =>
			console.error('pendulum_state invoke failed:', e)
		);
	});

	// Form state for adding a new bob
	let newBob = $state({ lengthRod: 120, mass: 10, theta: Math.PI / 10, omega: 0 });

	async function addBob(lengthRod: number, mass: number, theta: number, omega: number) {
		// Rust expects snake_case parameter names
		await invoke('add_bob', { lengthRod, mass, theta, omega })
			.then(() => setMessage('Bob added'))
			.catch((e) => setMessage(`Add failed: ${String(e)}`));
	}
	async function removeBob(index: number) {
		await invoke('remove_bob', { index })
			.then(() => setMessage('Bob removed'))
			.catch((e) => setMessage(`Remove failed: ${String(e)}`));
	}

	// Per-row modify forms: store optional strings so empty => no change
	let modifyForms = $state<
		Array<{ lengthRod?: string; mass?: string; theta?: string; omega?: string }>
	>([]);
	function ensureModifyFormsLength() {
		const n = pendulumState?.bobs.length ?? 0;
		while (modifyForms.length < n)
			modifyForms.push({
				lengthRod: undefined,
				mass: undefined,
				theta: undefined,
				omega: undefined
			});
		while (modifyForms.length > n) modifyForms.pop();
	}

	function parseOrNull(v?: string): number | null {
		if (v === null || v === undefined) return null;
		const t = v;
		if (!t) return null;
		const num = Number(t);
		return Number.isFinite(num) ? num : null;
	}

	async function modifyBob(index: number) {
		const form = modifyForms[index];
		if (!form) return;
		const length = parseOrNull(form.lengthRod);
		const mass = parseOrNull(form.mass);
		const theta = parseOrNull(form.theta);
		const omega = parseOrNull(form.omega);

		console.log({ length_rod: length, mass, theta, omega });

		// Build args, passing null for unchanged so Rust receives None
		await invoke('modify_bob', {
			index,
			length,
			mass,
			theta,
			omega
		})
			.then(() => {
				setMessage('Bob updated');
				// clear only the fields that were sent
				form.lengthRod = undefined;
				form.mass = undefined;
				form.theta = undefined;
				form.omega = undefined;
			})
			.catch((e) => setMessage(`Modify failed: ${String(e)}`));
	}
</script>

<div class="panel">
	{#if message}
		<div class="msg">{message}</div>
	{/if}

	<div class="section">
		<h3>Add bob</h3>
		<div class="row">
			<input type="number" step="1" bind:value={newBob.lengthRod} placeholder="length (px)" />
			<input type="number" step="0.1" bind:value={newBob.mass} placeholder="mass (kg)" />
			<input type="number" step="0.01" bind:value={newBob.theta} placeholder="theta (rad)" />
			<input type="number" step="0.01" bind:value={newBob.omega} placeholder="omega (rad/s)" />
			<button
				class="primary"
				onclick={() => addBob(newBob.lengthRod, newBob.mass, newBob.theta, newBob.omega)}
				>Add</button
			>
		</div>
		<div class="small muted">Tip: theta is in radians. Use ~0.314 for 18°.</div>
	</div>

	<div class="section">
		<h3>Modify bobs</h3>
		{#if !pendulumState}
			<div class="muted">Waiting for state…</div>
		{:else if pendulumState.bobs.length === 0}
			<div class="muted">No bobs yet. Add one above.</div>
		{:else}
			<div class="row header">
				<div>length (px)</div>
				<div>mass (kg)</div>
				<div>theta (rad)</div>
				<div>omega (rad/s)</div>
				<div>Actions</div>
			</div>
			{#each pendulumState.bobs as bob, i}
				<div class="row">
					<input
						type="number"
						step="1"
						placeholder={String(bob.lengthRod)}
						bind:value={modifyForms[i].lengthRod}
					/>
					<input
						type="number"
						step="0.1"
						placeholder={String(bob.mass)}
						bind:value={modifyForms[i].mass}
					/>
					<input
						type="number"
						step="0.01"
						placeholder={String(bob.theta)}
						bind:value={modifyForms[i].theta}
					/>
					<input
						type="number"
						step="0.01"
						placeholder={String(bob.omega)}
						bind:value={modifyForms[i].omega}
					/>
					<div style="display:flex; gap:0.5rem;">
						<button onclick={() => modifyBob(i)}>Save</button>
						<button class="danger" onclick={() => removeBob(i)}>Remove</button>
					</div>
				</div>
			{/each}
		{/if}
	</div>
</div>

<style>
	.panel {
		max-width: 700px;
		margin: 0 auto;
		padding: 1rem;
		display: grid;
		gap: 1rem;
		position: absolute;
		inset-inline-start: 10px;
		inset-block-start: 10px;
		color: #f2c16d;
	}
	.row {
		display: grid;
		grid-template-columns: 1fr 1fr 1fr 1fr auto;
		gap: 2rem;
		align-items: center;
	}
	.row.header {
		font-weight: 600;
		opacity: 0.8;
	}
	input[type='number'] {
		color: rgb(120, 167, 147);
		width: 100%;
		padding: 0.4rem 0.5rem;
		border: 1px solid #555555;
		border-radius: 6px;
		background-color: #363636;
	}
	button {
		padding: 0.45rem 0.75rem;
		border: 1px solid #999;
		background: #f6f6f6;
		border-radius: 6px;
		cursor: pointer;
	}
	button.primary {
		background: #2d8;
		border-color: #2d8;
		color: #053;
	}
	button.danger {
		background: #f66;
		border-color: #f33;
		color: #300;
	}
	.msg {
		text-align: center;
		color: #055;
	}
	.muted {
		color: #666;
		font-size: 0.9rem;
	}
	.section {
		border: 1px solid #121212;
		border-radius: 10px;
		padding: 1rem;
		background-color: #212120;
	}
	.small {
		font-size: 0.85rem;
	}
</style>
