<script lang="ts">
	import type { LogOrSpan } from '$lib/api';
	import type { TreeNode } from '$lib/types';
	import Self from './SpanTreeNode.svelte';
	import { formatDuration, getLevelColor } from '$lib/utils';

	let {
		node,
		currentSpanId,
		timeline,
		depth,
		handleRecordClick
	}: {
		node: TreeNode;
		currentSpanId: string;
		timeline: { start: number; end: number; duration: number };
		depth: number;
		handleRecordClick: (span: LogOrSpan) => void;
	} = $props();

	let duration = $derived(formatDuration(node.span.start_time, node.span.end_time));
	let levelColor = $derived(getLevelColor(node.span.level));
	let isCurrentSpan = $derived(node.span.span_id === currentSpanId);
	let indent = $derived(depth * 20);
	// Calculate timeline bar position and width
	let timelineBar = $derived.by(() => {
		if (timeline.duration === 0) {
			return { left: 0, width: 100 };
		}

		const spanStart = node.span.start_time;
		const spanEnd = node.span.end_time ?? spanStart;
		const spanDuration = spanEnd - spanStart;

		const leftPercent = ((spanStart - timeline.start) / timeline.duration) * 100;
		const widthPercent = (spanDuration / timeline.duration) * 100;

		return {
			left: leftPercent,
			width: Math.max(widthPercent, 0.5) // Minimum width for visibility
		};
	});

	// Calculate event positions
	let eventPositions = $derived.by(() => {
		if (timeline.duration === 0) {
			return node.events.map(() => 50);
		}

		return node.events.map((event) => {
			return ((event.timestamp - timeline.start) / timeline.duration) * 100;
		});
	});
</script>

<div class="tree-node" style="margin-left: {indent}px">
	<div class="span-item {isCurrentSpan ? 'current-span' : ''}">
		<div class="d-flex align-items-center gap-2 py-1">
			{#if depth > 0}
				<span class="tree-line text-muted">└─</span>
			{/if}
			<button type="button" class="span-link" onclick={() => handleRecordClick(node.span)}>
				<i class="bi bi-box-fill text-warning"></i>
				<span class="badge bg-{levelColor}">{node.span.level || 'NONE'}</span>
				<strong>{node.span.name}</strong>
				<span class="text-muted small">({duration})</span>
				{#if isCurrentSpan}
					<i class="bi bi-arrow-left text-info"></i>
				{/if}
			</button>
		</div>

		<!-- Timeline visualization -->
		<div class="timeline-container">
			<div class="timeline-background"></div>
			<div
				class="timeline-bar"
				style="left: {timelineBar.left}%; width: {timelineBar.width}%;"
				title={duration}
			></div>
			{#each eventPositions as eventPos, i}
				<div class="timeline-event" style="left: {eventPos}%;" title={node.events[i].name}></div>
			{/each}
		</div>
	</div>

	<!-- Render events for this span -->
	{#each node.events as event}
		{@const eventLevelColor = getLevelColor(event.level)}
		<div
			class="event-item d-flex align-items-center gap-2 py-1"
			style="margin-left: {(depth + 1) * 20}px"
		>
			<span class="tree-line text-muted">└─</span>
			<i class="bi bi-lightning-fill" style="color: var(--brand-cyan); font-size: 0.8rem;"></i>
			<span class="badge bg-{eventLevelColor}">{event.level || 'NONE'}</span>
			<span class="text-muted small">{event.name}</span>
		</div>
	{/each}

	{#each node.children as child}
		<Self node={child} {currentSpanId} {timeline} {handleRecordClick} depth={depth + 1} />
	{/each}
</div>

<style>
	.span-item {
		padding: 2px 4px;
		border-radius: 3px;
	}

	.span-link {
		border: none;
		background-color: transparent;
		color: inherit;
		text-decoration: none;
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

	.timeline-container {
		position: relative;
		height: 8px;
		margin: 2px 0 4px 0;
		width: 100%;
	}

	.timeline-background {
		position: absolute;
		top: 3px;
		left: 0;
		right: 0;
		height: 2px;
		background-color: rgba(108, 117, 125, 0.2);
		border-radius: 1px;
	}

	.timeline-bar {
		position: absolute;
		top: 2px;
		height: 4px;
		background-color: var(--bs-warning);
		border-radius: 2px;
		opacity: 0.8;
		transition: opacity 0.2s;
	}

	.timeline-bar:hover {
		opacity: 1;
	}

	.timeline-event {
		position: absolute;
		top: 0;
		width: 8px;
		height: 8px;
		background-color: var(--brand-cyan);
		border-radius: 50%;
		transform: translateX(-50%);
		border: 1px solid rgba(0, 0, 0, 0.2);
		cursor: help;
		transition: transform 0.2s;
	}

	.timeline-event:hover {
		transform: translateX(-50%) scale(1.3);
	}
</style>
