<script lang="ts">
	import type { SpanRecord, EventRecord, LogOrSpan } from '$lib/api';
	import { api } from '$lib/api';
	import { formatDuration } from '$lib/utils';
	import SpanTree from './SpanTree.svelte';
	import SpanEventList from './SpanEventList.svelte';
	import DetailItem from './ui/DetailItem.svelte';
	import { manager } from '$lib/manager.svelte';
	import AttributesDisplay from './ui/AttributesDisplay.svelte';

	let {
		span,
		handleRecordClick
	}: { span: SpanRecord; handleRecordClick: (record: LogOrSpan) => void } = $props();

	const records = $derived.by(() => manager.getTrace(span.trace_id));
	const allEvents = $derived.by(() => records.filter((record) => record.type === 'event'));
	const allSpans = $derived.by(() => records.filter((record) => record.type === 'span'));
	let attributesExpanded = $state(false);

	let duration = $derived(formatDuration(span.start_time, span.end_time));
</script>

<div class="card-body">
	<h6 class="card-title">
		<i class="bi bi-diagram-3 text-info"></i>
		Span Details
	</h6>

	<div class="mb-3">
		<strong class="text-info">Trace Tree:</strong>
		<div class="mt-2 p-3 bg-darker rounded">
			<SpanTree
				spans={allSpans}
				events={allEvents}
				currentSpanId={span.span_id}
				{handleRecordClick}
			/>
		</div>
	</div>

	<div class="row g-1 mt-1">
		<DetailItem label="Name" className="col-md-6">
			<span>{span.name}</span>
		</DetailItem>

		{#if span.service_name}
			<DetailItem label="Service" className="col-md-6">
				<span>{span.service_name}</span>
			</DetailItem>
		{/if}

		{#if span.target}
			<DetailItem label="Target" className="col-md-6">
				<span class="font-monospace small">{span.target}</span>
			</DetailItem>
		{/if}

		<DetailItem label="Duration" className="col-md-6">
			<span>{duration}</span>
		</DetailItem>

		<DetailItem label="Trace ID" className="col-md-6">
			<span class="font-monospace small">{span.trace_id}</span>
		</DetailItem>

		<DetailItem label="Span ID" className="col-md-6">
			<span class="font-monospace small">{span.span_id}</span>
		</DetailItem>

		{#if span.parent_span_id}
			<DetailItem label="Parent Span ID" className="col-md-6">
				<span class="font-monospace small">{span.parent_span_id}</span>
			</DetailItem>
		{/if}

		{#if span.kind}
			<DetailItem label="Kind" className="col-md-6">
				<span>{span.kind}</span>
			</DetailItem>
		{/if}

		{#if span.status && span.status !== 'Unset'}
			<DetailItem label="Status" className="col-md-6">
				<span>{span.status}</span>
			</DetailItem>
		{/if}
	</div>

	{#if span.events && span.events.length > 0}
		<div class="row">
			<SpanEventList events={span.events} spanLevel={span.level} />
		</div>
	{/if}

	{#if span.attributes && Object.keys(span.attributes).length > 0}
		<div class="row mt-3">
			<AttributesDisplay attrs={span.attributes} />
		</div>
	{/if}
</div>

<style>
	.font-monospace {
		font-family: 'Courier New', Courier, monospace;
	}
</style>
