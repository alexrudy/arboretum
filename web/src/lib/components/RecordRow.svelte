<script lang="ts">
	import type { LogOrSpan } from '$lib/api';
	import { formatTimestamp, formatTimestampFull, getLevelColor } from '$lib/utils';

	let {
		record,
		expanded = $bindable(false),
		onclick
	}: { record: LogOrSpan; expanded?: boolean; onclick?: () => void } = $props();

	let levelColor = $derived(getLevelColor(record.level));
	let timestamp = $derived(formatTimestamp(record.timestamp));
	let fullTimestamp = $derived(formatTimestampFull(record.timestamp));
</script>

<div class="list-group-item list-group-item-action" {onclick}>
	<div class="d-flex align-items-center gap-3">
		<div class="text-muted small" style="min-width: 230px;" title={fullTimestamp}>
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
		{:else if record.type === 'span'}
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
		{:else if record.type === 'event'}
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
				<i class="bi bi-lightning-fill me-2" style="color: var(--brand-cyan);"></i>
				<strong>{record.name}</strong>
			</div>
			<i class="bi bi-activity text-info"></i>
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
