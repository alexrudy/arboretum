<script lang="ts">
	import type { DatabaseStats } from '$lib/api';
	import { formatBytes } from '$lib/utils';
	import { manager } from '$lib/manager.svelte';

	let { stats = $bindable(null) }: { stats?: DatabaseStats | null } = $props();
</script>

<footer class="bg-dark text-light border-top border-secondary py-2 px-3">
	<div class="container-fluid">
		{#if stats}
			<div class="row text-center small">
				<div class="col">
					<i class="bi bi-file-text text-info"></i>
					<strong class="ms-1">{stats.total_logs.toLocaleString()}</strong>
					<span class=" ms-1">logs</span>
				</div>
				<div class="col">
					<i class="bi bi-diagram-3 text-info"></i>
					<strong class="ms-1">{stats.total_spans.toLocaleString()}</strong>
					<span class=" ms-1">spans</span>
				</div>
				<div class="col">
					<i class="bi bi-hdd text-info"></i>
					<strong class="ms-1">{formatBytes(stats.database_size_bytes)}</strong>
					<span class=" ms-1">database</span>
				</div>
				<div class="col">
					<i class="bi bi-info-circle text-info"></i>
					<span class="">Arboretum</span>
				</div>
				<div class="col">
					<i class="bi bi-database text-info"></i>
					<span class="">{manager.records.length.toLocaleString()}</span>
					<span class="ms-1">loaded records</span>
				</div>
			</div>
		{:else}
			<div class="text-center small">
				<i class="bi bi-hourglass-split"></i>
				Loading metadata...
			</div>
		{/if}
	</div>
</footer>

<style>
	footer {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		z-index: 1000;
	}
</style>
