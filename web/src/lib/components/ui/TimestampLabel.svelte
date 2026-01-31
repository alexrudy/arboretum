<script lang="ts">
	import { formatTimestamp, formatTimestampFull } from '$lib/utils';

	let { timestamp, minWidth = '230px' }: { timestamp: number; minWidth?: string } = $props();

	let formattedTime = $derived(formatTimestamp(timestamp));
	let fullTime = $derived(formatTimestampFull(timestamp));
</script>

<div class="text-muted small" style="min-width: {minWidth};" title={fullTime}>
	<i class="bi bi-clock"></i>
	{formattedTime}
</div>
