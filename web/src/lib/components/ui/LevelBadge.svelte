<script lang="ts">
	import type { Level } from '$lib/api';
	import { getLevelColor } from '$lib/utils';

	let { level, minWidth = '60px' }: { level: Level | null; minWidth?: string } = $props();

	let levelColor = $derived(getLevelColor(level));
</script>

<span class="badge bg-{levelColor}" style="min-width: {minWidth};">
	{level || 'NONE'}
</span>

<style>
	.badge {
		font-size: 0.75rem;
		font-weight: 600;
	}
</style>
