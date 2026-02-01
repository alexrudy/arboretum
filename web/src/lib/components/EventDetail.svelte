<script lang="ts">
	import type { EventRecord } from '$lib/api';
	import LevelBadge from './ui/LevelBadge.svelte';
	import DetailItem from './ui/DetailItem.svelte';
	import AttributesDisplay from './ui/AttributesDisplay.svelte';
	import { formatTimestamp, formatTimestampFull } from '$lib/utils';

	let { record: event }: { record: EventRecord } = $props();

	let timestamp = $derived(formatTimestamp(event.timestamp));
	let fullTimestamp = $derived(formatTimestampFull(event.timestamp));
</script>

<div class="card-body">
	<div class="row g-3">
		<div class="d-flex align-items-center gap-2">
			<i class="bi bi-lightning-fill" style="color: var(--brand-cyan);"></i>
			<strong class="text-muted">Span Event:</strong>
			<span>{event.name}</span>
			<span class="ms-auto"><LevelBadge level={event.level} /></span>
		</div>
	</div>
	<div class="row g-1 mt-1">
		<DetailItem label="Timestamp" className="col-md-6">
			<span title={fullTimestamp}>{timestamp}</span>
		</DetailItem>

		<DetailItem label="Span ID" className="col-md-6">
			<code class="text-info">{event.span_id}</code>
		</DetailItem>

		<DetailItem label="Trace ID" className="col-md-6">
			<code class="text-info">{event.trace_id}</code>
		</DetailItem>

		{#if event.service_name}
			<DetailItem label="Service" className="col-md-6">
				{event.service_name}
			</DetailItem>
		{/if}

		{#if event.target}
			<DetailItem label="Target" className="col-md-6">
				<code>{event.target}</code>
			</DetailItem>
		{/if}

		{#if event.attributes && Object.keys(event.attributes).length > 0}
			<div class="col-12">
				<AttributesDisplay attrs={event.attributes} />
			</div>
		{/if}
	</div>
</div>

<style>
	code {
		font-size: 0.875rem;
	}
</style>
