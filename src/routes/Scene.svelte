<script lang="ts">
	import { T, useThrelte } from '@threlte/core';
	import { Align, interactivity, OrbitControls } from '@threlte/extras';
	import { AmbientLight, BufferGeometry, Color, Fog, TextureLoader } from 'three';
	import Pendulum from './Pendulum.svelte';

	useThrelte().scene.background = new Color().setHex(0x3d3d3d);
	useThrelte().scene.fog = new Fog(0x3d3d3d, 10, 50);

	const sprite = new TextureLoader().load('Ellipse 1.svg');
	//2sprite.colorSpace = SRGBColorSpace;

	const spacing = 5;
	const count = 10;

	const total = count * count * count;
	const positions = new Float32Array(total * 3);

	// center grid around origin
	const offset = (count - 1) * spacing * 0.5;

	let index = 0;
	for (let i = 0; i < count; i++) {
		for (let j = 0; j < count; j++) {
			for (let k = 0; k < count; k++) {
				positions[index * 3 + 0] = spacing * i - offset; // x
				positions[index * 3 + 1] = spacing * j - offset; // y
				positions[index * 3 + 2] = spacing * k - offset; // z
				index++;
			}
		}
	}
</script>

<T.PerspectiveCamera
	makeDefault
	fov={50}
	position={[10, 10, 10]}
	oncreate={(ref) => {
		ref.lookAt(0, 1, 0);
	}}
>
	<OrbitControls enableDamping />
</T.PerspectiveCamera>

<T.DirectionalLight position={[0, 10, 10]} castShadow />
<T.AmbientLight intensity={0.5} />

<T.Points>
	<T.BufferGeometry>
		<T.BufferAttribute
			args={[positions, 3]}
			attach={({ parent, ref }) => {
				(parent as BufferGeometry).setAttribute('position', ref);
				return () => {
					// cleanup function called when ref changes or the component unmounts
					// https://threlte.xyz/docs/reference/core/t#attach
				};
			}}
		/>
	</T.BufferGeometry>
	<T.PointsMaterial
		size={0.25}
		map={sprite}
		transparent={true}
		alphaTest={0.1}
		depthWrite={false}
		sizeAttenuation={true}
	/>
</T.Points>

<Pendulum />
