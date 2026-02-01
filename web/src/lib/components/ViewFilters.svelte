<script lang="ts">
	let {
		spans = $bindable(),
		root = $bindable(),
		events = $bindable(),
		logs = $bindable()
	}: { spans: boolean; root: boolean; events: boolean; logs: boolean } = $props();
</script>

<div class="bg-dark border-bottom border-secondary">
	<div class="btn-group">
		<button
			class={`btn btn-sm btn-secondary ${spans ? 'enabled' : ''}`}
			onclick={() => {
				spans = !spans;
				if (!spans) {
					root = false;
				}
			}}
		>
			<i class="bi bi-diagram-3 text-info"></i>
			Spans
		</button>
		<button
			class={`btn btn-sm btn-secondary ${root ? 'enabled' : ''}`}
			disabled={!spans}
			onclick={() => {
				root = !root;
			}}>Roots</button
		>
	</div>

	<button
		class={`btn btn-sm btn-secondary ${events && !root ? 'enabled' : ''}`}
		onclick={() => {
			events = !events;
		}}
	>
		<i class="bi bi-lightning-fill" style="color: var(--brand-cyan);"></i>
		Events
	</button>
	<button
		class={`btn btn-sm btn-secondary ${logs && !root ? 'enabled' : ''}`}
		onclick={() => {
			logs = !logs;
		}}
	>
		<i class="bi bi-info-circle text-info"></i>
		Logs
	</button>
</div>

<style>
	button.enabled {
		background-color: color-mix(in srgb, var(--bs-primary), black 30%);
		color: white;
	}

	button.enabled:hover {
		background-color: color-mix(in srgb, var(--bs-primary), black 50%);
		color: white;
	}
</style>
