import { invoke } from '@tauri-apps/api/core';
import type { WorkItem, CreateWorkItemRequest, UpdateWorkItemRequest } from './app.svelte';

// --- Work item state for the board ---

let workItems = $state<WorkItem[]>([]);
let isLoading = $state(false);
let error = $state<string | null>(null);

export type BoardSort = 'backlog' | 'title' | 'priority' | 'storyPoints' | 'remainingWork' | 'id';
export type BoardGrouping = 'hierarchy' | 'flat';

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
				new: groupItemsForColumn(workItems, 'new', 'hierarchy', 'backlog'),
				active: groupItemsForColumn(workItems, 'active', 'hierarchy', 'backlog'),
				done: groupItemsForColumn(workItems, 'done', 'hierarchy', 'backlog')
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
					new: groupItemsForColumn(filtered, 'new', view.groupBy, view.sortBy),
					active: groupItemsForColumn(filtered, 'active', view.groupBy, view.sortBy),
					done: groupItemsForColumn(filtered, 'done', view.groupBy, view.sortBy)
				}
			};
		},

		async fetchSprintItems(organization: string, project: string, team: string, iterationPath: string) {
			isLoading = true;
			error = null;
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
			try {
				const updated = await invoke<WorkItem>('move_work_item', {
					organization,
					project,
					id,
					workItemType,
					targetColumn
				});
				replaceItem(updated);

				// If moving to done and it's a PBI/Bug, cascade to children
				if (targetColumn === 'done' && (workItemType === 'Product Backlog Item' || workItemType === 'Bug')) {
					const updatedItems = await invoke<WorkItem[]>('mark_item_done_cascade', {
						organization,
						project,
						id,
						workItemType
					});
					for (const item of updatedItems) {
						replaceItem(item);
					}
				}

				if (workItemType === 'Task' && updated.parentId) {
					const parentUpdated = await invoke<WorkItem | null>('sync_parent_state', {
						organization,
						project,
						parentId: updated.parentId
					});

					if (parentUpdated) {
						replaceItem(parentUpdated);
					}
				}
			} catch (e) {
				error = `Failed to move item: ${e}`;
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
	groupBy: BoardGrouping,
	sortBy: BoardSort
): GroupedItem[] {
	const columnItems = sortWorkItems(allItems.filter((wi) => wi.boardColumn === column), sortBy);

	if (groupBy === 'flat') {
		return columnItems.map((item) => ({ item, children: [] }));
	}

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
