<script lang="ts">
	import type { SpanRecord, EventRecord } from '$lib/api';
	import type { TreeNode } from '$lib/types';
	import SpanTreeNode from './SpanTreeNode.svelte';

	let {
		spans,
		events,
		currentSpanId
	}: { spans: SpanRecord[]; events: EventRecord[]; currentSpanId: string } = $props();

	// Calculate timeline bounds for the entire trace
	function calculateTimelineBounds(
		spans: SpanRecord[],
		events: EventRecord[]
	): { start: number; end: number; duration: number } {
		if (spans.length === 0) {
			return { start: 0, end: 0, duration: 0 };
		}

		let minTime = spans[0].start_time;
		let maxTime = spans[0].end_time ?? spans[0].start_time;

		for (const span of spans) {
			minTime = Math.min(minTime, span.start_time);
			maxTime = Math.max(maxTime, span.end_time ?? span.start_time);
		}

		for (const event of events) {
			minTime = Math.min(minTime, event.timestamp);
			maxTime = Math.max(maxTime, event.timestamp);
		}

		return {
			start: minTime,
			end: maxTime,
			duration: maxTime - minTime
		};
	}

	function buildTree(spans: SpanRecord[], events: EventRecord[]): TreeNode[] {
		const spanMap = new Map<string, TreeNode>();
		const roots: TreeNode[] = [];

		// Create nodes with empty event arrays
		for (const span of spans) {
			spanMap.set(span.span_id, { span, children: [], events: [] });
		}

		// Attach events to their parent spans
		for (const event of events) {
			const parent = spanMap.get(event.span_id);
			if (parent) {
				parent.events.push(event);
			}
		}

		// Sort events by timestamp for each span
		for (const node of spanMap.values()) {
			node.events.sort((a, b) => a.timestamp - b.timestamp);
		}

		// Build tree structure
		for (const span of spans) {
			const node = spanMap.get(span.span_id)!;
			if (span.parent_span_id) {
				const parent = spanMap.get(span.parent_span_id);
				if (parent) {
					parent.children.push(node);
				} else {
					roots.push(node);
				}
			} else {
				roots.push(node);
			}
		}

		return roots;
	}

	let tree = $derived(buildTree(spans, events));
	let timeline = $derived(calculateTimelineBounds(spans, events));
</script>

<div class="span-tree">
	{#each tree as node}
		<SpanTreeNode {node} {currentSpanId} {timeline} depth={0} />
	{/each}
</div>

<style>
	.span-tree {
		font-family: 'Courier New', Courier, monospace;
		font-size: 0.875rem;
	}
</style>
