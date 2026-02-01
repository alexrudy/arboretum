import type { SpanRecord, EventRecord, LogOrSpan } from './api';

export interface TreeNode {
	span: SpanRecord;
	children: TreeNode[];
	events: EventRecord[];
}

export function getRecordID(record: LogOrSpan): string {
	if (record.type === 'log') {
		return `log-${record.timestamp}`;
	} else if (record.type === 'span') {
		return `span-${record.span_id}`;
	} else {
		return `event-${record.timestamp}-${record.span_id}`;
	}
}
