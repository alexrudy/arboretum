<script lang="ts">
	import type { SpanRecord } from '$lib/api';
	import { formatDuration, getLevelColor } from '$lib/utils';

	interface TreeNode {
		span: SpanRecord;
		children: TreeNode[];
	}

	export let node: TreeNode;
	export let currentSpanId: string;
	export let depth: number;

	$: duration = formatDuration(node.span.start_time, node.span.end_time);
	$: levelColor = getLevelColor(node.span.level);
	$: isCurrentSpan = node.span.span_id === currentSpanId;
	$: indent = depth * 20;
</script>

<div class="tree-node" style="margin-left: {indent}px">
	<div
		class="span-item d-flex align-items-center gap-2 py-1 {isCurrentSpan ? 'current-span' : ''}"
	>
		{#if depth > 0}
			<span class="tree-line text-muted">└─</span>
		{/if}
		<i class="bi bi-box-fill text-warning"></i>
		<span class="badge bg-{levelColor}">{node.span.level || 'NONE'}</span>
		<strong>{node.span.name}</strong>
		<span class="text-muted small">({duration})</span>
		{#if isCurrentSpan}
			<i class="bi bi-arrow-left text-info"></i>
		{/if}
	</div>

	{#each node.children as child}
		<svelte:self node={child} {currentSpanId} depth={depth + 1} />
	{/each}
</div>

<style>
	.span-item {
		padding: 2px 4px;
		border-radius: 3px;
	}

	.current-span {
		background-color: rgba(13, 202, 240, 0.1);
		border-left: 3px solid var(--brand-cyan);
		padding-left: 8px;
	}

	.tree-line {
		font-family: monospace;
	}

	.badge {
		font-size: 0.7rem;
	}
</style>
