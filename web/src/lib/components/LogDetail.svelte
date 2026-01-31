<script lang="ts">
	import type { LogRecord } from '$lib/api';

	export let log: LogRecord;
</script>

<div class="card mt-2 mb-3">
	<div class="card-body">
		<h6 class="card-title">
			<i class="bi bi-info-circle text-info"></i>
			Log Details
		</h6>

		<div class="row g-3 mt-1">
			{#if log.service_name}
				<div class="col-md-6">
					<strong class="text-info">Service:</strong>
					<span class="ms-2">{log.service_name}</span>
				</div>
			{/if}

			{#if log.target}
				<div class="col-md-6">
					<strong class="text-info">Target:</strong>
					<span class="ms-2 font-monospace small">{log.target}</span>
				</div>
			{/if}

			{#if log.span_id}
				<div class="col-md-6">
					<strong class="text-info">Span ID:</strong>
					<span class="ms-2 font-monospace small">{log.span_id}</span>
				</div>
			{/if}

			{#if log.trace_id}
				<div class="col-md-6">
					<strong class="text-info">Trace ID:</strong>
					<span class="ms-2 font-monospace small">{log.trace_id}</span>
				</div>
			{/if}
		</div>

		{#if log.attributes && Object.keys(log.attributes).length > 0}
			<div class="mt-3">
				<strong class="text-info">Attributes:</strong>
				<pre class="bg-darker p-2 rounded mt-2 small">{JSON.stringify(
						log.attributes,
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
