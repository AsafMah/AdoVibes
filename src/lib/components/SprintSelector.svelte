<script lang="ts">
	import type { Sprint } from '$lib/stores/app.svelte';

	interface Props {
		sprints: Sprint[];
		currentSprint: Sprint | null;
		selectedSprint: Sprint | null;
		onSelectSprint: (sprint: Sprint) => void;
	}

	let { sprints, currentSprint, selectedSprint, onSelectSprint }: Props = $props();

	function formatDate(dateStr?: string) {
		if (!dateStr) return '';
		const d = new Date(dateStr);
		return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
	}

	function getDaysRemaining(finishDate?: string): string {
		if (!finishDate) return '';
		const end = new Date(finishDate);
		const now = new Date();
		const diff = Math.ceil((end.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
		if (diff < 0) return 'Ended';
		if (diff === 0) return 'Ends today';
		return `${diff}d left`;
	}

	const sortedSprints = $derived(
		[...sprints].sort((a, b) => {
			if (!a.startDate || !b.startDate) return 0;
			return new Date(b.startDate).getTime() - new Date(a.startDate).getTime();
		})
	);
</script>

<div class="space-y-1">
	<h3 class="px-2 text-xs font-semibold uppercase tracking-wider text-surface-500 dark:text-surface-400">Sprints</h3>
	<div class="max-h-[300px] overflow-y-auto">
		{#each sortedSprints as sprint (sprint.id)}
			<button
				class="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors
					{selectedSprint?.id === sprint.id ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-800 dark:text-primary-200 font-medium' : 'text-surface-600 dark:text-surface-400 hover:bg-surface-100 dark:hover:bg-surface-800'}"
				onclick={() => onSelectSprint(sprint)}
			>
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-1.5">
						{#if currentSprint?.id === sprint.id}
							<span class="h-1.5 w-1.5 rounded-full bg-green-500" title="Current sprint"></span>
						{/if}
						<span class="truncate">{sprint.name}</span>
					</div>
					{#if sprint.startDate || sprint.finishDate}
						<div class="mt-0.5 text-xs text-surface-400 dark:text-surface-500">
							{formatDate(sprint.startDate)} - {formatDate(sprint.finishDate)}
							{#if currentSprint?.id === sprint.id}
								<span class="ml-1 text-green-600 dark:text-green-400">{getDaysRemaining(sprint.finishDate)}</span>
							{/if}
						</div>
					{/if}
				</div>
			</button>
		{/each}
	</div>
</div>
