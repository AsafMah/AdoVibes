<script lang="ts">
	import type { CreateWorkItemRequest } from '$lib/stores/app.svelte';

	interface Props {
		iterationPath: string;
		parentId?: number;
		parentTitle?: string;
		onSubmit: (request: CreateWorkItemRequest) => void;
		onClose: () => void;
	}

	let { iterationPath, parentId, parentTitle, onSubmit, onClose }: Props = $props();

	// parentId is stable for the lifetime of this dialog, initial capture is intentional
	let workItemType = $state(parentId ? 'Task' : 'Product Backlog Item');
	let title = $state('');
	let description = $state('');
	let assignedTo = $state('');
	let priority = $state<number | undefined>(undefined);
	let storyPoints = $state<number | undefined>(undefined);
	let tags = $state('');

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (!title.trim()) return;

		const request: CreateWorkItemRequest = {
			workItemType,
			title: title.trim(),
			iterationPath,
			parentId,
		};

		if (description.trim()) request.description = description.trim();
		if (assignedTo.trim()) request.assignedTo = assignedTo.trim();
		if (priority != null) request.priority = priority;
		if (storyPoints != null) request.storyPoints = storyPoints;
		if (tags.trim()) request.tags = tags.trim();

		onSubmit(request);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
	onkeydown={handleKeydown}
	role="dialog"
	aria-modal="true"
>
	<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
	<div class="mx-4 w-full max-w-lg rounded-xl bg-white dark:bg-surface-900 p-6 shadow-2xl" onclick={(e) => e.stopPropagation()}>
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-lg font-semibold text-surface-800 dark:text-surface-100">
				{parentId ? `Add Task to ${parentTitle || `#${parentId}`}` : 'Create Work Item'}
			</h2>
			<button
				class="rounded p-1 text-surface-400 dark:text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800 hover:text-surface-600 dark:hover:text-surface-300"
				onclick={onClose}
			>
				✕
			</button>
		</div>

		<form onsubmit={handleSubmit} class="space-y-4">
			{#if !parentId}
				<div>
					<label for="type-select" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Type</label>
					<select
						id="type-select"
						class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
						bind:value={workItemType}
					>
						<option value="Product Backlog Item">Product Backlog Item</option>
						<option value="Bug">Bug</option>
						<option value="Task">Task</option>
						<option value="Epic">Epic</option>
						<option value="Feature">Feature</option>
					</select>
				</div>
			{/if}

			<div>
				<label for="title-input" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Title</label>
				<!-- svelte-ignore a11y_autofocus -->
				<input
					id="title-input"
					type="text"
					class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20"
					placeholder="Enter title..."
					bind:value={title}
					autofocus
				/>
			</div>

			<div>
				<label for="desc-input" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Description</label>
				<textarea
					id="desc-input"
					class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20"
					rows="3"
					placeholder="Optional description..."
					bind:value={description}
				></textarea>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div>
					<label for="assigned-input" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Assigned To</label>
					<input
						id="assigned-input"
						type="text"
						class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
						placeholder="email@domain.com"
						bind:value={assignedTo}
					/>
				</div>
				<div>
					<label for="priority-input" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Priority</label>
					<select
						id="priority-input"
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

			{#if workItemType === 'Product Backlog Item' || workItemType === 'Epic' || workItemType === 'Feature'}
				<div>
					<label for="sp-input" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Story Points</label>
					<input
						id="sp-input"
						type="number"
						class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
						placeholder="e.g. 5"
						bind:value={storyPoints}
					/>
				</div>
			{/if}

			<div>
				<label for="tags-input" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Tags</label>
				<input
					id="tags-input"
					type="text"
					class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100"
					placeholder="Separate with semicolons"
					bind:value={tags}
				/>
			</div>

			<div class="flex justify-end gap-2 pt-2">
				<button
					type="button"
					class="rounded-lg border border-surface-300 dark:border-surface-600 px-4 py-2 text-sm text-surface-600 dark:text-surface-300 hover:bg-surface-100 dark:hover:bg-surface-800"
					onclick={onClose}
				>
					Cancel
				</button>
				<button
					type="submit"
					class="rounded-lg bg-primary-500 px-4 py-2 text-sm font-medium text-white hover:bg-primary-600 disabled:opacity-50"
					disabled={!title.trim()}
				>
					Create
				</button>
			</div>
		</form>
	</div>
</div>
