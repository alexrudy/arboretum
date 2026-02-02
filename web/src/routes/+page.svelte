<script lang="ts">
	import { onMount } from 'svelte';
	import NavigationBar from '$lib/components/NavigationBar.svelte';
	import RecordRow from '$lib/components/RecordRow.svelte';
	import LogDetail from '$lib/components/LogDetail.svelte';
	import SpanDetail from '$lib/components/SpanDetail.svelte';
	import EventDetail from '$lib/components/EventDetail.svelte';
	import MetadataFooter from '$lib/components/MetadataFooter.svelte';
	import { api, Levels } from '$lib/api';
	import { ArboretumManager } from '$lib/manager.svelte';
	import type {
		LogOrSpan,
		Level,
		DatabaseStats,
		PaginatedResponse,
		GetRecordParams
	} from '$lib/api';
	import { getRecordID } from '$lib/types';
	import ViewFilters from '$lib/components/ViewFilters.svelte';

	let expandedRecordId = $state<string | null>(null);
	let metadata = $state<DatabaseStats | null>(null);
	let error = $state<string | null>(null);
	let scrollContainer: HTMLElement | null = $state(null);
	let followMode = $state(true); // Auto-scroll to latest records
	let isAtBottom = $state(true); // Track if scrolled to bottom

	let showSpans = $state(true);
	let showLogs = $state(true);
	let showOnlyRootSpans = $state(false);
	let showEvents = $state(true);

	const manager = new ArboretumManager(api);

	const AUTO_REFRESH_INTERVAL = 2000; // Refresh every 2 seconds when at bottom
	const METADATA_REFRESH_INTERVAL = 5000; // Refresh metadata every 5 seconds
	const BUFFER_SIZE = 200; // Number of items to render above/below visible area

	function applySearchFilters(record: LogOrSpan): boolean {
		if (!showEvents && record.type === 'event') {
			return false;
		}

		if (!showLogs && record.type === 'log') {
			return false;
		}

		if (!showSpans && record.type === 'span') {
			return false;
		}

		if (showOnlyRootSpans) {
			return record.type === 'span' && !record.parent_span_id;
		}
		return true;
	}

	// Build and flatten hierarchy
	let hierarchicalRecords = $derived.by(() => {
		return manager.hierarchy(applySearchFilters);
	});

	// Scroll to bottom when records change (only if follow mode is enabled)
	$effect(() => {
		if (manager.records.length > 0 && scrollContainer && !manager.loading && followMode) {
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

	async function loadMetadata() {
		try {
			metadata = await api.getMetadata();
		} catch (e) {
			console.error('Failed to load metadata:', e);
		}
	}

	function toggleFollowMode() {
		followMode = !followMode;
		if (followMode && scrollContainer) {
			// Immediately scroll to bottom when enabling follow mode
			scrollContainer.scrollTop = scrollContainer.scrollHeight;
		}
	}

	function handleSearch(params: GetRecordParams) {
		manager.loadRecords({ replace: true });
	}

	function handleRecordClick(record: LogOrSpan) {
		let recordId = getRecordID(record);

		// Disable follow mode when jumping to a specific record
		followMode = false;

		/// Toggle expanded record
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

		// // Enable follow mode when user scrolls to bottom
		// if (isAtBottom && !followMode) {
		// 	followMode = true;
		// }

		// Disable follow mode when user scrolls away from bottom
		if (!isAtBottom && followMode) {
			followMode = false;
		}
	}

	onMount(() => {
		manager.loadRecords({});
		loadMetadata();

		// Add scroll listener
		if (scrollContainer) {
			scrollContainer.addEventListener('scroll', handleScroll);
		}

		// Auto-refresh for new records when at bottom
		const autoRefreshInterval = setInterval(() => {
			if (isAtBottom && followMode) {
				manager.loadRecords({});
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
	<NavigationBar {manager}>
		<!-- View controls -->
		<ViewFilters
			bind:root={showOnlyRootSpans}
			bind:events={showEvents}
			bind:logs={showLogs}
			bind:spans={showSpans}
		/>
	</NavigationBar>

	<main class="flex-grow-1 overflow-auto pb-5" bind:this={scrollContainer}>
		<div class="container-fluid py-3">
			{#if error}
				<div class="alert alert-danger" role="alert">
					<i class="bi bi-exclamation-triangle-fill me-2"></i>
					{error}
				</div>
			{/if}

			{#if manager.loading && manager.records.length === 0}
				<div class="text-center py-5">
					<div class="spinner-border text-info" role="status">
						<span class="visually-hidden">Loading...</span>
					</div>
				</div>
			{:else if manager.records.length === 0}
				<div class="text-center py-5 text-muted">
					<i class="bi bi-inbox" style="font-size: 3rem;"></i>
					<p class="mt-3">No records found</p>
				</div>
			{:else}
				<div class="list-group">
					{#each hierarchicalRecords as hierarchicalRecord}
						{@const record = hierarchicalRecord.record}
						{@const recordId =
							record.type === 'log'
								? `log-${record.timestamp}`
								: record.type === 'span'
									? `span-${record.span_id}`
									: `event-${record.timestamp}-${record.span_id}`}
						{@const isExpanded = expandedRecordId === recordId}
						{@const indentPx = hierarchicalRecord.depth * 20}

						<div id={recordId} class="card mb-2" style="margin-left: {indentPx}px;">
							<RecordRow {record} expanded={isExpanded} onclick={() => handleRecordClick(record)} />

							{#if isExpanded}
								<div>
									{#if record.type === 'log'}
										<LogDetail log={record} />
									{:else if record.type === 'span'}
										<SpanDetail span={record} {handleRecordClick} />
									{:else if record.type === 'event'}
										<EventDetail {record} bind:expandedRecordId />
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
