<script lang="ts">
	import type { SpanEvent, Level } from '$lib/api';
	import { formatTimestamp, formatTimestampFull, getLevelColor } from '$lib/utils';

	let { events, spanLevel }: { events: SpanEvent[]; spanLevel: Level | null } = $props();

	// Parse events from JSON string if needed
	let parsedEvents = $derived.by(() => {
		if (!events) return [];

		// If events is a string (JSON), parse it
		if (typeof events === 'string') {
			try {
				return JSON.parse(events) as SpanEvent[];
			} catch {
				return [];
			}
		}

		return events;
	});

	// Extract level from event attributes or fall back to span level
	function getEventLevel(event: SpanEvent): Level | null {
		if (event.attributes?.level) {
			const levelStr = event.attributes.level.toString().toUpperCase();
			if (['TRACE', 'DEBUG', 'INFO', 'WARN', 'ERROR', 'FATAL'].includes(levelStr)) {
				return levelStr as Level;
			}
		}
		return spanLevel;
	}

	// Extract message from event
	function getEventMessage(event: SpanEvent): string {
		// Check common message fields
		if (event.attributes?.message) return event.attributes.message;
		if (event.attributes?.msg) return event.attributes.msg;
		if (event.attributes?.body) return event.attributes.body;

		// Fall back to event name
		return event.name;
	}

	// Get attributes without the message/level fields
	function getEventAttributes(event: SpanEvent): Record<string, any> | null {
		if (!event.attributes) return null;

		const { message, msg, body, level, ...rest } = event.attributes;
		return Object.keys(rest).length > 0 ? rest : null;
	}
</script>

{#if parsedEvents.length > 0}
	<div class="event-list">
		<strong class="text-info d-block mb-2">
			<i class="bi bi-lightning-fill me-1"></i>
			Events ({parsedEvents.length}):
		</strong>

		<div class="events-container">
			{#each parsedEvents as event}
				{@const eventLevel = getEventLevel(event)}
				{@const levelColor = getLevelColor(eventLevel)}
				{@const message = getEventMessage(event)}
				{@const timestamp = formatTimestamp(event.time)}
				{@const fullTimestamp = formatTimestampFull(event.time)}
				{@const attrs = getEventAttributes(event)}

				<div class="event-row">
					<div class="event-header d-flex align-items-center gap-2 mb-1">
						<span class="event-time text-muted small font-monospace" title={fullTimestamp}>
							<i class="bi bi-clock-fill"></i>
							{timestamp}
						</span>
						<span class="badge bg-{levelColor}">
							{eventLevel || 'INFO'}
						</span>
						<span class="event-name text-info small">
							<i class="bi bi-tag-fill"></i>
							{event.name}
						</span>
					</div>

					<div class="event-message">
						{message}
					</div>

					{#if attrs && Object.keys(attrs).length > 0}
						<details class="event-attributes mt-1">
							<summary class="text-muted small" style="cursor: pointer;">
								<i class="bi bi-braces"></i>
								Attributes ({Object.keys(attrs).length})
							</summary>
							<pre class="bg-darker p-2 rounded mt-1 small">{JSON.stringify(attrs, null, 2)}</pre>
						</details>
					{/if}
				</div>
			{/each}
		</div>
	</div>
{/if}

<style>
	.event-list {
		margin-top: 1rem;
	}

	.events-container {
		border-left: 3px solid var(--brand-cyan);
		padding-left: 1rem;
	}

	.event-row {
		padding: 0.75rem;
		margin-bottom: 0.5rem;
		background-color: rgba(13, 202, 240, 0.05);
		border-radius: 0.25rem;
		border: 1px solid rgba(13, 202, 240, 0.2);
	}

	.event-row:hover {
		background-color: rgba(13, 202, 240, 0.1);
	}

	.event-message {
		color: #e0e0e0;
		font-size: 0.95rem;
		padding-left: 0.5rem;
	}

	.event-time {
		min-width: 230px;
	}

	.event-name {
		font-family: 'Courier New', Courier, monospace;
	}

	.badge {
		font-size: 0.7rem;
		font-weight: 600;
		min-width: 60px;
		text-align: center;
	}

	.font-monospace {
		font-family: 'Courier New', Courier, monospace;
	}

	pre {
		color: #e0e0e0;
		margin: 0;
		max-height: 200px;
		overflow: auto;
	}

	details summary {
		user-select: none;
	}

	details[open] summary {
		margin-bottom: 0.5rem;
	}
</style>
