<script lang="ts">
	let { target, minWidth = '200px' }: { target: string | null; minWidth?: string } = $props();
</script>

<div class="text-muted small target-label" style="min-width: {minWidth};">
	<i class="bi bi-code-slash"></i>
	{target || 'unknown'}
</div>

<style>
	.target-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
