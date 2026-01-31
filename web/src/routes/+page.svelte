<script lang="ts">
	import { onMount } from 'svelte';
	import NavigationBar from '$lib/components/NavigationBar.svelte';
	import RecordRow from '$lib/components/RecordRow.svelte';
	import LogDetail from '$lib/components/LogDetail.svelte';
	import SpanDetail from '$lib/components/SpanDetail.svelte';
	import EventDetail from '$lib/components/EventDetail.svelte';
	import MetadataFooter from '$lib/components/MetadataFooter.svelte';
	import { api } from '$lib/api';
	import type { LogOrSpan, Level, DatabaseStats } from '$lib/api';

	let records = $state<LogOrSpan[]>([]);
	let expandedRecordId = $state<string | null>(null);
	let metadata = $state<DatabaseStats | null>(null);
	let loading = $state(false);
	let loadingOlder = $state(false);
	let error = $state<string | null>(null);
	let scrollContainer: HTMLElement | null = $state(null);
	let hasMoreRecords = $state(true);
	let totalRecordsLoaded = $state(0);
	let followMode = $state(true); // Auto-scroll to latest records
	let isAtBottom = $state(true); // Track if scrolled to bottom

	// Search parameters
	let serviceName = $state('');
	let target = $state('');
	let level = $state<Level | null>(null);

	const PAGE_SIZE = 50; // Number of records to load at a time
	const AUTO_REFRESH_INTERVAL = 2000; // Refresh every 2 seconds when at bottom
	const METADATA_REFRESH_INTERVAL = 5000; // Refresh metadata every 5 seconds

	// Scroll to bottom when records change (only if follow mode is enabled)
	$effect(() => {
		if (records.length > 0 && scrollContainer && !loading && followMode) {
			// Use setTimeout to ensure DOM has updated
			setTimeout(() => {
				if (scrollContainer) {
					scrollContainer.scrollTop = scrollContainer.scrollHeight;
				}
			}, 0);
		}
	});

	function checkIfAtBottom(): boolean {
		if (!scrollContainer) return false;
		const threshold = 100; // Within 100px of bottom
		const distanceFromBottom =
			scrollContainer.scrollHeight - scrollContainer.scrollTop - scrollContainer.clientHeight;
		return distanceFromBottom < threshold;
	}

	async function loadRecords() {
		loading = true;
		error = null;
		try {
			const newRecords = await api.getRecords({
				service_name: serviceName || undefined,
				target: target || undefined,
				level: level || undefined,
				limit: PAGE_SIZE,
				offset: 0
			});
			records = newRecords;
			totalRecordsLoaded = newRecords.length;
			hasMoreRecords = newRecords.length === PAGE_SIZE;
			expandedRecordId = null; // Collapse all when new search
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load records';
			records = [];
			totalRecordsLoaded = 0;
			hasMoreRecords = false;
		} finally {
			loading = false;
		}
	}

	async function loadOlderRecords() {
		if (loadingOlder || !hasMoreRecords || loading) return;

		loadingOlder = true;
		try {
			const olderRecords = await api.getRecords({
				service_name: serviceName || undefined,
				target: target || undefined,
				level: level || undefined,
				limit: PAGE_SIZE,
				offset: totalRecordsLoaded
			});

			if (olderRecords.length > 0) {
				// Save current scroll position
				const previousScrollHeight = scrollContainer?.scrollHeight ?? 0;
				const previousScrollTop = scrollContainer?.scrollTop ?? 0;

				// Prepend older records to the beginning
				records = [...olderRecords, ...records];
				totalRecordsLoaded += olderRecords.length;
				hasMoreRecords = olderRecords.length === PAGE_SIZE;

				// Restore scroll position after DOM update
				setTimeout(() => {
					if (scrollContainer) {
						const newScrollHeight = scrollContainer.scrollHeight;
						scrollContainer.scrollTop =
							previousScrollTop + (newScrollHeight - previousScrollHeight);
					}
				}, 0);
			} else {
				hasMoreRecords = false;
			}
		} catch (e) {
			console.error('Failed to load older records:', e);
		} finally {
			loadingOlder = false;
		}
	}

	async function loadMetadata() {
		try {
			metadata = await api.getMetadata();
		} catch (e) {
			console.error('Failed to load metadata:', e);
		}
	}

	async function checkForNewRecords() {
		if (loading || loadingOlder || !isAtBottom) return;

		try {
			// Fetch latest records to see if there are new ones
			const latestRecords = await api.getRecords({
				service_name: serviceName || undefined,
				target: target || undefined,
				level: level || undefined,
				limit: PAGE_SIZE,
				offset: 0
			});

			if (latestRecords.length > 0 && records.length > 0) {
				// Check if there are newer records than what we have
				const latestTimestamp = getRecordTimestamp(latestRecords[latestRecords.length - 1]);
				const currentLatestTimestamp = getRecordTimestamp(records[records.length - 1]);

				if (latestTimestamp > currentLatestTimestamp) {
					// Find new records that we don't have yet
					const newRecords = latestRecords.filter((newRec) => {
						const newTs = getRecordTimestamp(newRec);
						return newTs > currentLatestTimestamp;
					});

					if (newRecords.length > 0) {
						records = [...records, ...newRecords];
						totalRecordsLoaded += newRecords.length;
					}
				}
			}
		} catch (e) {
			console.error('Failed to check for new records:', e);
		}
	}

	function getRecordTimestamp(record: LogOrSpan): number {
		return record.timestamp;
	}

	function toggleFollowMode() {
		followMode = !followMode;
		if (followMode && scrollContainer) {
			// Immediately scroll to bottom when enabling follow mode
			scrollContainer.scrollTop = scrollContainer.scrollHeight;
		}
	}

	function handleSearch(detail: { serviceName: string; target: string; level: Level | null }) {
		serviceName = detail.serviceName;
		target = detail.target;
		level = detail.level;
		loadRecords();
	}

	function handleRecordClick(record: LogOrSpan) {
		let recordId: string;
		if (record.type === 'log') {
			recordId = `log-${record.timestamp}`;
		} else if (record.type === 'span') {
			recordId = `span-${record.span_id}`;
		} else {
			recordId = `event-${record.timestamp}-${record.span_id}`;
		}

		if (expandedRecordId === recordId) {
			expandedRecordId = null;
		} else {
			expandedRecordId = recordId;
		}
	}

	function handleScroll() {
		if (!scrollContainer) return;

		// Update isAtBottom state
		isAtBottom = checkIfAtBottom();

		// Enable follow mode when user scrolls to bottom
		if (isAtBottom && !followMode) {
			followMode = true;
		}

		// Disable follow mode when user scrolls away from bottom
		if (!isAtBottom && followMode) {
			followMode = false;
		}

		// Load more when scrolled within 200px of the top
		if (!loadingOlder && hasMoreRecords && scrollContainer.scrollTop < 200) {
			loadOlderRecords();
		}
	}

	onMount(() => {
		loadRecords();
		loadMetadata();

		// Add scroll listener
		if (scrollContainer) {
			scrollContainer.addEventListener('scroll', handleScroll);
		}

		// Auto-refresh for new records when at bottom
		const autoRefreshInterval = setInterval(() => {
			if (isAtBottom && followMode) {
				checkForNewRecords();
			}
		}, AUTO_REFRESH_INTERVAL);

		// Refresh metadata periodically (independent of follow mode)
		const metadataInterval = setInterval(loadMetadata, METADATA_REFRESH_INTERVAL);

		return () => {
			clearInterval(autoRefreshInterval);
			clearInterval(metadataInterval);
			if (scrollContainer) {
				scrollContainer.removeEventListener('scroll', handleScroll);
			}
		};
	});
</script>

<div class="d-flex flex-column vh-100">
	<NavigationBar bind:serviceName bind:target bind:level onsearch={handleSearch} />

	<main class="flex-grow-1 overflow-auto pb-5" bind:this={scrollContainer}>
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
				{#if loadingOlder}
					<div class="text-center py-2">
						<div class="spinner-border spinner-border-sm text-info" role="status">
							<span class="visually-hidden">Loading older records...</span>
						</div>
						<span class="text-muted small ms-2">Loading older records...</span>
					</div>
				{/if}

				<div class="list-group">
					{#each records as record}
						{@const recordId =
							record.type === 'log'
								? `log-${record.timestamp}`
								: record.type === 'span'
									? `span-${record.span_id}`
									: `event-${record.timestamp}-${record.span_id}`}
						{@const isExpanded = expandedRecordId === recordId}

						<div class="mb-2">
							<RecordRow {record} expanded={isExpanded} onclick={() => handleRecordClick(record)} />

							{#if isExpanded}
								<div class="mt-2">
									{#if record.type === 'log'}
										<LogDetail log={record} />
									{:else if record.type === 'span'}
										<SpanDetail span={record} {handleRecordClick} />
									{:else if record.type === 'event'}
										<EventDetail {record} />
									{/if}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</main>

	<!-- Follow Mode Toggle Button -->
	<button
		class="follow-toggle btn btn-{followMode ? 'info' : 'secondary'} rounded-circle shadow"
		onclick={toggleFollowMode}
		title={followMode ? 'Following latest (click to pause)' : 'Paused (click to follow latest)'}
	>
		<i class="bi bi-{followMode ? 'pause-fill' : 'play-fill'}"></i>
	</button>

	<MetadataFooter bind:stats={metadata} />
</div>

<style>
	.follow-toggle {
		position: fixed;
		bottom: 80px;
		right: 20px;
		width: 50px;
		height: 50px;
		z-index: 1000;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.2rem;
		transition: all 0.3s ease;
	}

	.follow-toggle:hover {
		transform: scale(1.1);
	}
</style>
