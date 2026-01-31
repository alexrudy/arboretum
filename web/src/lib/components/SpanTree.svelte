<script lang="ts">
	import type { SpanRecord } from '$lib/api';
	import SpanTreeNode from './SpanTreeNode.svelte';

	export let spans: SpanRecord[];
	export let currentSpanId: string;

	interface TreeNode {
		span: SpanRecord;
		children: TreeNode[];
	}

	function buildTree(spans: SpanRecord[]): TreeNode[] {
		const spanMap = new Map<string, TreeNode>();
		const roots: TreeNode[] = [];

		// Create nodes
		for (const span of spans) {
			spanMap.set(span.span_id, { span, children: [] });
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

	$: tree = buildTree(spans);
</script>

<div class="span-tree">
	{#each tree as node}
		<SpanTreeNode {node} {currentSpanId} depth={0} />
	{/each}
</div>

<style>
	.span-tree {
		font-family: 'Courier New', Courier, monospace;
		font-size: 0.875rem;
	}
</style>
