<script lang="ts">
	import { T } from '@threlte/core';
	import { invoke, Channel } from '@tauri-apps/api/core';
</script>

<T.Mesh position={[0, -2, 0]}>
	<T.SphereGeometry args={[0.4, 32, 16]} />
	<T.MeshStandardMaterial color="brown" />
</T.Mesh>
