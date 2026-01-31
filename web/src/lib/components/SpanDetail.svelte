<script lang="ts">
	import type { SpanRecord } from '$lib/api';
	import { api } from '$lib/api';
	import { formatDuration } from '$lib/utils';
	import SpanTree from './SpanTree.svelte';

	export let span: SpanRecord;

	let allSpans: SpanRecord[] = [];
	let loading = true;

	// Load all spans in the trace to build the tree
	async function loadTraceSpans() {
		try {
			const spans = await api.getSpans({ trace_id: span.trace_id });
			allSpans = spans as SpanRecord[];
		} catch (error) {
			console.error('Failed to load trace spans:', error);
			allSpans = [span];
		} finally {
			loading = false;
		}
	}

	$: if (span) {
		loading = true;
		loadTraceSpans();
	}

	$: duration = formatDuration(span.start_time, span.end_time);
</script>

<div class="card mt-2 mb-3">
	<div class="card-body">
		<h6 class="card-title">
			<i class="bi bi-diagram-3 text-info"></i>
			Span Details
		</h6>

		{#if !loading}
			<div class="mb-3">
				<strong class="text-info">Trace Tree:</strong>
				<div class="mt-2 p-3 bg-darker rounded">
					<SpanTree spans={allSpans} currentSpanId={span.span_id} />
				</div>
			</div>
		{:else}
			<div class="text-center text-muted my-3">
				<i class="bi bi-hourglass-split"></i>
				Loading trace tree...
			</div>
		{/if}

		<div class="row g-3 mt-1">
			<div class="col-md-6">
				<strong class="text-info">Name:</strong>
				<span class="ms-2">{span.name}</span>
			</div>

			{#if span.service_name}
				<div class="col-md-6">
					<strong class="text-info">Service:</strong>
					<span class="ms-2">{span.service_name}</span>
				</div>
			{/if}

			{#if span.target}
				<div class="col-md-6">
					<strong class="text-info">Target:</strong>
					<span class="ms-2 font-monospace small">{span.target}</span>
				</div>
			{/if}

			<div class="col-md-6">
				<strong class="text-info">Duration:</strong>
				<span class="ms-2">{duration}</span>
			</div>

			<div class="col-md-6">
				<strong class="text-info">Trace ID:</strong>
				<span class="ms-2 font-monospace small">{span.trace_id}</span>
			</div>

			<div class="col-md-6">
				<strong class="text-info">Span ID:</strong>
				<span class="ms-2 font-monospace small">{span.span_id}</span>
			</div>

			{#if span.parent_span_id}
				<div class="col-md-6">
					<strong class="text-info">Parent Span ID:</strong>
					<span class="ms-2 font-monospace small">{span.parent_span_id}</span>
				</div>
			{/if}

			{#if span.kind}
				<div class="col-md-6">
					<strong class="text-info">Kind:</strong>
					<span class="ms-2">{span.kind}</span>
				</div>
			{/if}

			{#if span.status}
				<div class="col-md-6">
					<strong class="text-info">Status:</strong>
					<span class="ms-2">{span.status}</span>
				</div>
			{/if}
		</div>

		{#if span.events && span.events.length > 0}
			<div class="mt-3">
				<strong class="text-info">Events:</strong>
				<pre class="bg-darker p-2 rounded mt-2 small">{JSON.stringify(span.events, null, 2)}</pre>
			</div>
		{/if}

		{#if span.attributes && Object.keys(span.attributes).length > 0}
			<div class="mt-3">
				<strong class="text-info">Attributes:</strong>
				<pre class="bg-darker p-2 rounded mt-2 small">{JSON.stringify(
						span.attributes,
						null,
						2
					)}</pre>
			</div>
		{/if}
	</div>
</div>

<style>
	pre {
		color: #e0e0e0;
		margin: 0;
		max-height: 400px;
		overflow: auto;
	}

	.font-monospace {
		font-family: 'Courier New', Courier, monospace;
	}
</style>
