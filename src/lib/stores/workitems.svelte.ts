import { invoke } from '@tauri-apps/api/core';
import type { WorkItem, CreateWorkItemRequest, UpdateWorkItemRequest } from './app.svelte';

// --- Work item state for the board ---

let workItems = $state<WorkItem[]>([]);
let isLoading = $state(false);
let error = $state<string | null>(null);
let movingItemId = $state<number | null>(null);
let loadingSprintKey: string | null = null;
let cachedAuthHeader: string | null = null;
let cachedAuthHeaderExpiresAt = 0;
const AUTH_HEADER_TTL_MS = 5 * 60 * 1000;

export type BoardSort = 'backlog' | 'title' | 'priority' | 'storyPoints' | 'remainingWork' | 'id';
export type BoardGrouping = 'hierarchy';

export interface BoardViewOptions {
	search: string;
	type: string;
	assignee: string;
	mineOnly: boolean;
	currentUser?: string | null;
	sortBy: BoardSort;
	groupBy: BoardGrouping;
}

export interface BoardViewData {
	allItems: WorkItem[];
	newItems: WorkItem[];
	activeItems: WorkItem[];
	doneItems: WorkItem[];
	groupedByColumn: { new: GroupedItem[]; active: GroupedItem[]; done: GroupedItem[] };
}

export function getWorkItemsState() {
	return {
		get workItems() { return workItems; },
		get isLoading() { return isLoading; },
		get error() { return error; },
		get movingItemId() { return movingItemId; },
		get isMovePending() { return movingItemId !== null; },
		async primeAuthHeader(forceRefresh = false) {
			await getAuthHeader(forceRefresh);
		},
		get assignees() {
			return Array.from(new Set(workItems.map((wi) => wi.assignedTo?.trim()).filter(Boolean) as string[]))
				.sort((left, right) => left.localeCompare(right));
		},

		get newItems() {
			return workItems.filter((wi) => wi.boardColumn === 'new');
		},
		get activeItems() {
			return workItems.filter((wi) => wi.boardColumn === 'active');
		},
		get doneItems() {
			return workItems.filter((wi) => wi.boardColumn === 'done');
		},

		/** Group work items: PBIs/Bugs at top level, Tasks nested under their parent */
		get groupedByColumn() {
			const columns = {
				new: groupItemsForColumn(workItems, 'new', 'backlog'),
				active: groupItemsForColumn(workItems, 'active', 'backlog'),
				done: groupItemsForColumn(workItems, 'done', 'backlog')
			};
			return columns;
		},

		getBoardData(view: BoardViewOptions): BoardViewData {
			const filtered = sortWorkItems(filterWorkItems(workItems, view), view.sortBy);
			return {
				allItems: filtered,
				newItems: filtered.filter((wi) => wi.boardColumn === 'new'),
				activeItems: filtered.filter((wi) => wi.boardColumn === 'active'),
				doneItems: filtered.filter((wi) => wi.boardColumn === 'done'),
				groupedByColumn: {
					new: groupItemsForColumn(filtered, 'new', view.sortBy),
					active: groupItemsForColumn(filtered, 'active', view.sortBy),
					done: groupItemsForColumn(filtered, 'done', view.sortBy)
				}
			};
		},

		async fetchSprintItems(organization: string, project: string, team: string, iterationPath: string) {
			const sprintKey = `${organization}::${project}::${team}::${iterationPath}`;
			if (loadingSprintKey === sprintKey) {
				return;
			}

			isLoading = true;
			error = null;
			loadingSprintKey = sprintKey;
			try {
				workItems = await invoke<WorkItem[]>('get_sprint_work_items', {
					organization,
					project,
					team,
					iterationPath
				});
			} catch (e) {
				error = `Failed to load work items: ${e}`;
			} finally {
				if (loadingSprintKey === sprintKey) {
					loadingSprintKey = null;
				}
				isLoading = false;
			}
		},

		async moveItem(
			organization: string,
			project: string,
			id: number,
			workItemType: string,
			targetColumn: string
	) {
			if (movingItemId !== null) {
				return null;
			}

			const previousItem = workItems.find((wi) => wi.id === id);
			if (!previousItem || previousItem.boardColumn === targetColumn) {
				return null;
			}

			error = null;
			movingItemId = id;

			try {
				const updated = await moveWorkItemInBrowser(
					organization,
					project,
					id,
					workItemType,
					targetColumn
				);
				replaceItem(updated);

				if (workItemType === 'Product Backlog Item' || workItemType === 'Bug') {
					await syncLoadedChildTasksToColumn(organization, project, updated.id, targetColumn as WorkItem['boardColumn']);
				}

				if (workItemType === 'Task' && updated.parentId) {
					await syncParentStateFromLoadedItems(organization, project, updated.parentId);
				}
				return updated;
			} catch (e) {
				error = `Failed to move item: ${e}`;
				return null;
			} finally {
				movingItemId = null;
			}
		},

		async createItem(organization: string, project: string, request: CreateWorkItemRequest) {
			try {
				const created = await invoke<WorkItem>('create_work_item', {
					organization,
					project,
					request
				});
				workItems = [...workItems, created];
				return created;
			} catch (e) {
				error = `Failed to create item: ${e}`;
				return null;
			}
		},

		async updateItem(organization: string, project: string, request: UpdateWorkItemRequest) {
			try {
				const updated = await invoke<WorkItem>('update_work_item', {
					organization,
					project,
					request
				});
				replaceItem(updated);
				return updated;
			} catch (e) {
				error = `Failed to update item: ${e}`;
				return null;
			}
		},

		clearError() {
			error = null;
		}
	};
}

function replaceItem(updated: WorkItem) {
	const idx = workItems.findIndex((wi) => wi.id === updated.id);
	if (idx >= 0) {
		workItems = [...workItems.slice(0, idx), updated, ...workItems.slice(idx + 1)];
	}
}

interface AdoRelation {
	rel?: string;
	url?: string;
}

interface AdoAssignedTo {
	displayName?: string;
	uniqueName?: string;
}

interface AdoWorkItemResponse {
	id: number;
	fields?: Record<string, unknown>;
	relations?: AdoRelation[];
}

async function moveWorkItemInBrowser(
	organization: string,
	project: string,
	id: number,
	workItemType: string,
	targetColumn: string
): Promise<WorkItem> {
	const authHeader = await getAuthHeader();
	const adoState = columnToAdoState(workItemType, targetColumn);
	return patchWorkItemState(organization, project, id, authHeader, adoState, true);
}

async function syncParentStateFromLoadedItems(
	organization: string,
	project: string,
	parentId: number
) {
	const parent = workItems.find((item) => item.id === parentId);
	if (!parent) {
		return null;
	}

	const childTasks = workItems.filter(
		(item) => item.parentId === parentId && item.workItemType === 'Task'
	);
	if (childTasks.length === 0) {
		return null;
	}

	const targetColumn: WorkItem['boardColumn'] = childTasks.every((item) => item.boardColumn === 'done')
		? 'done'
		: childTasks.some((item) => item.boardColumn !== 'new')
			? 'active'
			: 'new';

	if (parent.boardColumn === targetColumn) {
		return parent;
	}

	const updatedParent = await moveWorkItemInBrowser(
		organization,
		project,
		parent.id,
		parent.workItemType,
		targetColumn
	);
	replaceItem(updatedParent);
	return updatedParent;
}

async function syncLoadedChildTasksToColumn(
	organization: string,
	project: string,
	parentId: number,
	targetColumn: WorkItem['boardColumn']
) {
	const childTasks = workItems.filter(
		(item) => item.parentId === parentId && item.workItemType === 'Task' && item.boardColumn !== targetColumn
	);
	if (childTasks.length === 0) {
		return [];
	}

	const authHeader = await getAuthHeader();
	const results = await Promise.allSettled(
		childTasks.map((child) =>
			patchWorkItemState(
				organization,
				project,
				child.id,
				authHeader,
				columnToAdoState('Task', targetColumn),
				true
			)
		)
	);

	const failedIds: number[] = [];
	for (let index = 0; index < results.length; index += 1) {
		const result = results[index];
		if (result.status === 'fulfilled') {
			replaceItem(result.value);
		} else {
			failedIds.push(childTasks[index].id);
		}
	}

	if (failedIds.length > 0) {
		throw new Error(`Failed to sync child tasks: ${failedIds.join(', ')}`);
	}

	return childTasks;
}

async function getAuthHeader(forceRefresh = false): Promise<string> {
	const now = Date.now();
	if (!forceRefresh && cachedAuthHeader && now < cachedAuthHeaderExpiresAt) {
		return cachedAuthHeader;
	}

	cachedAuthHeader = await invoke<string>('get_auth_header');
	cachedAuthHeaderExpiresAt = now + AUTH_HEADER_TTL_MS;
	return cachedAuthHeader;
}

async function patchWorkItemState(
	organization: string,
	project: string,
	id: number,
	authHeader: string,
	adoState: string,
	allowRetry: boolean
): Promise<WorkItem> {
	const response = await fetch(
		`https://dev.azure.com/${encodeURIComponent(organization)}/${encodeURIComponent(project)}/_apis/wit/workitems/${id}?api-version=7.1`,
		{
			method: 'PATCH',
			headers: {
				Authorization: authHeader,
				'Content-Type': 'application/json-patch+json',
				Accept: 'application/json'
			},
			body: JSON.stringify([
				{
					op: 'replace',
					path: '/fields/System.State',
					value: adoState
				}
			])
		}
	);

	if (response.status === 401 && allowRetry) {
		const refreshedAuthHeader = await getAuthHeader(true);
		return patchWorkItemState(organization, project, id, refreshedAuthHeader, adoState, false);
	}

	if (!response.ok) {
		const body = await response.text();
		throw new Error(`HTTP ${response.status} ${response.statusText} — ${body}`);
	}

	const raw = (await response.json()) as AdoWorkItemResponse;
	return parseAdoWorkItem(raw);
}

function columnToAdoState(workItemType: string, column: string): string {
	switch (column) {
		case 'new':
			return workItemType === 'Task' ? 'To Do' : 'New';
		case 'active':
			if (workItemType === 'Task') return 'In Progress';
			if (workItemType === 'Product Backlog Item' || workItemType === 'Bug') return 'Committed';
			return 'Active';
		case 'done':
			return 'Done';
		default:
			return 'New';
	}
}

function mapStateToColumn(workItemType: string, state: string): WorkItem['boardColumn'] {
	const stateLower = state.toLowerCase();
	switch (workItemType) {
		case 'Task':
			if (stateLower === 'to do' || stateLower === 'new') return 'new';
			if (stateLower === 'done' || stateLower === 'closed' || stateLower === 'removed') return 'done';
			return 'active';
		case 'Product Backlog Item':
		case 'Bug':
		case 'Epic':
		case 'Feature':
			if (stateLower === 'new') return 'new';
			if (stateLower === 'done' || stateLower === 'closed' || stateLower === 'removed') return 'done';
			return 'active';
		default:
			if (stateLower === 'new' || stateLower === 'to do') return 'new';
			if (stateLower === 'done' || stateLower === 'closed' || stateLower === 'removed') return 'done';
			return 'active';
	}
}

function parseAdoWorkItem(raw: AdoWorkItemResponse): WorkItem {
	const fields = raw.fields ?? {};
	const title = getFieldString(fields, 'System.Title');
	const state = getFieldString(fields, 'System.State');
	const workItemType = getFieldString(fields, 'System.WorkItemType');
	const assigned = fields['System.AssignedTo'] as AdoAssignedTo | undefined;
	const parentRelation = raw.relations?.find((relation) => relation.rel === 'System.LinkTypes.Hierarchy-Reverse');
	const parentId = parentRelation?.url ? parseInt(parentRelation.url.split('/').pop() ?? '', 10) : undefined;

	return {
		id: raw.id,
		title,
		state,
		workItemType,
		assignedTo: assigned?.displayName ?? assigned?.uniqueName,
		iterationPath: getOptionalString(fields, 'System.IterationPath'),
		areaPath: getOptionalString(fields, 'System.AreaPath'),
		priority: getOptionalNumber(fields, 'Microsoft.VSTS.Common.Priority'),
		storyPoints: getOptionalNumber(fields, 'Microsoft.VSTS.Scheduling.StoryPoints'),
		remainingWork: getOptionalNumber(fields, 'Microsoft.VSTS.Scheduling.RemainingWork'),
		description: getOptionalString(fields, 'System.Description'),
		tags: getOptionalString(fields, 'System.Tags'),
		parentId: Number.isFinite(parentId) ? parentId : undefined,
		boardColumn: mapStateToColumn(workItemType, state)
	};
}

function getFieldString(fields: Record<string, unknown>, key: string): string {
	const value = fields[key];
	return typeof value === 'string' ? value : '';
}

function getOptionalString(fields: Record<string, unknown>, key: string): string | undefined {
	const value = fields[key];
	return typeof value === 'string' ? value : undefined;
}

function getOptionalNumber(fields: Record<string, unknown>, key: string): number | undefined {
	const value = fields[key];
	return typeof value === 'number' ? value : undefined;
}

function filterWorkItems(items: WorkItem[], view: BoardViewOptions) {
	const search = view.search.trim().toLowerCase();
	const currentUser = view.currentUser?.trim().toLowerCase();

	return items.filter((item) => {
		if (search) {
			const haystack = [
				item.title,
				String(item.id),
				item.assignedTo,
				item.workItemType,
				item.tags
			].filter(Boolean).join(' ').toLowerCase();

			if (!haystack.includes(search)) {
				return false;
			}
		}

		if (view.type !== 'all' && item.workItemType !== view.type) {
			return false;
		}

		if (view.assignee !== 'all' && (item.assignedTo ?? 'Unassigned') !== view.assignee) {
			return false;
		}

		if (view.mineOnly) {
			if (!currentUser || item.assignedTo?.trim().toLowerCase() !== currentUser) {
				return false;
			}
		}

		return true;
	});
}

function sortWorkItems(items: WorkItem[], sortBy: BoardSort) {
	return [...items].sort((left, right) => compareWorkItems(left, right, sortBy));
}

function compareWorkItems(left: WorkItem, right: WorkItem, sortBy: BoardSort) {
	switch (sortBy) {
		case 'title':
			return left.title.localeCompare(right.title) || left.id - right.id;
		case 'priority':
			return compareNullableNumber(left.priority, right.priority) || left.id - right.id;
		case 'storyPoints':
			return compareNullableNumber(left.storyPoints, right.storyPoints) || left.id - right.id;
		case 'remainingWork':
			return compareNullableNumber(left.remainingWork, right.remainingWork) || left.id - right.id;
		case 'id':
			return left.id - right.id;
		case 'backlog':
		default:
			return compareNullableNumber(left.priority, right.priority) || left.id - right.id;
	}
}

function compareNullableNumber(left?: number, right?: number) {
	if (left == null && right == null) return 0;
	if (left == null) return 1;
	if (right == null) return -1;
	return left - right;
}

export interface GroupedItem {
	item: WorkItem;
	children: WorkItem[];
}

function isParentType(workItem: WorkItem) {
	return workItem.workItemType === 'Product Backlog Item'
		|| workItem.workItemType === 'Bug'
		|| workItem.workItemType === 'Epic'
		|| workItem.workItemType === 'Feature';
}

function groupItemsForColumn(
	allItems: WorkItem[],
	column: WorkItem['boardColumn'],
	sortBy: BoardSort
): GroupedItem[] {
	const columnItems = sortWorkItems(allItems.filter((wi) => wi.boardColumn === column), sortBy);

	const allItemsById = new Map(allItems.map((wi) => [wi.id, wi]));
	const groups = new Map<number, GroupedItem>();
	const orderedGroups: GroupedItem[] = [];

	for (const item of columnItems) {
		if (isParentType(item)) {
			if (!groups.has(item.id)) {
				const group = { item, children: [] };
				groups.set(item.id, group);
				orderedGroups.push(group);
			}
			continue;
		}

		if (item.workItemType !== 'Task' || !item.parentId) {
			orderedGroups.push({ item, children: [] });
			continue;
		}

		const parent = allItemsById.get(item.parentId);
		if (!parent || !isParentType(parent) || parent.boardColumn !== column) {
			orderedGroups.push({ item, children: [] });
			continue;
		}

		let group = groups.get(parent.id);
		if (!group) {
			group = { item: parent, children: [] };
			groups.set(parent.id, group);
			orderedGroups.push(group);
		}

		group.children.push(item);
	}

	for (const group of orderedGroups) {
		group.children = sortWorkItems(group.children, sortBy);
	}

	return orderedGroups;
}
