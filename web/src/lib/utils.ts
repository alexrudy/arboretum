import type { Level } from './api';

export function formatTimestamp(nanos: number): string {
	// Convert nanoseconds to milliseconds
	const ms = Math.floor(nanos / 1_000_000);
	const date = new Date(ms);
	return date.toLocaleString();
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

	return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
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
