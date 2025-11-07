<script lang="ts">
	import { T, useTask } from '@threlte/core';
	import { Align, interactivity, OrbitControls, PointsMaterial } from '@threlte/extras';
	import { Spring } from 'svelte/motion';
	import { BufferGeometry, SRGBColorSpace, TextureLoader } from 'three';

	interactivity();
	const scale = new Spring(1);

	let rotation = $state(0);
	useTask((delta) => {
		rotation += delta;
	});

	const sprite = new TextureLoader().load('disc.png');
	//2sprite.colorSpace = SRGBColorSpace;

	const spacing = 1;
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

<T.Mesh
	rotation.y={rotation}
	position.y={1}
	scale={scale.current}
	onpointerenter={() => {
		scale.target = 1.5;
	}}
	onpointerleave={() => {
		scale.target = 1;
	}}
	castShadow
>
	<T.BoxGeometry args={[1, 2, 1]} />
	<T.MeshStandardMaterial color="hotpink" />
</T.Mesh>

<T.Mesh rotation.x={-Math.PI / 2} receiveShadow>
	<T.CircleGeometry args={[4, 40]} />
	<T.MeshStandardMaterial color="white" />
</T.Mesh>

<Align>
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
		/>
	</T.Points>
</Align>
