<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { getAppState } from '$lib/stores/app.svelte';
	import { getThemeState } from '$lib/stores/theme.svelte';
	import { attachConsole } from '@tauri-apps/plugin-log';
	import SprintSelector from '$lib/components/SprintSelector.svelte';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	const app = getAppState();
	const theme = getThemeState();

	onMount(() => {
		theme.init();
		// Forward Rust logs to browser devtools console
		attachConsole();
		document.body.dataset.appReady = 'true';
	});
</script>

{#if !app.isSetupComplete}
	{@render children()}
{:else}
	<div class="flex h-screen flex-col">
		<!-- Top bar -->
		<header class="flex items-center justify-between border-b border-surface-200 dark:border-surface-700 bg-white dark:bg-surface-900 px-4 py-2">
			<div class="flex items-center gap-4">
				<h1 class="text-lg font-bold text-primary-600">AdoVibes</h1>
				<div class="flex items-center gap-2 text-sm text-surface-500 dark:text-surface-400">
					<span>{app.organization}</span>
					<span class="text-surface-300 dark:text-surface-600">/</span>
					<span class="font-medium text-surface-700 dark:text-surface-200">{app.project}</span>
					<span class="text-surface-300 dark:text-surface-600">/</span>
					<span>{app.team}</span>
				</div>
			</div>
			<div class="flex items-center gap-3">
				{#if app.currentUser}
					<span class="text-sm text-surface-500 dark:text-surface-400">{app.currentUser.displayName}</span>
				{/if}
				<button
					class="rounded-md px-2 py-1 text-surface-500 dark:text-surface-400 hover:bg-surface-100 dark:hover:bg-surface-800 hover:text-surface-700 dark:hover:text-surface-200"
					onclick={() => theme.toggle()}
					title="Toggle dark mode"
				>
					{#if theme.isDark}
						<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" /></svg>
					{:else}
						<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" /></svg>
					{/if}
				</button>
				<button
					class="rounded-md px-3 py-1 text-xs text-surface-500 dark:text-surface-400 hover:bg-surface-100 dark:hover:bg-surface-800 hover:text-surface-700 dark:hover:text-surface-200"
					onclick={() => app.resetSetup()}
				>
					Switch Project
				</button>
			</div>
		</header>

		<div class="flex flex-1 overflow-hidden">
			<!-- Sidebar -->
			<aside class="w-64 shrink-0 overflow-y-auto border-r border-surface-200 dark:border-surface-700 bg-surface-50 dark:bg-surface-900 p-4">
				<SprintSelector
					sprints={app.sprints}
					currentSprint={app.currentSprint}
					selectedSprint={app.selectedSprint}
					onSelectSprint={(sprint) => { app.selectedSprint = sprint; }}
				/>

				<div class="mt-6">
					<button
						class="w-full rounded-lg bg-primary-500 px-3 py-2 text-sm font-medium text-white hover:bg-primary-600"
						onclick={() => {
							const event = new CustomEvent('create-item');
							window.dispatchEvent(event);
						}}
					>
						+ New Item
					</button>
				</div>

				<!-- Keyboard shortcuts help -->
				<div class="mt-6 space-y-1">
					<h3 class="px-2 text-xs font-semibold uppercase tracking-wider text-surface-500 dark:text-surface-400">Shortcuts</h3>
					<div class="space-y-0.5 px-2 text-xs text-surface-400 dark:text-surface-500">
						<div><kbd class="font-mono">↑↓</kbd> Navigate</div>
						<div><kbd class="font-mono">←→</kbd> Switch columns</div>
						<div><kbd class="font-mono">Ctrl+←/→</kbd> Move item</div>
						<div><kbd class="font-mono">Enter</kbd> Open item</div>
						<div><kbd class="font-mono">N</kbd> New item</div>
						<div><kbd class="font-mono">D</kbd> Mark done</div>
						<div><kbd class="font-mono">Esc</kbd> Deselect</div>
					</div>
				</div>
			</aside>

			<!-- Main content -->
			<main class="flex-1 overflow-hidden">
				{@render children()}
			</main>
		</div>
	</div>
{/if}
