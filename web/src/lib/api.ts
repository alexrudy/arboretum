// API client for Arboretum backend

export type Level = 'TRACE' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'FATAL';
export const Levels: Level[] = ['TRACE', 'DEBUG', 'INFO', 'WARN', 'ERROR', 'FATAL'];

export interface LogRecord {
	type: 'log';
	timestamp: number;
	service_name: string | null;
	level: Level | null;
	target: string | null;
	message: string | null;
	span_id: string | null;
	trace_id: string | null;
	attributes: Record<string, any> | null;
}

export interface SpanEvent {
	name: string;
	time: number;
	attributes?: Record<string, any>;
}

export interface SpanRecord {
	type: 'span';
	timestamp: number;
	trace_id: string;
	span_id: string;
	parent_span_id: string | null;
	service_name: string | null;
	name: string;
	kind: string | null;
	start_time: number;
	end_time: number | null;
	level: Level | null;
	target: string | null;
	attributes: Record<string, any> | null;
	events: SpanEvent[] | null;
	status: string | null;
}

export interface EventRecord {
	type: 'event';
	timestamp: number;
	span_id: string;
	trace_id: string;
	service_name: string | null;
	name: string;
	level: Level | null;
	target: string | null;
	attributes: Record<string, any> | null;
}

export type LogOrSpan = LogRecord | SpanRecord | EventRecord;

export interface PaginatedResponse<T> {
	records: T[];
	cursor: string | null;
}

export interface DatabaseStats {
	total_logs: number;
	total_spans: number;
	oldest_log_timestamp: number | null;
	newest_log_timestamp: number | null;
	oldest_span_timestamp: number | null;
	newest_span_timestamp: number | null;
	database_size_bytes: number | null;
}

export class ArboretumClient {
	private baseUrl: string;

	constructor(baseUrl: string = 'http://localhost:3333') {
		this.baseUrl = baseUrl;
	}

	async getRecords(params: {
		service_name?: string;
		target?: string;
		level?: string;
		cursor?: string;
		lookback?: number;
	}): Promise<PaginatedResponse<LogOrSpan>> {
		const queryParams = new URLSearchParams();
		if (params.service_name) queryParams.append('service_name', params.service_name);
		if (params.target) queryParams.append('target', params.target);
		if (params.level) queryParams.append('level', params.level);
		if (params.cursor) queryParams.append('cursor', params.cursor);

		if (params.lookback !== undefined) queryParams.append('lookback', params.lookback.toString());

		const url = `${this.baseUrl}/api/v1/records?${queryParams}`;
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error(`Failed to fetch records: ${response.statusText}`);
		}
		return response.json();
	}

	async getSpans(params: {
		service_name?: string;
		target?: string;
		trace_id?: string;
		span_id?: string;
	}): Promise<PaginatedResponse<SpanRecord>> {
		const queryParams = new URLSearchParams();
		if (params.service_name) queryParams.append('service_name', params.service_name);
		if (params.target) queryParams.append('target', params.target);
		if (params.trace_id) queryParams.append('trace_id', params.trace_id);
		if (params.span_id) queryParams.append('span_id', params.span_id);

		const url = `${this.baseUrl}/api/v1/spans?${queryParams}`;
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error(`Failed to fetch spans: ${response.statusText}`);
		}
		return response.json();
	}

	async getEvents(params: {
		service_name?: string;
		target?: string;
		trace_id?: string;
		span_id?: string;
		level?: string;
	}): Promise<PaginatedResponse<EventRecord>> {
		const queryParams = new URLSearchParams();
		if (params.service_name) queryParams.append('service_name', params.service_name);
		if (params.target) queryParams.append('target', params.target);
		if (params.trace_id) queryParams.append('trace_id', params.trace_id);
		if (params.span_id) queryParams.append('span_id', params.span_id);
		if (params.level) queryParams.append('level', params.level);

		const url = `${this.baseUrl}/api/v1/events?${queryParams}`;
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error(`Failed to fetch events: ${response.statusText}`);
		}
		return response.json();
	}

	async getMetadata(): Promise<DatabaseStats> {
		const url = `${this.baseUrl}/api/v1/metadata`;
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error(`Failed to fetch metadata: ${response.statusText}`);
		}
		return response.json();
	}
}

export const api = new ArboretumClient();
