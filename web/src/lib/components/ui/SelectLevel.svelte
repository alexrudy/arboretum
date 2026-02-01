<script lang="ts">
	import type { Level } from '$lib/api';
	import LevelBadge from './LevelBadge.svelte';

	let { level = $bindable() } = $props();
	let open = $state(false);
	const levels: (Level | null)[] = [null, 'TRACE', 'DEBUG', 'INFO', 'WARN', 'ERROR', 'FATAL'];
</script>

<div>
	<div>
		<div class="level-selector px-2">
			<button class="level-button" type="button" onclick={() => (open = !open)}>
				<LevelBadge {level} />
			</button>
		</div>

		{#if open}
			<div class="position-relative level-menu">
				<div class="position-absolute top-100 start-0 d-flex flex-column bg-dark rounded mt-2">
					{#each levels as selectLevel}
						<div class="level-select-item px-2 py-1">
							<button
								class="level-button"
								onclick={() => {
									open = false;
									level = selectLevel;
								}}
								type="button"
							>
								<LevelBadge level={selectLevel} />
							</button>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>

	<div class="visually-hidden">
		<select class="form-select" style="max-width: 150px;" bind:value={level}>
			<option value={null}>All Levels</option>
			<option value="TRACE">TRACE</option>
			<option value="DEBUG">DEBUG</option>
			<option value="INFO">INFO</option>
			<option value="WARN">WARN</option>
			<option value="ERROR">ERROR</option>
			<option value="FATAL">FATAL</option>
		</select>
	</div>
</div>

<style>
	.level-button {
		background-color: transparent;
		border: none;
		cursor: pointer;
		text-decoration: none;
		width: 100%;
		text-align: start;
	}

	.level-select-item:hover {
		opacity: 0.8;

		background-color: var(--bs-secondary);
	}

	.level-selector .level-button {
		background-color: inherit;
		border: none;
		cursor: pointer;
		text-decoration: none;
		width: 100%;
		text-align: start;
	}

	.level-menu {
		background-color: var(--bs-secondary);
		border-radius: var(--border-radius);
		box-shadow: var(--box-shadow);
		padding: var(--spacing-sm);
		width: 150px;
		z-index: 1000;
	}

	.level-selector {
		width: 150px;
	}
</style>
