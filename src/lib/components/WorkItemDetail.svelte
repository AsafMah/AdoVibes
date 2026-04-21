<script lang="ts">
	import type { WorkItem, UpdateWorkItemRequest } from '$lib/stores/app.svelte';

	interface Props {
		item: WorkItem;
		onUpdate: (request: UpdateWorkItemRequest) => void;
		onClose: () => void;
	}

	let { item, onUpdate, onClose }: Props = $props();

	/* svelte-ignore state_referenced_locally */
	let title = $state(item.title);
	/* svelte-ignore state_referenced_locally */
	let description = $state(item.description || '');
	/* svelte-ignore state_referenced_locally */
	let assignedTo = $state(item.assignedTo || '');
	/* svelte-ignore state_referenced_locally */
	let priority = $state(item.priority);
	/* svelte-ignore state_referenced_locally */
	let storyPoints = $state(item.storyPoints);
	/* svelte-ignore state_referenced_locally */
	let remainingWork = $state(item.remainingWork);
	/* svelte-ignore state_referenced_locally */
	let tags = $state(item.tags || '');

	$effect(() => {
		title = item.title;
		description = item.description || '';
		assignedTo = item.assignedTo || '';
		priority = item.priority;
		storyPoints = item.storyPoints;
		remainingWork = item.remainingWork;
		tags = item.tags || '';
	});

	const typeColors: Record<string, string> = {
		'Product Backlog Item': 'text-blue-600 dark:text-blue-400',
		'Bug': 'text-red-600 dark:text-red-400',
		'Task': 'text-yellow-600 dark:text-yellow-400',
		'Epic': 'text-purple-600 dark:text-purple-400',
		'Feature': 'text-green-600 dark:text-green-400'
	};

	function handleSave() {
		const request: UpdateWorkItemRequest = { id: item.id };
		if (title !== item.title) request.title = title;
		if (description !== (item.description || '')) request.description = description;
		if (assignedTo !== (item.assignedTo || '')) request.assignedTo = assignedTo;
		if (priority !== item.priority) request.priority = priority;
		if (storyPoints !== item.storyPoints) request.storyPoints = storyPoints;
		if (remainingWork !== item.remainingWork) request.remainingWork = remainingWork;
		if (tags !== (item.tags || '')) request.tags = tags;

		onUpdate(request);
		onClose();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
		if (e.key === 's' && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			handleSave();
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
	onkeydown={handleKeydown}
	role="dialog"
	aria-modal="true"
	tabindex="-1"
>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="mx-4 w-full max-w-2xl rounded-xl bg-white dark:bg-surface-900 shadow-2xl" onclick={(e) => e.stopPropagation()}>
		<!-- Header -->
		<div class="flex items-center justify-between border-b border-surface-200 dark:border-surface-700 px-6 py-4">
			<div class="flex items-center gap-3">
				<span class="{typeColors[item.workItemType] || 'text-surface-600 dark:text-surface-400'} text-sm font-semibold">
					{item.workItemType}
				</span>
				<span class="text-sm text-surface-400 dark:text-surface-500">#{item.id}</span>
				<span class="rounded px-2 py-0.5 text-xs font-medium
					{item.boardColumn === 'new' ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300' : ''}
					{item.boardColumn === 'active' ? 'bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300' : ''}
					{item.boardColumn === 'done' ? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300' : ''}
				">
					{item.state}
				</span>
			</div>
			<button
				class="rounded p-1 text-surface-400 dark:text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800 hover:text-surface-600 dark:hover:text-surface-300"
				onclick={onClose}
			>
				✕
			</button>
		</div>

		<!-- Body -->
		<div class="max-h-[70vh] overflow-y-auto p-6 space-y-4">
			<div>
				<label for="detail-title" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Title</label>
				<input
					id="detail-title"
					type="text"
					class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20"
					bind:value={title}
				/>
			</div>

			<div>
				<label for="detail-desc" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Description</label>
				<textarea
					id="detail-desc"
					class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
					rows="4"
					bind:value={description}
				></textarea>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div>
					<label for="detail-assigned" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Assigned To</label>
					<input
						id="detail-assigned"
						type="text"
						class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
						bind:value={assignedTo}
					/>
				</div>
				<div>
					<label for="detail-priority" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Priority</label>
					<select
						id="detail-priority"
						class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
						bind:value={priority}
					>
						<option value={undefined}>None</option>
						<option value={1}>1 - Critical</option>
						<option value={2}>2 - High</option>
						<option value={3}>3 - Medium</option>
						<option value={4}>4 - Low</option>
					</select>
				</div>
			</div>

			<div class="grid grid-cols-2 gap-4">
				{#if item.workItemType !== 'Task'}
					<div>
						<label for="detail-sp" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Story Points</label>
						<input
							id="detail-sp"
							type="number"
							class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
							bind:value={storyPoints}
						/>
					</div>
				{/if}
				{#if item.workItemType === 'Task'}
					<div>
						<label for="detail-rw" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Remaining Work (hours)</label>
						<input
							id="detail-rw"
							type="number"
							class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
							bind:value={remainingWork}
						/>
					</div>
				{/if}
				<div>
					<label for="detail-tags" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Tags</label>
					<input
						id="detail-tags"
						type="text"
						class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
						placeholder="Separate with semicolons"
						bind:value={tags}
					/>
				</div>
			</div>

			{#if item.iterationPath}
				<div class="text-sm text-surface-500 dark:text-surface-400">
					<strong>Iteration:</strong> {item.iterationPath}
				</div>
			{/if}
			{#if item.parentId}
				<div class="text-sm text-surface-500 dark:text-surface-400">
					<strong>Parent:</strong> #{item.parentId}
				</div>
			{/if}
		</div>

		<!-- Footer -->
		<div class="flex justify-end gap-2 border-t border-surface-200 dark:border-surface-700 px-6 py-4">
			<span class="mr-auto text-xs text-surface-400 dark:text-surface-500">Ctrl+S to save</span>
			<button
				type="button"
				class="rounded-lg border border-surface-300 dark:border-surface-600 px-4 py-2 text-sm text-surface-600 dark:text-surface-300 hover:bg-surface-100 dark:hover:bg-surface-800"
				onclick={onClose}
			>
				Cancel
			</button>
			<button
				type="button"
				class="rounded-lg bg-primary-500 px-4 py-2 text-sm font-medium text-white hover:bg-primary-600"
				onclick={handleSave}
			>
				Save
			</button>
		</div>
	</div>
</div>
