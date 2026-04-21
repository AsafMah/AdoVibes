<script lang="ts">
	import type { WorkItem } from '$lib/stores/app.svelte';
	import WorkItemCard from './WorkItemCard.svelte';

	interface Props {
		parent: WorkItem;
		children: WorkItem[];
		selectedItemId?: number | null;
		onSelectItem?: (item: WorkItem) => void;
		onOpenItem?: (item: WorkItem) => void;
		onAddTask?: (parent: WorkItem) => void;
	}

	let { parent, children, selectedItemId = null, onSelectItem, onOpenItem, onAddTask }: Props = $props();

	let expanded = $state(true);

	const doneCount = $derived(children.filter(c => c.boardColumn === 'done').length);
	const totalCount = $derived(children.length);
</script>

<div class="rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-900">
	<WorkItemCard
		item={parent}
		isSelected={selectedItemId === parent.id}
		onSelect={() => onSelectItem?.(parent)}
		onOpen={() => onOpenItem?.(parent)}
	/>

	{#if children.length > 0}
		<div class="border-t border-surface-200 dark:border-surface-700 px-2 py-1">
			<button
				class="flex w-full items-center justify-between text-xs text-surface-500 dark:text-surface-400 hover:text-surface-700 dark:hover:text-surface-200"
				onclick={() => expanded = !expanded}
			>
				<span>Tasks ({doneCount}/{totalCount})</span>
				<span class="text-[10px]">{expanded ? '▼' : '▶'}</span>
			</button>
		</div>
	{/if}

	{#if expanded && children.length > 0}
		<div class="space-y-1 px-1 pb-2">
			{#each children as child (child.id)}
				<WorkItemCard
					item={child}
					isSelected={selectedItemId === child.id}
					onSelect={() => onSelectItem?.(child)}
					onOpen={() => onOpenItem?.(child)}
				/>
			{/each}
		</div>
	{/if}

	{#if onAddTask}
		<div class="border-t border-surface-200 dark:border-surface-700 px-2 py-1.5">
			<button
				class="w-full rounded px-2 py-1 text-left text-xs text-surface-400 dark:text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800 hover:text-surface-600 dark:hover:text-surface-300"
				onclick={() => onAddTask?.(parent)}
			>
				+ Add Task
			</button>
		</div>
	{/if}
</div>
