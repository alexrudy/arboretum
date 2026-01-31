<script lang="ts">
	import type { LogOrSpan } from '$lib/api';
	import { formatTimestamp, getLevelColor } from '$lib/utils';
	import { createEventDispatcher } from 'svelte';

	export let record: LogOrSpan;
	export let expanded = false;

	const dispatch = createEventDispatcher();

	function toggleExpanded() {
		expanded = !expanded;
		dispatch('toggle', { record, expanded });
	}

	$: levelColor = getLevelColor(record.level);
	$: timestamp = formatTimestamp(record.timestamp);
</script>

<div class="list-group-item list-group-item-action" on:click={toggleExpanded}>
	<div class="d-flex align-items-center gap-3">
		<div class="text-muted small" style="min-width: 180px;">
			<i class="bi bi-clock"></i>
			{timestamp}
		</div>

		{#if record.type === 'log'}
			<span class="badge bg-{levelColor}" style="min-width: 60px;">
				{record.level || 'NONE'}
			</span>
			<div
				class="text-muted small"
				style="min-width: 200px; overflow: hidden; text-overflow: ellipsis;"
			>
				<i class="bi bi-code-slash"></i>
				{record.target || 'unknown'}
			</div>
			<div class="flex-grow-1">
				{record.message || '(no message)'}
			</div>
			<i class="bi bi-file-text text-info"></i>
		{:else}
			<span class="badge bg-{levelColor}" style="min-width: 60px;">
				{record.level || 'NONE'}
			</span>
			<div
				class="text-muted small"
				style="min-width: 200px; overflow: hidden; text-overflow: ellipsis;"
			>
				<i class="bi bi-code-slash"></i>
				{record.target || 'unknown'}
			</div>
			<div class="flex-grow-1">
				<i class="bi bi-diagram-3 text-warning me-2"></i>
				<strong>{record.name}</strong>
			</div>
			<i class="bi bi-box text-info"></i>
		{/if}

		<i class="bi bi-chevron-{expanded ? 'up' : 'down'}"></i>
	</div>
</div>

<style>
	.list-group-item {
		cursor: pointer;
		transition: background-color 0.15s ease-in-out;
	}

	.badge {
		font-size: 0.75rem;
		font-weight: 600;
	}
</style>
