<script lang="ts">
	import type { EventRecord } from '$lib/api';
	import { formatTimestamp, formatTimestampFull, getLevelColor } from '$lib/utils';

	let { record: event }: { record: EventRecord } = $props();

	let levelColor = $derived(getLevelColor(event.level));
	let timestamp = $derived(formatTimestamp(event.timestamp));
	let fullTimestamp = $derived(formatTimestampFull(event.timestamp));
</script>

<div class="card">
	<div class="card-header">
		<div class="d-flex align-items-center gap-2">
			<i class="bi bi-lightning-fill" style="color: var(--brand-cyan);"></i>
			<strong>Span Event:</strong>
			<span>{event.name}</span>
			<span class="badge bg-{levelColor} ms-auto">{event.level || 'NONE'}</span>
		</div>
	</div>
	<div class="card-body">
		<div class="row g-3">
			<div class="col-md-6">
				<div class="detail-item">
					<strong>Timestamp:</strong>
					<span title={fullTimestamp}>{timestamp}</span>
				</div>
			</div>

			<div class="col-md-6">
				<div class="detail-item">
					<strong>Span ID:</strong>
					<code class="text-info">{event.span_id}</code>
				</div>
			</div>

			<div class="col-md-6">
				<div class="detail-item">
					<strong>Trace ID:</strong>
					<code class="text-info">{event.trace_id}</code>
				</div>
			</div>

			{#if event.service_name}
				<div class="col-md-6">
					<div class="detail-item">
						<strong>Service:</strong>
						<span>{event.service_name}</span>
					</div>
				</div>
			{/if}

			{#if event.target}
				<div class="col-md-6">
					<div class="detail-item">
						<strong>Target:</strong>
						<code>{event.target}</code>
					</div>
				</div>
			{/if}

			{#if event.attributes && Object.keys(event.attributes).length > 0}
				<div class="col-12">
					<div class="detail-item">
						<strong>Attributes:</strong>
						<pre class="bg-darker p-2 rounded mt-2">{JSON.stringify(
								event.attributes,
								null,
								2
							)}</pre>
					</div>
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
