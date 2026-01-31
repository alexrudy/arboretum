import type { Level } from './api';

export function formatTimestamp(nanos: number): string {
	// Convert nanoseconds to milliseconds
	const ms = nanos / 1_000_000;
	const date = new Date(ms);

	// Format with millisecond precision
	const dateStr = date.toLocaleDateString();
	const hours = date.getHours().toString().padStart(2, '0');
	const minutes = date.getMinutes().toString().padStart(2, '0');
	const seconds = date.getSeconds().toString().padStart(2, '0');
	const milliseconds = Math.floor(date.getMilliseconds()).toString().padStart(3, '0');

	return `${dateStr} ${hours}:${minutes}:${seconds}.${milliseconds}`;
}

export function formatTimestampFull(nanos: number): string {
	// Convert nanoseconds to milliseconds for Date object
	const ms = nanos / 1_000_000;
	const date = new Date(ms);

	// Get the fractional part in microseconds and nanoseconds
	const microseconds = Math.floor((nanos / 1_000) % 1_000)
		.toString()
		.padStart(3, '0');
	const nanosecondsPart = (nanos % 1_000).toString().padStart(3, '0');

	// Format with full precision
	const dateStr = date.toLocaleDateString();
	const hours = date.getHours().toString().padStart(2, '0');
	const minutes = date.getMinutes().toString().padStart(2, '0');
	const seconds = date.getSeconds().toString().padStart(2, '0');
	const milliseconds = Math.floor(date.getMilliseconds()).toString().padStart(3, '0');

	return `${dateStr} ${hours}:${minutes}:${seconds}.${milliseconds}.${microseconds}.${nanosecondsPart}`;
}

export function getLevelColor(level: Level | null): string {
	if (!level) return 'secondary';

	switch (level) {
		case 'TRACE':
			return 'secondary';
		case 'DEBUG':
			return 'info';
		case 'INFO':
			return 'success';
		case 'WARN':
			return 'warning';
		case 'ERROR':
			return 'danger';
		case 'FATAL':
			return 'danger';
		default:
			return 'secondary';
	}
}

export function formatBytes(bytes: number | null): string {
	if (bytes === null) return 'N/A';
	if (bytes === 0) return '0 Bytes';

	const k = 1024;
	const sizes = ['Bytes', 'KB', 'MB', 'GB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));

	return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

export function formatDuration(startNanos: number, endNanos: number | null): string {
	if (!endNanos) return 'ongoing';

	const durationNanos = endNanos - startNanos;
	const durationMs = durationNanos / 1_000_000;

	if (durationMs < 1) {
		return `${(durationNanos / 1_000).toFixed(2)}μs`;
	} else if (durationMs < 1000) {
		return `${durationMs.toFixed(2)}ms`;
	} else {
		return `${(durationMs / 1000).toFixed(2)}s`;
	}
}
