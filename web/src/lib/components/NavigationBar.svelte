<script lang="ts">
	import type { Snippet } from 'svelte';
	import SelectLevel from './ui/SelectLevel.svelte';
	import type { ArboretumManager } from '$lib/manager.svelte';

	let {
		manager,
		children
	}: {
		manager: ArboretumManager;
		children?: Snippet;
	} = $props();

	function handleSearch() {
		manager.loadRecords({ replace: true });
	}

	function handleKeyPress(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleSearch();
		}
	}
</script>

<nav class="navbar navbar-dark bg-dark border-bottom border-secondary sticky-top">
	<div class="container-fluid">
		<a class="navbar-brand d-flex align-items-center" href="/">
			<i class="bi bi-tree-fill text-info me-2"></i>
			<span>Arboretum</span>
		</a>

		<div class="d-flex gap-2 flex-grow-1 mx-4">
			<div class="input-group" style="max-width: 300px;">
				<span class="input-group-text bg-dark text-light border-secondary">
					<i class="bi bi-hdd-network"></i>
				</span>
				<input
					type="text"
					class="form-control"
					placeholder="Service name"
					bind:value={manager.service_name}
					onkeypress={handleKeyPress}
				/>
			</div>

			<div class="input-group" style="max-width: 300px;">
				<span class="input-group-text bg-dark text-light border-secondary">
					<i class="bi bi-bullseye"></i>
				</span>
				<input
					type="text"
					class="form-control"
					placeholder="Target (e.g., module::path)"
					bind:value={manager.target}
					onkeypress={handleKeyPress}
				/>
			</div>

			<div class="input-group" style="max-width: 300px;">
				<span class="input-group-text bg-dark text-light border-secondary">
					<i class="bi bi-tag-fill"></i>
				</span>
				<input
					type="text"
					class="form-control"
					placeholder="Message"
					bind:value={manager.message}
					onkeypress={handleKeyPress}
				/>
			</div>

			<SelectLevel bind:level={manager.level} onChange={handleSearch} />

			<button class="btn btn-primary" onclick={handleSearch}>
				<i class="bi bi-search"></i>
				Search
			</button>
		</div>
		<div>
			{@render children?.()}
		</div>
	</div>
</nav>

<style>
	.navbar-brand {
		font-weight: 600;
		font-size: 1.25rem;
	}

	.input-group-text {
		min-width: 45px;
		justify-content: center;
	}
</style>
