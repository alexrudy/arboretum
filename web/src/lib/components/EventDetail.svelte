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

<div class="card">
	<div class="card-header">
		<div class="d-flex align-items-center gap-2">
			<i class="bi bi-lightning-fill" style="color: var(--brand-cyan);"></i>
			<strong>Span Event:</strong>
			<span>{event.name}</span>
			<span class="ms-auto"><LevelBadge level={event.level} /></span>
		</div>
	</div>
	<div class="card-body">
		<div class="row g-3">
			<div class="col-md-6">
				<DetailItem label="Timestamp">
					<span title={fullTimestamp}>{timestamp}</span>
				</DetailItem>
			</div>

			<div class="col-md-6">
				<DetailItem label="Span ID">
					<code class="text-info">{event.span_id}</code>
				</DetailItem>
			</div>

			<div class="col-md-6">
				<DetailItem label="Trace ID">
					<code class="text-info">{event.trace_id}</code>
				</DetailItem>
			</div>

			{#if event.service_name}
				<div class="col-md-6">
					<DetailItem label="Service">
						{event.service_name}
					</DetailItem>
				</div>
			{/if}

			{#if event.target}
				<div class="col-md-6">
					<DetailItem label="Target">
						<code>{event.target}</code>
					</DetailItem>
				</div>
			{/if}

			{#if event.attributes && Object.keys(event.attributes).length > 0}
				<div class="col-12">
					<AttributesDisplay attrs={event.attributes} />
				</div>
			{/if}
		</div>
	</div>
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

	pre {
		color: #e0e0e0;
		margin: 0;
		max-height: 400px;
		overflow: auto;
	}

	code {
		font-size: 0.875rem;
	}
</style>
