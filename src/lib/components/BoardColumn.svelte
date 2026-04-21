<script lang="ts">
	import type { WorkItem } from '$lib/stores/app.svelte';
	import type { GroupedItem } from '$lib/stores/workitems.svelte';
	import PbiGroup from './PbiGroup.svelte';
	import WorkItemCard from './WorkItemCard.svelte';
	import { dndzone } from 'svelte-dnd-action';

	interface Props {
		title: string;
		column: 'new' | 'active' | 'done';
		groups: GroupedItem[];
		items: WorkItem[];
		selectedItemId?: number | null;
		onSelectItem?: (item: WorkItem) => void;
		onOpenItem?: (item: WorkItem) => void;
		onAddTask?: (parent: WorkItem) => void;
		onDropItem?: (itemId: number, targetColumn: string) => void;
	}

	let { title, column, groups, items, selectedItemId = null, onSelectItem, onOpenItem, onAddTask, onDropItem }: Props = $props();

	const columnColors: Record<string, string> = {
		new: 'border-t-blue-500',
		active: 'border-t-amber-500',
		done: 'border-t-green-500'
	};

	const columnBg: Record<string, string> = {
		new: 'bg-blue-50 dark:bg-blue-950/30',
		active: 'bg-amber-50 dark:bg-amber-950/30',
		done: 'bg-green-50 dark:bg-green-950/30'
	};

	// For dnd - create a flat draggable list of item IDs
	let dragItems = $derived(items.map(wi => ({ id: wi.id, item: wi })));

	function handleDndConsider(e: CustomEvent<{ items: Array<{ id: number; item: WorkItem }> }>) {
		// Update local items for visual feedback during drag
	}

	function handleDndFinalize(e: CustomEvent<{ items: Array<{ id: number; item: WorkItem }> }>) {
		const droppedItems = e.detail.items;
		// Find items that were dragged into this column from another column
		for (const dItem of droppedItems) {
			if (dItem.item && dItem.item.boardColumn !== column) {
				onDropItem?.(dItem.id, column);
			}
		}
	}
</script>

<div class="flex h-full min-w-[320px] max-w-[400px] flex-1 flex-col rounded-lg border-t-4 {columnColors[column]} bg-surface-50 dark:bg-surface-900 shadow-sm">
	<div class="flex items-center justify-between px-4 py-3 {columnBg[column]}">
		<h2 class="text-sm font-semibold uppercase tracking-wide text-surface-700 dark:text-surface-200">{title}</h2>
		<span class="rounded-full bg-surface-200 dark:bg-surface-700 px-2 py-0.5 text-xs font-medium text-surface-600 dark:text-surface-300">
			{items.length}
		</span>
	</div>

	<div
		class="board-column flex-1 overflow-y-auto p-2"
		use:dndzone={{ items: dragItems, dropTargetStyle: { outline: '2px solid #6366f1', borderRadius: '0.5rem' } }}
		onconsider={handleDndConsider}
		onfinalize={handleDndFinalize}
	>
		{#each groups as group (group.item.id)}
			{#if group.children.length > 0}
				<div class="mb-2">
					<PbiGroup
						parent={group.item}
						children={group.children}
						{selectedItemId}
						{onSelectItem}
						{onOpenItem}
						{onAddTask}
					/>
				</div>
			{:else if group.item.workItemType === 'Task'}
				<div class="mb-2">
					<WorkItemCard
						item={group.item}
						isSelected={selectedItemId === group.item.id}
						onSelect={() => onSelectItem?.(group.item)}
						onOpen={() => onOpenItem?.(group.item)}
					/>
				</div>
			{:else}
				<div class="mb-2">
					<PbiGroup
						parent={group.item}
						children={[]}
						{selectedItemId}
						{onSelectItem}
						{onOpenItem}
						{onAddTask}
					/>
				</div>
			{/if}
		{/each}

		{#if groups.length === 0}
			<div class="flex h-24 items-center justify-center rounded-lg border-2 border-dashed border-surface-300 dark:border-surface-600 text-sm text-surface-400 dark:text-surface-500">
				No items
			</div>
		{/if}
	</div>
</div>
