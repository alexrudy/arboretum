<script lang="ts">
	let { attrs: attributes }: { attrs: Record<string, any> | null } = $props();
</script>

{#if attributes && Object.keys(attributes).length > 0}
	<div class="attributes-display">
		<strong>Attributes:</strong>
		<pre class="bg-darker p-2 rounded mt-2">{JSON.stringify(attributes, null, 2)}</pre>
	</div>
{/if}

<style>
	.attributes-display strong {
		color: #adb5bd;
		font-size: 0.875rem;
	}

	pre {
		color: #e0e0e0;
		margin: 0;
		max-height: 400px;
		overflow: auto;
		font-size: 0.875rem;
	}
</style>
