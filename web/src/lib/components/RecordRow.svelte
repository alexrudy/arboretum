<script lang="ts">
	import type { LogOrSpan } from '$lib/api';
	import LevelBadge from './ui/LevelBadge.svelte';
	import TargetLabel from './ui/TargetLabel.svelte';
	import TimestampLabel from './ui/TimestampLabel.svelte';

	let {
		record,
		expanded = $bindable(false),
		onclick
	}: { record: LogOrSpan; expanded?: boolean; onclick?: () => void } = $props();
</script>

<div class="list-group-item list-group-item-action">
	<button type="button" class="list-group-item-button d-flex align-items-center gap-3" {onclick}>
		<TimestampLabel timestamp={record.timestamp} />

		{#if record.type === 'log'}
			<LevelBadge level={record.level} />
			<TargetLabel target={record.target} />
			<div class="flex-grow-1">
				{record.message || '(no message)'}
			</div>
			<i class="bi bi-file-text text-info"></i>
		{:else if record.type === 'span'}
			<LevelBadge level={record.level} />
			<TargetLabel target={record.target} />
			<div class="flex-grow-1">
				<i class="bi bi-diagram-3 text-warning me-2"></i>
				<strong>{record.name}</strong>
			</div>
			<i class="bi bi-box text-info"></i>
		{:else if record.type === 'event'}
			<LevelBadge level={record.level} />
			<TargetLabel target={record.target} />
			<div class="flex-grow-1">
				<i class="bi bi-lightning-fill me-2" style="color: var(--brand-cyan);"></i>
				<strong>{record.name}</strong>
			</div>
			<i class="bi bi-activity text-info"></i>
		{/if}

		<i class="bi bi-chevron-{expanded ? 'up' : 'down'}"></i>
	</button>
</div>

<style>
	.list-group-item {
		cursor: pointer;
		transition: background-color 0.15s ease-in-out;
	}

	.list-group-item-button {
		width: 100%;
		border: none;
		background-color: transparent;
		color: inherit;
		text-decoration: none;
	}
</style>
