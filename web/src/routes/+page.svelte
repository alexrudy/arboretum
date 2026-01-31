<script lang="ts">
	import { onMount } from 'svelte';
	import NavigationBar from '$lib/components/NavigationBar.svelte';
	import RecordRow from '$lib/components/RecordRow.svelte';
	import LogDetail from '$lib/components/LogDetail.svelte';
	import SpanDetail from '$lib/components/SpanDetail.svelte';
	import MetadataFooter from '$lib/components/MetadataFooter.svelte';
	import { api } from '$lib/api';
	import type { LogOrSpan, Level, DatabaseStats } from '$lib/api';

	let records: LogOrSpan[] = [];
	let expandedRecordId: string | null = null;
	let metadata: DatabaseStats | null = null;
	let loading = false;
	let error: string | null = null;

	// Search parameters
	let serviceName = '';
	let target = '';
	let level: Level | null = null;

	async function loadRecords() {
		loading = true;
		error = null;
		try {
			records = await api.getRecords({
				service_name: serviceName || undefined,
				target: target || undefined,
				level: level || undefined
			});
			expandedRecordId = null; // Collapse all when new search
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load records';
			records = [];
		} finally {
			loading = false;
		}
	}

	async function loadMetadata() {
		try {
			metadata = await api.getMetadata();
		} catch (e) {
			console.error('Failed to load metadata:', e);
		}
	}

	function handleSearch(
		event: CustomEvent<{ serviceName: string; target: string; level: Level | null }>
	) {
		serviceName = event.detail.serviceName;
		target = event.detail.target;
		level = event.detail.level;
		loadRecords();
	}

	function handleRecordClick(record: LogOrSpan) {
		const recordId = record.type === 'log' ? `log-${record.timestamp}` : `span-${record.span_id}`;

		if (expandedRecordId === recordId) {
			expandedRecordId = null;
		} else {
			expandedRecordId = recordId;
		}
	}

	onMount(() => {
		loadRecords();
		loadMetadata();

		// Refresh metadata every 30 seconds
		const interval = setInterval(loadMetadata, 30000);
		return () => clearInterval(interval);
	});
</script>

<div class="d-flex flex-column vh-100">
	<NavigationBar on:search={handleSearch} />

	<main class="flex-grow-1 overflow-auto pb-5">
		<div class="container-fluid py-3">
			{#if error}
				<div class="alert alert-danger" role="alert">
					<i class="bi bi-exclamation-triangle-fill me-2"></i>
					{error}
				</div>
			{/if}

			{#if loading}
				<div class="text-center py-5">
					<div class="spinner-border text-info" role="status">
						<span class="visually-hidden">Loading...</span>
					</div>
				</div>
			{:else if records.length === 0}
				<div class="text-center py-5 text-muted">
					<i class="bi bi-inbox" style="font-size: 3rem;"></i>
					<p class="mt-3">No records found</p>
				</div>
			{:else}
				<div class="list-group">
					{#each records as record}
						{@const recordId =
							record.type === 'log' ? `log-${record.timestamp}` : `span-${record.span_id}`}
						{@const isExpanded = expandedRecordId === recordId}

						<div class="mb-2">
							<RecordRow
								{record}
								expanded={isExpanded}
								on:click={() => handleRecordClick(record)}
							/>

							{#if isExpanded}
								<div class="mt-2">
									{#if record.type === 'log'}
										<LogDetail log={record} />
									{:else}
										<SpanDetail span={record} />
									{/if}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</main>

	<MetadataFooter stats={metadata} />
</div>
