<script lang="ts">
	import type { WorkItem } from '$lib/stores/app.svelte';

	interface Props {
		item: WorkItem;
		isBusy?: boolean;
		isSelected?: boolean;
		isDraggable?: boolean;
		isDragging?: boolean;
		onSelect?: () => void;
		onOpen?: () => void;
		onPointerDown?: (event: PointerEvent) => void;
	}

	let {
		item,
		isBusy = false,
		isSelected = false,
		isDraggable = false,
		isDragging = false,
		onSelect,
		onOpen,
		onPointerDown
	}: Props = $props();

	const typeColors: Record<string, string> = {
		'Product Backlog Item': 'bg-blue-500',
		'Bug': 'bg-red-500',
		'Task': 'bg-yellow-500',
		'Epic': 'bg-purple-500',
		'Feature': 'bg-green-500'
	};

	const typeLabels: Record<string, string> = {
		'Product Backlog Item': 'PBI',
		'Bug': 'Bug',
		'Task': 'Task',
		'Epic': 'Epic',
		'Feature': 'Feature'
	};

	function handleClick() {
		if (isBusy) return;
		onSelect?.();
	}

	function handleDblClick() {
		if (isBusy) return;
		onOpen?.();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (isBusy) return;
		if (e.key === 'Enter') {
			e.preventDefault();
			onOpen?.();
		}
	}

	function handlePointerDown(event: PointerEvent) {
		if (!isDraggable || isBusy || event.button !== 0) {
			return;
		}
		onPointerDown?.(event);
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="card rounded-md border p-3 transition-all cursor-pointer
		{isBusy ? 'opacity-70 cursor-wait' : ''}
		{isDragging ? 'opacity-35' : ''}
		{isDraggable && !isBusy ? 'cursor-grab active:cursor-grabbing' : ''}
		{isSelected ? 'ring-2 ring-primary-500 border-primary-500 bg-surface-100 dark:bg-surface-800' : 'border-surface-300 dark:border-surface-600 bg-surface-50 dark:bg-surface-900 hover:border-surface-400 dark:hover:border-surface-500'}
		{item.workItemType === 'Task' ? 'ml-4 border-l-2' : ''}"
	style="border-left-color: {item.workItemType === 'Task' ? '#eab308' : 'inherit'}"
	tabindex={isBusy ? -1 : 0}
	role="button"
	onclick={handleClick}
	ondblclick={handleDblClick}
	onkeydown={handleKeydown}
	onpointerdown={handlePointerDown}
	data-item-id={item.id}
>
	<div class="flex items-start gap-2">
		<span class="mt-0.5 inline-block h-2 w-2 shrink-0 rounded-full {typeColors[item.workItemType] || 'bg-gray-500'}" title={item.workItemType}></span>
		<div class="min-w-0 flex-1">
			<div class="flex items-center gap-1.5">
				<span class="text-xs font-medium text-surface-500 dark:text-surface-400">{typeLabels[item.workItemType] || item.workItemType}</span>
				<span class="text-xs text-surface-400 dark:text-surface-500">#{item.id}</span>
				{#if isBusy}
					<span class="rounded bg-primary-100 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide text-primary-700 dark:bg-primary-900/40 dark:text-primary-300">
						Moving...
					</span>
				{/if}
			</div>
			<p class="mt-0.5 text-sm font-medium leading-snug text-surface-800 dark:text-surface-100 line-clamp-2">{item.title}</p>
		</div>
	</div>

	<div class="mt-2 flex items-center justify-between text-xs text-surface-500 dark:text-surface-400">
		<div class="flex items-center gap-2">
			{#if item.assignedTo}
				<span class="max-w-[120px] truncate" title={item.assignedTo}>
					{item.assignedTo}
				</span>
			{:else}
				<span class="italic text-surface-400 dark:text-surface-500">Unassigned</span>
			{/if}
		</div>
		<div class="flex items-center gap-2">
			{#if item.storyPoints != null}
				<span class="rounded bg-surface-200 dark:bg-surface-700 px-1.5 py-0.5 font-medium">{item.storyPoints} SP</span>
			{/if}
			{#if item.remainingWork != null}
				<span class="rounded bg-surface-200 dark:bg-surface-700 px-1.5 py-0.5">{item.remainingWork}h</span>
			{/if}
			{#if item.priority != null}
				<span class="rounded bg-surface-200 dark:bg-surface-700 px-1.5 py-0.5">P{item.priority}</span>
			{/if}
		</div>
	</div>

	{#if item.tags}
		<div class="mt-1.5 flex flex-wrap gap-1">
			{#each item.tags.split(';').map(t => t.trim()).filter(Boolean) as tag}
				<span class="rounded bg-primary-100 dark:bg-primary-900/30 px-1.5 py-0.5 text-xs text-primary-700 dark:text-primary-300">{tag}</span>
			{/each}
		</div>
	{/if}
</div>
