<script lang="ts">
	import type { Project, Team } from '$lib/stores/app.svelte';
	import Combobox from './Combobox.svelte';
	import type { ComboboxOption } from './Combobox.svelte';

	interface Props {
		organization: string;
		selectedProject: string;
		selectedTeam: string;
		isLoading: boolean;
		onOrgChange: (org: string) => void;
		onProjectSelect: (project: string) => void;
		onTeamSelect: (team: string) => void;
		onComplete: () => void;
		searchProjects: (query: string) => Promise<Project[]>;
		searchTeams: (query: string) => Promise<Team[]>;
	}

	let {
		organization,
		selectedProject,
		selectedTeam,
		isLoading,
		onOrgChange,
		onProjectSelect,
		onTeamSelect,
		onComplete,
		searchProjects,
		searchTeams
	}: Props = $props();

	let orgInput = $state(organization);
	let orgConnected = $state(!!organization);

	$effect(() => {
		orgInput = organization;
		orgConnected = !!organization;
	});

	function handleOrgSubmit() {
		if (orgInput.trim()) {
			onOrgChange(orgInput.trim());
			orgConnected = true;
		}
	}

	async function handleProjectSearch(query: string): Promise<ComboboxOption[]> {
		const results = await searchProjects(query);
		return results.map(p => ({ label: p.name, value: p.name }));
	}

	async function handleTeamSearch(query: string): Promise<ComboboxOption[]> {
		const results = await searchTeams(query);
		return results.map(t => ({ label: t.name, value: t.name }));
	}

	const canComplete = $derived(organization && selectedProject && selectedTeam);
</script>

<div class="mx-auto max-w-lg space-y-6 p-8">
	<div class="text-center">
		<h1 class="text-3xl font-bold text-surface-800 dark:text-surface-100">AdoVibes</h1>
		<p class="mt-2 text-surface-500 dark:text-surface-400">Connect to your Azure DevOps project</p>
	</div>

	<!-- Organization -->
	<div class="space-y-2">
		<label for="org-input" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Organization</label>
		<div class="flex gap-2">
			<input
				id="org-input"
				type="text"
				class="flex-1 rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20"
				placeholder="your-organization"
				bind:value={orgInput}
				onkeydown={(e) => { if (e.key === 'Enter') handleOrgSubmit(); }}
			/>
			<button
				class="rounded-lg bg-primary-500 px-4 py-2 text-sm font-medium text-white hover:bg-primary-600 disabled:opacity-50"
				onclick={handleOrgSubmit}
				disabled={!orgInput.trim() || isLoading}
			>
				{isLoading ? 'Loading...' : 'Connect'}
			</button>
		</div>
		<p class="text-xs text-surface-400 dark:text-surface-500">
			From your Azure DevOps URL: https://dev.azure.com/<strong>{orgInput || 'your-org'}</strong>
		</p>
	</div>

	<!-- Project -->
	{#if orgConnected}
		<div class="space-y-2">
			<label for="project-select" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Project</label>
			<Combobox
				id="project-select"
				value={selectedProject}
				placeholder="Type to search projects..."
				onSearch={handleProjectSearch}
				onSelect={onProjectSelect}
			/>
		</div>
	{/if}

	<!-- Team -->
	{#if selectedProject}
		<div class="space-y-2">
			<label for="team-select" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Team</label>
			<Combobox
				id="team-select"
				value={selectedTeam}
				placeholder="Type to search teams..."
				onSearch={handleTeamSearch}
				onSelect={onTeamSelect}
			/>
		</div>
	{/if}

	<!-- Go button -->
	{#if canComplete}
		<button
			class="w-full rounded-lg bg-primary-500 py-3 text-sm font-semibold text-white hover:bg-primary-600 transition-colors"
			onclick={onComplete}
		>
			Open Board →
		</button>
	{/if}
</div>
