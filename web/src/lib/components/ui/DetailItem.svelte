<script lang="ts">
	import type { Snippet } from 'svelte';

	let { label, children }: { label: string; children: Snippet } = $props();
</script>

<div class="detail-item">
	<strong>{label}:</strong>
	<span>{@render children()}</span>
</div>

<style>
	.detail-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.detail-item strong {
		color: #adb5bd;
		font-size: 0.875rem;
	}
</style>
