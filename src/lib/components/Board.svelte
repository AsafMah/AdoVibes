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
		onMoveItem: (id: number, workItemType: string, targetColumn: string) => Promise<WorkItem | null>;
		onCreateItem: (request: CreateWorkItemRequest) => void;
		onUpdateItem: (request: UpdateWorkItemRequest) => void;
		isMovePending: boolean;
		movingItemId: number | null;
	}

	let {
		groupedByColumn,
		newItems,
		activeItems,
		doneItems,
		iterationPath,
		onMoveItem,
		onCreateItem,
		onUpdateItem,
		isMovePending,
		movingItemId
	}: Props = $props();

	let selectedItem = $state<WorkItem | null>(null);
	let detailItem = $state<WorkItem | null>(null);
	let showCreateDialog = $state(false);
	let createParent = $state<WorkItem | null>(null);
	let dragState = $state<{
		item: WorkItem;
		pointerId: number;
		startX: number;
		startY: number;
		currentX: number;
		currentY: number;
		offsetX: number;
		offsetY: number;
		width: number;
		dragging: boolean;
		originColumn: Column;
	} | null>(null);
	let hoveredColumn = $state<Column | null>(null);

	// Keyboard navigation
	let focusedColumn = $state<Column>('new');
	const DRAG_THRESHOLD_PX = 6;
	const dragTypeColors: Record<string, string> = {
		'Product Backlog Item': 'bg-blue-500',
		'Bug': 'bg-red-500',
		'Task': 'bg-yellow-500',
		'Epic': 'bg-purple-500',
		'Feature': 'bg-green-500'
	};
	const dragTypeLabels: Record<string, string> = {
		'Product Backlog Item': 'PBI',
		'Bug': 'Bug',
		'Task': 'Task',
		'Epic': 'Epic',
		'Feature': 'Feature'
	};

	onMount(() => {
		const handleCreateItem = () => {
			showCreateDialog = true;
			createParent = null;
		};

		window.addEventListener('create-item', handleCreateItem);

		return () => {
			window.removeEventListener('create-item', handleCreateItem);
			stopPointerDrag();
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
				if (!selectedItem || isMovePending) break;
				const nextCol = getNextColumn(focusedColumn, action.direction);
				if (nextCol) {
					void moveSelectedItem(selectedItem, nextCol);
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
				if (selectedItem && selectedItem.boardColumn !== 'done' && !isMovePending) {
					void moveSelectedItem(selectedItem, 'done');
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

	function handleItemPointerDown(event: PointerEvent, item: WorkItem) {
		if (isMovePending || event.button !== 0) {
			return;
		}

		const target = event.currentTarget as HTMLElement | null;
		if (!target) {
			return;
		}

		const rect = target.getBoundingClientRect();
		selectedItem = item;
		focusedColumn = item.boardColumn as Column;
		dragState = {
			item,
			pointerId: event.pointerId,
			startX: event.clientX,
			startY: event.clientY,
			currentX: event.clientX,
			currentY: event.clientY,
			offsetX: event.clientX - rect.left,
			offsetY: event.clientY - rect.top,
			width: rect.width,
			dragging: false,
			originColumn: item.boardColumn as Column
		};
		window.addEventListener('pointermove', handleGlobalPointerMove, true);
		window.addEventListener('pointerup', handleGlobalPointerUp, true);
		window.addEventListener('pointercancel', handleGlobalPointerCancel, true);
	}

	function handleDropItem(itemId: number, targetColumn: string) {
		if (isMovePending) {
			return;
		}

		const allItems = [...newItems, ...activeItems, ...doneItems];
		const item = allItems.find((wi) => wi.id === itemId);
		if (item) {
			void moveSelectedItem(item, targetColumn);
		}
	}

	async function moveSelectedItem(item: WorkItem, targetColumn: string) {
		const updated = await onMoveItem(item.id, item.workItemType, targetColumn);
		if (!updated) {
			return;
		}

		focusedColumn = updated.boardColumn as Column;
		selectedItem = updated;
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

	function handleGlobalPointerMove(event: PointerEvent) {
		if (!dragState || event.pointerId !== dragState.pointerId) {
			return;
		}

		const movedFarEnough =
			Math.abs(event.clientX - dragState.startX) >= DRAG_THRESHOLD_PX ||
			Math.abs(event.clientY - dragState.startY) >= DRAG_THRESHOLD_PX;
		const dragging = dragState.dragging || movedFarEnough;

		dragState = {
			...dragState,
			currentX: event.clientX,
			currentY: event.clientY,
			dragging
		};

		if (dragging) {
			event.preventDefault();
			document.body.style.userSelect = 'none';
			document.body.style.cursor = 'grabbing';
			hoveredColumn = getColumnFromPoint(event.clientX, event.clientY);
		}
	}

	function handleGlobalPointerUp(event: PointerEvent) {
		if (!dragState || event.pointerId !== dragState.pointerId) {
			return;
		}

		const activeDrag = dragState;
		const targetColumn = hoveredColumn;
		stopPointerDrag();
		if (activeDrag.dragging && targetColumn && targetColumn !== activeDrag.originColumn) {
			handleDropItem(activeDrag.item.id, targetColumn);
		}
	}

	function handleGlobalPointerCancel(event: PointerEvent) {
		if (!dragState || event.pointerId !== dragState.pointerId) {
			return;
		}

		stopPointerDrag();
	}

	function stopPointerDrag() {
		window.removeEventListener('pointermove', handleGlobalPointerMove, true);
		window.removeEventListener('pointerup', handleGlobalPointerUp, true);
		window.removeEventListener('pointercancel', handleGlobalPointerCancel, true);
		document.body.style.userSelect = '';
		document.body.style.cursor = '';
		dragState = null;
		hoveredColumn = null;
	}

	function getColumnFromPoint(clientX: number, clientY: number): Column | null {
		const element = document.elementFromPoint(clientX, clientY)?.closest<HTMLElement>('[data-board-column]');
		const column = element?.dataset.boardColumn;
		return column === 'new' || column === 'active' || column === 'done' ? column : null;
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex h-full gap-4 overflow-x-auto p-4">
	<BoardColumn
		title="New"
		column="new"
		groups={groupedByColumn.new}
		items={newItems}
		draggingItemId={dragState?.dragging ? dragState.item.id : null}
		dropTargetColumn={dragState?.dragging ? hoveredColumn : null}
		selectedItemId={selectedItem?.id}
		onSelectItem={handleSelectItem}
		onOpenItem={handleOpenItem}
		onAddTask={handleAddTask}
		onItemPointerDown={handleItemPointerDown}
		movingItemId={movingItemId}
		dragEnabled={!isMovePending}
	/>
	<BoardColumn
		title="Active"
		column="active"
		groups={groupedByColumn.active}
		items={activeItems}
		draggingItemId={dragState?.dragging ? dragState.item.id : null}
		dropTargetColumn={dragState?.dragging ? hoveredColumn : null}
		selectedItemId={selectedItem?.id}
		onSelectItem={handleSelectItem}
		onOpenItem={handleOpenItem}
		onAddTask={handleAddTask}
		onItemPointerDown={handleItemPointerDown}
		movingItemId={movingItemId}
		dragEnabled={!isMovePending}
	/>
	<BoardColumn
		title="Done"
		column="done"
		groups={groupedByColumn.done}
		items={doneItems}
		draggingItemId={dragState?.dragging ? dragState.item.id : null}
		dropTargetColumn={dragState?.dragging ? hoveredColumn : null}
		selectedItemId={selectedItem?.id}
		onSelectItem={handleSelectItem}
		onOpenItem={handleOpenItem}
		onAddTask={handleAddTask}
		onItemPointerDown={handleItemPointerDown}
		movingItemId={movingItemId}
		dragEnabled={!isMovePending}
	/>
</div>

{#if dragState?.dragging}
	<div
		class="pointer-events-none fixed z-50"
		style={`left: ${dragState.currentX - dragState.offsetX}px; top: ${dragState.currentY - dragState.offsetY}px; width: ${dragState.width}px;`}
	>
		<div class="rounded-md border border-primary-300 bg-white/95 p-3 shadow-2xl dark:border-primary-700 dark:bg-surface-900/95">
			<div class="flex items-start gap-2">
				<span class="mt-0.5 inline-block h-2 w-2 shrink-0 rounded-full {dragTypeColors[dragState.item.workItemType] || 'bg-gray-500'}"></span>
				<div class="min-w-0">
					<div class="flex items-center gap-1.5">
						<span class="text-xs font-medium text-surface-500 dark:text-surface-400">
							{dragTypeLabels[dragState.item.workItemType] || dragState.item.workItemType}
						</span>
						<span class="text-xs text-surface-400 dark:text-surface-500">#{dragState.item.id}</span>
					</div>
					<p class="mt-0.5 line-clamp-2 text-sm font-medium text-surface-800 dark:text-surface-100">
						{dragState.item.title}
					</p>
				</div>
			</div>
		</div>
	</div>
{/if}

{#if isMovePending}
	<div class="fixed right-4 bottom-12 rounded-lg border border-primary-200 bg-primary-50 px-3 py-2 text-sm text-primary-700 shadow-sm dark:border-primary-900/40 dark:bg-primary-950/60 dark:text-primary-300">
		Saving move...
	</div>
{/if}

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
