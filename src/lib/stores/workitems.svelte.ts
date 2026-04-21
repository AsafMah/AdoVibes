import { invoke } from '@tauri-apps/api/core';
import type { WorkItem, CreateWorkItemRequest, UpdateWorkItemRequest } from './app.svelte';

// --- Work item state for the board ---

let workItems = $state<WorkItem[]>([]);
let isLoading = $state(false);
let error = $state<string | null>(null);

export function getWorkItemsState() {
	return {
		get workItems() { return workItems; },
		get isLoading() { return isLoading; },
		get error() { return error; },

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
				new: groupItemsForColumn(workItems, 'new'),
				active: groupItemsForColumn(workItems, 'active'),
				done: groupItemsForColumn(workItems, 'done')
			};
			return columns;
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

				// If moving a task to done, check if parent should also be done
				if (targetColumn === 'done' && workItemType === 'Task') {
					const item = workItems.find((wi) => wi.id === id);
					if (item?.parentId) {
						const parentUpdated = await invoke<WorkItem | null>('check_and_complete_parent', {
							organization,
							project,
							parentId: item.parentId
						});
						if (parentUpdated) {
							replaceItem(parentUpdated);
						}
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

function groupItemsForColumn(allItems: WorkItem[], column: WorkItem['boardColumn']): GroupedItem[] {
	const columnItems = allItems.filter((wi) => wi.boardColumn === column);
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
		if (!parent || !isParentType(parent)) {
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

	return orderedGroups;
}
