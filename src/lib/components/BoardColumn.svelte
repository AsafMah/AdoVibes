<script lang="ts">
	import { flip } from 'svelte/animate';
	import type { WorkItem } from '$lib/stores/app.svelte';
	import type { GroupedItem } from '$lib/stores/workitems.svelte';
	import PbiGroup from './PbiGroup.svelte';
	import WorkItemCard from './WorkItemCard.svelte';

	interface Props {
		title: string;
		column: 'new' | 'active' | 'done';
		groups: GroupedItem[];
		items: WorkItem[];
		dragEnabled?: boolean;
		movingItemId?: number | null;
		draggingItemId?: number | null;
		dropTargetColumn?: 'new' | 'active' | 'done' | null;
		selectedItemId?: number | null;
		onSelectItem?: (item: WorkItem) => void;
		onOpenItem?: (item: WorkItem) => void;
		onAddTask?: (parent: WorkItem) => void;
		onItemPointerDown?: (event: PointerEvent, item: WorkItem) => void;
	}

	let {
		title,
		column,
		groups,
		items,
		dragEnabled = false,
		movingItemId = null,
		draggingItemId = null,
		dropTargetColumn = null,
		selectedItemId = null,
		onSelectItem,
		onOpenItem,
		onAddTask,
		onItemPointerDown
	}: Props = $props();

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

	const flipDurationMs = 150;
</script>

<div
	class="flex h-full min-w-[320px] max-w-[400px] flex-1 flex-col rounded-lg border-t-4 {columnColors[column]} bg-surface-50 dark:bg-surface-900 shadow-sm transition-all
		{dropTargetColumn === column ? 'ring-2 ring-primary-400 ring-offset-2 dark:ring-offset-surface-950 scale-[1.01]' : ''}"
	data-board-column={column}
>
	<div class="flex items-center justify-between px-4 py-3 {columnBg[column]}">
		<h2 class="text-sm font-semibold uppercase tracking-wide text-surface-700 dark:text-surface-200">{title}</h2>
		<span class="rounded-full bg-surface-200 dark:bg-surface-700 px-2 py-0.5 text-xs font-medium text-surface-600 dark:text-surface-300">
			{items.length}
		</span>
	</div>

	{#if dragEnabled}
		<div class="board-column flex-1 overflow-y-auto p-2" role="presentation">
			{#each groups as group (group.item.id)}
				<div class="mb-2" role="presentation" animate:flip={{ duration: flipDurationMs }}>
					{#if group.children.length > 0}
						<PbiGroup
							parent={group.item}
							children={group.children}
							{movingItemId}
							{draggingItemId}
							{selectedItemId}
							isParentDraggable={movingItemId === null}
							{onSelectItem}
							{onOpenItem}
							{onAddTask}
							onParentPointerDown={onItemPointerDown}
							onTaskPointerDown={onItemPointerDown}
						/>
					{:else if group.item.workItemType === 'Task'}
						<WorkItemCard
							item={group.item}
							isBusy={movingItemId === group.item.id}
							isDragging={draggingItemId === group.item.id}
							isDraggable={movingItemId === null}
							isSelected={selectedItemId === group.item.id}
							onSelect={() => onSelectItem?.(group.item)}
							onOpen={() => onOpenItem?.(group.item)}
							onPointerDown={(event) => onItemPointerDown?.(event, group.item)}
						/>
					{:else}
						<PbiGroup
							parent={group.item}
							children={[]}
							{movingItemId}
							{draggingItemId}
							{selectedItemId}
							isParentDraggable={movingItemId === null}
							{onSelectItem}
							{onOpenItem}
							{onAddTask}
							onParentPointerDown={onItemPointerDown}
						/>
					{/if}
				</div>
			{/each}

			{#if groups.length === 0}
				<div class="flex h-24 items-center justify-center rounded-lg border-2 border-dashed border-surface-300 dark:border-surface-600 text-sm text-surface-400 dark:text-surface-500">
					No items
				</div>
			{/if}
		</div>
	{:else}
		<div class="board-column flex-1 overflow-y-auto p-2">
			{#each groups as group (group.item.id)}
				{#if group.children.length > 0}
					<div class="mb-2">
						<PbiGroup
							parent={group.item}
							children={group.children}
							{movingItemId}
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
							isBusy={movingItemId === group.item.id}
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
							{movingItemId}
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
	{/if}
</div>
