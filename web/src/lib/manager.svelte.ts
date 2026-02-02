import {
	api,
	type ArboretumClient,
	type GetRecordParams,
	type Level,
	type LogOrSpan,
	type PaginatedResponse
} from './api';

export class ArboretumManager {
	private api: ArboretumClient;
	private latest: string | null = $state(null);
	private max_records: number = 2000;
	records: LogOrSpan[] = $state([]);
	loading: boolean = $state(false);

	service_name: string | null = $state(null);
	target: string | null = $state(null);
	message: string | null = $state(null);
	level: Level | null = $state(null);
	trace_id: string | null = $state(null);

	constructor(api: ArboretumClient) {
		this.api = api;
		this.latest = null;
	}

	async loadRecords({ replace }: { replace?: boolean }) {
		if (this.loading) {
			return;
		}
		this.loading = true;
		try {
			let response: PaginatedResponse<LogOrSpan>;

			if (!replace && this.latest) {
				response = await this.api.getRecords({
					since: this.latest,
					service_name: this.service_name!,
					target: this.target!,
					message: this.message!,
					level: this.level!,
					trace_id: this.trace_id!
				});
			} else {
				response = await this.api.getRecords({
					lookback: 10,
					service_name: this.service_name!,
					target: this.target!,
					message: this.message!,
					level: this.level!,
					trace_id: this.trace_id!
				});
			}

			if (!replace && response.records.length > 0 && this.records.length > 0) {
				// Check if there are newer records than what we have
				const latestTimestamp = response.records[response.records.length - 1].timestamp;
				const currentLatestTimestamp = this.records
					? this.records[this.records.length - 1].timestamp
					: 0;

				if (latestTimestamp > currentLatestTimestamp) {
					// Find new records that we don't have yet
					const newRecords = response.records.filter(
						(newRec) => newRec.timestamp > currentLatestTimestamp
					);

					if (newRecords.length > 0) {
						this.records = [...this.records, ...newRecords];
					}
					console.log('New records found:', newRecords.length);
				}
			} else {
				this.records = response.records;
			}

			if (this.records.length > this.max_records) {
				this.records = this.records.slice(-this.max_records);
			}
		} catch (e) {
			console.error('Failed to check for new records:', e);
		} finally {
			this.loading = false;
		}
	}

	hierarchy(predicate: (record: LogOrSpan) => boolean): HierarchicalRecord[] {
		const tree = buildHierarchy(this.records.filter(predicate));
		return flattenHierarchy(tree);
	}
}

// Build hierarchical structure from flat records
export interface HierarchicalRecord {
	record: LogOrSpan;
	children: HierarchicalRecord[];
	depth: number;
}

function buildHierarchy(flatRecords: LogOrSpan[]): HierarchicalRecord[] {
	const spanMap = new Map<string, HierarchicalRecord>();
	const roots: HierarchicalRecord[] = [];

	// First pass: create nodes for all spans
	for (const record of flatRecords) {
		if (record.type === 'span') {
			spanMap.set(record.span_id, {
				record,
				children: [],
				depth: 0
			});
		}
	}

	// Second pass: attach events to their parent spans and build hierarchy
	for (const record of flatRecords) {
		if (record.type === 'event') {
			const parent = spanMap.get(record.span_id);
			if (parent) {
				parent.children.push({
					record,
					children: [],
					depth: parent.depth + 1
				});
			} else {
				// Orphaned event - add as root
				roots.push({ record, children: [], depth: 0 });
			}
		} else if (record.type === 'span') {
			const node = spanMap.get(record.span_id)!;
			if (record.parent_span_id) {
				const parent = spanMap.get(record.parent_span_id);
				if (parent) {
					node.depth = parent.depth + 1;
					parent.children.push(node);
				} else {
					// Parent not in current view - add as root
					roots.push(node);
				}
			} else {
				// No parent - this is a root span
				roots.push(node);
			}
		} else if (record.type === 'log') {
			// Logs are always roots in this view
			roots.push({ record, children: [], depth: 0 });
		}
	}

	// Sort children by timestamp within each parent
	function sortChildren(node: HierarchicalRecord) {
		node.children.sort((a, b) => a.record.timestamp - b.record.timestamp);
		node.children.forEach(sortChildren);
	}
	roots.forEach(sortChildren);

	return roots;
}

// Flatten hierarchy back to a list for rendering
function flattenHierarchy(hierarchy: HierarchicalRecord[]): HierarchicalRecord[] {
	const result: HierarchicalRecord[] = [];
	function traverse(node: HierarchicalRecord) {
		result.push(node);
		node.children.forEach(traverse);
	}
	hierarchy.forEach(traverse);
	return result;
}
