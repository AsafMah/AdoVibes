<script lang="ts">
	import { onMount } from 'svelte';
	import type { WorkItem, CreateWorkItemRequest, UpdateWorkItemRequest } from '$lib/stores/app.svelte';
	import type { GroupedItem } from '$lib/stores/workitems.svelte';
	import BoardColumn from './BoardColumn.svelte';
	import CreateItemDialog from './CreateItemDialog.svelte';
	import WorkItemDetail from './WorkItemDetail.svelte';
	import { handleBoardKeydown, getNextColumn, type Column } from '$lib/keyboard';

	interface Props {
		groupedByColumn: { new: GroupedItem[]; active: GroupedItem[]; done: GroupedItem[] };
		newItems: WorkItem[];
		activeItems: WorkItem[];
		doneItems: WorkItem[];
		iterationPath: string;
		onMoveItem: (id: number, workItemType: string, targetColumn: string) => void;
		onCreateItem: (request: CreateWorkItemRequest) => void;
		onUpdateItem: (request: UpdateWorkItemRequest) => void;
	}

	let {
		groupedByColumn,
		newItems,
		activeItems,
		doneItems,
		iterationPath,
		onMoveItem,
		onCreateItem,
		onUpdateItem
	}: Props = $props();

	let selectedItem = $state<WorkItem | null>(null);
	let detailItem = $state<WorkItem | null>(null);
	let showCreateDialog = $state(false);
	let createParent = $state<WorkItem | null>(null);

	// Keyboard navigation
	let focusedColumn = $state<Column>('new');

	onMount(() => {
		const handleCreateItem = () => {
			showCreateDialog = true;
			createParent = null;
		};

		window.addEventListener('create-item', handleCreateItem);

		return () => {
			window.removeEventListener('create-item', handleCreateItem);
		};
	});

	function getAllItemsInColumn(column: Column): WorkItem[] {
		switch (column) {
			case 'new': return newItems;
			case 'active': return activeItems;
			case 'done': return doneItems;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (showCreateDialog || detailItem) return;

		const action = handleBoardKeydown(e);
		if (!action) return;

		const columnItems = getAllItemsInColumn(focusedColumn);

		switch (action.type) {
			case 'navigate': {
				if (action.direction === 'up' || action.direction === 'down') {
					const currentIdx = selectedItem
						? columnItems.findIndex((wi) => wi.id === selectedItem!.id)
						: -1;
					let nextIdx: number;
					if (action.direction === 'down') {
						nextIdx = currentIdx < columnItems.length - 1 ? currentIdx + 1 : 0;
					} else {
						nextIdx = currentIdx > 0 ? currentIdx - 1 : columnItems.length - 1;
					}
					if (columnItems[nextIdx]) {
						selectedItem = columnItems[nextIdx];
						scrollToItem(columnItems[nextIdx].id);
					}
				} else {
					const nextCol = getNextColumn(focusedColumn, action.direction === 'left' ? 'left' : 'right');
					if (nextCol) {
						focusedColumn = nextCol;
						const items = getAllItemsInColumn(nextCol);
						if (items.length > 0) {
							selectedItem = items[0];
							scrollToItem(items[0].id);
						} else {
							selectedItem = null;
						}
					}
				}
				break;
			}
			case 'move': {
				if (!selectedItem) break;
				const nextCol = getNextColumn(focusedColumn, action.direction);
				if (nextCol) {
					onMoveItem(selectedItem.id, selectedItem.workItemType, nextCol);
					focusedColumn = nextCol;
					selectedItem = { ...selectedItem, boardColumn: nextCol };
				}
				break;
			}
			case 'open':
				if (selectedItem) detailItem = selectedItem;
				break;
			case 'create':
				showCreateDialog = true;
				createParent = null;
				break;
			case 'edit':
				if (selectedItem) detailItem = selectedItem;
				break;
			case 'done':
				if (selectedItem && selectedItem.boardColumn !== 'done') {
					onMoveItem(selectedItem.id, selectedItem.workItemType, 'done');
				}
				break;
			case 'escape':
				selectedItem = null;
				break;
		}
	}

	function scrollToItem(id: number) {
		const el = document.querySelector(`[data-item-id="${id}"]`);
		el?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
	}

	function handleSelectItem(item: WorkItem) {
		selectedItem = item;
		focusedColumn = item.boardColumn as Column;
	}

	function handleOpenItem(item: WorkItem) {
		detailItem = item;
	}

	function handleAddTask(parent: WorkItem) {
		createParent = parent;
		showCreateDialog = true;
	}

	function handleDropItem(itemId: number, targetColumn: string) {
		const allItems = [...newItems, ...activeItems, ...doneItems];
		const item = allItems.find((wi) => wi.id === itemId);
		if (item) {
			onMoveItem(item.id, item.workItemType, targetColumn);
		}
	}

	function handleCreateSubmit(request: CreateWorkItemRequest) {
		onCreateItem(request);
		showCreateDialog = false;
		createParent = null;
	}

	function handleUpdateSubmit(request: UpdateWorkItemRequest) {
		onUpdateItem(request);
		detailItem = null;
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex h-full gap-4 overflow-x-auto p-4">
	<BoardColumn
		title="New"
		column="new"
		groups={groupedByColumn.new}
		items={newItems}
		selectedItemId={selectedItem?.id}
		onSelectItem={handleSelectItem}
		onOpenItem={handleOpenItem}
		onAddTask={handleAddTask}
		onDropItem={handleDropItem}
	/>
	<BoardColumn
		title="Active"
		column="active"
		groups={groupedByColumn.active}
		items={activeItems}
		selectedItemId={selectedItem?.id}
		onSelectItem={handleSelectItem}
		onOpenItem={handleOpenItem}
		onAddTask={handleAddTask}
		onDropItem={handleDropItem}
	/>
	<BoardColumn
		title="Done"
		column="done"
		groups={groupedByColumn.done}
		items={doneItems}
		selectedItemId={selectedItem?.id}
		onSelectItem={handleSelectItem}
		onOpenItem={handleOpenItem}
		onAddTask={handleAddTask}
		onDropItem={handleDropItem}
	/>
</div>

<!-- Keyboard shortcut hint -->
<div class="fixed bottom-0 left-0 right-0 border-t border-surface-200 dark:border-surface-700 bg-surface-50 dark:bg-surface-900 px-4 py-1.5 text-xs text-surface-400 dark:text-surface-500">
	<span class="mr-4">↑↓ Navigate</span>
	<span class="mr-4">←→ Switch columns</span>
	<span class="mr-4"><kbd class="rounded border border-surface-300 dark:border-surface-600 bg-surface-100 dark:bg-surface-800 px-1">Ctrl+←/→</kbd> Move item</span>
	<span class="mr-4"><kbd class="rounded border border-surface-300 dark:border-surface-600 bg-surface-100 dark:bg-surface-800 px-1">Enter</kbd> Open</span>
	<span class="mr-4"><kbd class="rounded border border-surface-300 dark:border-surface-600 bg-surface-100 dark:bg-surface-800 px-1">N</kbd> New item</span>
	<span class="mr-4"><kbd class="rounded border border-surface-300 dark:border-surface-600 bg-surface-100 dark:bg-surface-800 px-1">D</kbd> Mark done</span>
	<span><kbd class="rounded border border-surface-300 dark:border-surface-600 bg-surface-100 dark:bg-surface-800 px-1">Esc</kbd> Deselect</span>
</div>

{#if showCreateDialog}
	<CreateItemDialog
		{iterationPath}
		parentId={createParent?.id}
		parentTitle={createParent?.title}
		onSubmit={handleCreateSubmit}
		onClose={() => { showCreateDialog = false; createParent = null; }}
	/>
{/if}

{#if detailItem}
	<WorkItemDetail
		item={detailItem}
		onUpdate={handleUpdateSubmit}
		onClose={() => detailItem = null}
	/>
{/if}
