<script lang="ts">
	import { flip } from 'svelte/animate';
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
		dragEnabled?: boolean;
		movingItemId?: number | null;
		selectedItemId?: number | null;
		onSelectItem?: (item: WorkItem) => void;
		onOpenItem?: (item: WorkItem) => void;
		onAddTask?: (parent: WorkItem) => void;
		onDropItem?: (itemId: number, targetColumn: string) => void;
	}

	let { title, column, groups, items, dragEnabled = false, movingItemId = null, selectedItemId = null, onSelectItem, onOpenItem, onAddTask, onDropItem }: Props = $props();

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
	type DragEntry = { id: number; item: WorkItem; children: WorkItem[] };
	const TASK_DRAG_MIME = 'application/x-adovibes-task';

	let dragEntries = $state<DragEntry[]>([]);

	$effect(() => {
		dragEntries = groups.map((group) => ({ id: group.item.id, item: group.item, children: group.children }));
	});

	function handleDndConsider(e: CustomEvent<{ items: DragEntry[] }>) {
		dragEntries = e.detail.items;
	}

	function handleDndFinalize(e: CustomEvent<{ items: DragEntry[] }>) {
		const droppedEntries = e.detail.items;
		dragEntries = droppedEntries;
		const currentIds = new Set(groups.map((group) => group.item.id));
		for (const entry of droppedEntries) {
			if (!currentIds.has(entry.id)) {
				onDropItem?.(entry.item.id, column);
			}
		}
	}

	function handleTaskDragStart(event: DragEvent, item: WorkItem) {
		event.dataTransfer?.setData(
			TASK_DRAG_MIME,
			JSON.stringify({
				id: item.id,
				workItemType: item.workItemType,
				sourceColumn: item.boardColumn
			})
		);
		if (event.dataTransfer) {
			event.dataTransfer.effectAllowed = 'move';
		}
	}

	function handleTaskDragOver(event: DragEvent) {
		if (!dragEnabled || movingItemId !== null) {
			return;
		}

		const payload = event.dataTransfer?.types.includes(TASK_DRAG_MIME);
		if (!payload) {
			return;
		}

		event.preventDefault();
		if (event.dataTransfer) {
			event.dataTransfer.dropEffect = 'move';
		}
	}

	function handleTaskDrop(event: DragEvent) {
		if (!dragEnabled || movingItemId !== null) {
			return;
		}

		const raw = event.dataTransfer?.getData(TASK_DRAG_MIME);
		if (!raw) {
			return;
		}

		event.preventDefault();
		const payload = JSON.parse(raw) as { id: number; sourceColumn: string };
		if (payload.sourceColumn !== column) {
			onDropItem?.(payload.id, column);
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

	{#if dragEnabled}
		<div
			class="board-column flex-1 overflow-y-auto p-2"
			role="presentation"
			use:dndzone={{
				items: dragEntries,
				flipDurationMs,
				dropTargetStyle: { outline: '2px solid #6366f1', borderRadius: '0.5rem' }
			}}
			ondragover={handleTaskDragOver}
			ondrop={handleTaskDrop}
			onconsider={handleDndConsider}
			onfinalize={handleDndFinalize}
		>
			{#each dragEntries as entry (entry.id)}
				<div class="mb-2" animate:flip={{ duration: flipDurationMs }}>
					{#if entry.children.length > 0}
						<PbiGroup
							parent={entry.item}
							children={entry.children}
							{movingItemId}
							{selectedItemId}
							{onSelectItem}
							{onOpenItem}
							{onAddTask}
							onTaskDragStart={handleTaskDragStart}
						/>
					{:else if entry.item.workItemType === 'Task'}
						<WorkItemCard
							item={entry.item}
							isBusy={movingItemId === entry.item.id}
							isDraggable={movingItemId === null}
							isSelected={selectedItemId === entry.item.id}
							onSelect={() => onSelectItem?.(entry.item)}
							onOpen={() => onOpenItem?.(entry.item)}
							onDragStart={(event) => handleTaskDragStart(event, entry.item)}
						/>
					{:else}
						<PbiGroup
							parent={entry.item}
							children={[]}
							{movingItemId}
							{selectedItemId}
							{onSelectItem}
							{onOpenItem}
							{onAddTask}
							onTaskDragStart={handleTaskDragStart}
						/>
					{/if}
				</div>
			{/each}

			{#if dragEntries.length === 0}
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
