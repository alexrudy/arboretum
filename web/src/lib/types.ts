import type { SpanRecord, EventRecord } from './api';

export interface TreeNode {
	span: SpanRecord;
	children: TreeNode[];
	events: EventRecord[];
}
