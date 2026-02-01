<script lang="ts">
	import type { Snippet } from 'svelte';

	let { label, className, children }: { label: string; className?: string; children: Snippet } =
		$props();
</script>

<div class={`detail-item ${className ?? ''}`}>
	<strong class="text-info me-2">{label}:</strong>
	{@render children()}
</div>
