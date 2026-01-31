<script lang="ts">
	import type { Level } from '$lib/api';

	let {
		serviceName = $bindable(''),
		target = $bindable(''),
		level = $bindable<Level | null>(null),
		onsearch
	}: {
		serviceName?: string;
		target?: string;
		level?: Level | null;
		onsearch?: (detail: { serviceName: string; target: string; level: Level | null }) => void;
	} = $props();

	function handleSearch() {
		onsearch?.({ serviceName, target, level });
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
					bind:value={serviceName}
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
					bind:value={target}
					onkeypress={handleKeyPress}
				/>
			</div>

			<select class="form-select" style="max-width: 150px;" bind:value={level}>
				<option value={null}>All Levels</option>
				<option value="TRACE">TRACE</option>
				<option value="DEBUG">DEBUG</option>
				<option value="INFO">INFO</option>
				<option value="WARN">WARN</option>
				<option value="ERROR">ERROR</option>
				<option value="FATAL">FATAL</option>
			</select>

			<button class="btn btn-primary" onclick={handleSearch}>
				<i class="bi bi-search"></i>
				Search
			</button>
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
