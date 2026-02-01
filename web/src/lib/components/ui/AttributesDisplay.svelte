<script lang="ts">
	let { attrs: attributes }: { attrs: Record<string, any> | null } = $props();

	let expanded = $state(false);
</script>

{#if attributes && Object.keys(attributes).length > 0}
	<div class="attributes-display">
		<button class="d-block attributes-button" onclick={() => (expanded = !expanded)}>
			<div>
				<strong class="text-info">Attributes:</strong>
				<i class="text-info bi bi-chevron-{expanded ? 'up' : 'down'}"></i>
			</div>
			{#if expanded}
				<div>
					<pre class="bg-darker p-2 rounded mt-2 small">{JSON.stringify(attributes, null, 2)}</pre>
				</div>
			{/if}
		</button>
	</div>
{/if}

<style>
	pre {
		color: #e0e0e0;
		margin: 0;
		max-height: 400px;
		overflow: auto;
	}

	.attributes-button {
		background-color: transparent;
		border: none;
		cursor: pointer;
		text-decoration: none;
		width: 100%;
		text-align: start;
	}
</style>
