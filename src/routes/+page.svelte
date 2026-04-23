<script lang="ts">
	import { onMount } from 'svelte';
	import { getAppState } from '$lib/stores/app.svelte';
	import {
		getWorkItemsState,
		type BoardGrouping,
		type BoardSort,
		type BoardViewData
	} from '$lib/stores/workitems.svelte';
	import ProjectSelector from '$lib/components/ProjectSelector.svelte';
	import Board from '$lib/components/Board.svelte';
	import type { CreateWorkItemRequest, UpdateWorkItemRequest } from '$lib/stores/app.svelte';

	const app = getAppState();
	const workItemsStore = getWorkItemsState();

	let authChoice = $state<'azcli' | 'pat' | null>(null);
	let azCliOrgInput = $state('');
	let patInput = $state('');
	let patOrgInput = $state('');
	let searchQuery = $state('');
	let typeFilter = $state('all');
	let assigneeFilter = $state('all');
	let mineOnly = $state(false);
	let sortBy = $state<BoardSort>('backlog');
	let groupBy = $state<BoardGrouping>('hierarchy');
	let isRestoringSession = $state(false);

	const boardView = $derived.by<BoardViewData>(() =>
		workItemsStore.getBoardData({
			search: searchQuery,
			type: typeFilter,
			assignee: assigneeFilter,
			mineOnly,
			currentUser: app.currentUser?.displayName ?? null,
			sortBy,
			groupBy
		})
	);

	const assignees = $derived(workItemsStore.assignees);
	const isInitialBoardLoad = $derived(isRestoringSession || app.isLoading || workItemsStore.isLoading);
	const initialLoadMessage = $derived(
		isRestoringSession
			? 'Restoring your board...'
			: workItemsStore.isLoading
				? 'Loading sprint items...'
				: 'Loading Azure DevOps data...'
	);

	function clearBoardControls() {
		searchQuery = '';
		typeFilter = 'all';
		assigneeFilter = 'all';
		mineOnly = false;
		sortBy = 'backlog';
		groupBy = 'hierarchy';
	}

	onMount(async () => {
		if (app.isSetupComplete) {
			isRestoringSession = true;
			const ok = await app.restoreAuth();
			if (ok) {
				await Promise.all([
					app.fetchUser(),
					app.fetchSprints(),
					workItemsStore.primeAuthHeader(true)
				]);
			} else {
				app.resetSetup();
			}
			isRestoringSession = false;
		}
	});

	// Reload work items when sprint changes
	let lastSprintId = $state<string | null>(null);
	$effect(() => {
		const sprintId = app.selectedSprint?.id ?? null;
		if (sprintId && sprintId !== lastSprintId && app.isSetupComplete) {
			lastSprintId = sprintId;
			workItemsStore.fetchSprintItems(
				app.organization,
				app.project,
				app.team,
				app.selectedSprint!.path
			);
		} else if (!sprintId) {
			lastSprintId = null;
		}
	});

	function handleOrgChange(org: string) {
		app.organization = org;
	}

	function handleProjectSelect(projectName: string) {
		app.project = projectName;
	}

	function handleTeamSelect(teamName: string) {
		app.team = teamName;
	}

	async function handleSetupComplete() {
		app.completeSetup();
		await Promise.all([
			app.fetchUser(),
			app.fetchSprints(),
			workItemsStore.primeAuthHeader(true)
		]);
	}

	async function handleAzCliLogin() {
		if (!azCliOrgInput.trim()) return;
		const ok = await app.loginWithAzCli(azCliOrgInput.trim());
		if (ok) {
			await workItemsStore.primeAuthHeader(true);
			authChoice = null;
		}
	}

	async function handlePatLogin() {
		if (!patInput.trim() || !patOrgInput.trim()) return;
		const ok = await app.loginWithPat(patInput.trim(), patOrgInput.trim());
		if (ok) {
			await workItemsStore.primeAuthHeader(true);
			authChoice = null;
		}
	}

	function handleMoveItem(id: number, workItemType: string, targetColumn: string) {
		return workItemsStore.moveItem(app.organization, app.project, id, workItemType, targetColumn);
	}

	async function handleCreateItem(request: CreateWorkItemRequest) {
		await workItemsStore.createItem(app.organization, app.project, request);
	}

	async function handleUpdateItem(request: UpdateWorkItemRequest) {
		await workItemsStore.updateItem(app.organization, app.project, request);
	}
</script>

{#if !app.isAuthenticated && !app.isSetupComplete}
	<!-- Auth method selection -->
	<div class="flex h-screen items-center justify-center bg-surface-50 dark:bg-surface-950">
		<div class="mx-4 w-full max-w-md space-y-6 text-center">
			<h1 class="text-3xl font-bold text-surface-800 dark:text-surface-100">AdoVibes</h1>
			<p class="text-surface-600 dark:text-surface-400">A better Azure DevOps experience</p>

			{#if app.error}
				<div class="rounded-lg bg-red-50 dark:bg-red-900/20 p-4 text-left text-sm text-red-700 dark:text-red-300">
					{app.error}
					<button class="ml-2 text-red-500 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300" onclick={() => app.clearError()}>✕</button>
				</div>
			{/if}

			{#if !authChoice}
				<!-- Choose auth method -->
				<div class="space-y-3">
					<p class="text-sm text-surface-500 dark:text-surface-400">Choose how to authenticate:</p>

					<button
						class="flex w-full items-center gap-3 rounded-lg border border-surface-300 dark:border-surface-700 p-4 text-left hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors"
						onclick={() => authChoice = 'azcli'}
					>
						<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 text-lg font-bold">az</div>
						<div>
							<div class="font-medium text-surface-800 dark:text-surface-100">Azure CLI</div>
							<div class="text-xs text-surface-500 dark:text-surface-400">Use your existing <code class="rounded bg-surface-100 dark:bg-surface-800 px-1">az login</code> session</div>
						</div>
					</button>

					<button
						class="flex w-full items-center gap-3 rounded-lg border border-surface-300 dark:border-surface-700 p-4 text-left hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors"
						onclick={() => authChoice = 'pat'}
					>
						<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400 text-lg font-bold">🔑</div>
						<div>
							<div class="font-medium text-surface-800 dark:text-surface-100">Personal Access Token</div>
							<div class="text-xs text-surface-500 dark:text-surface-400">Use a PAT from your Azure DevOps settings</div>
						</div>
					</button>
				</div>
			{:else if authChoice === 'azcli'}
				<!-- Azure CLI login -->
				<div class="space-y-4 text-left">
					<button class="text-sm text-primary-600 hover:text-primary-700" onclick={() => { authChoice = null; app.clearError(); }}>← Back</button>

					<div class="rounded-lg bg-blue-50 dark:bg-blue-900/20 p-4 text-sm text-blue-800 dark:text-blue-300">
						<p class="font-semibold">Azure CLI Login</p>
						<p class="mt-1">Make sure you've run <code class="rounded bg-blue-100 dark:bg-blue-900/30 px-1.5 py-0.5 font-mono text-xs">az login</code> in your terminal, then enter the Azure DevOps organization you want to access.</p>
					</div>

					<div>
						<label for="azcli-org" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Organization</label>
						<input
							id="azcli-org"
							type="text"
							class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20"
							placeholder="your-organization"
							bind:value={azCliOrgInput}
							onkeydown={(e) => { if (e.key === 'Enter') handleAzCliLogin(); }}
						/>
						<p class="mt-1 text-xs text-surface-400 dark:text-surface-500">From https://dev.azure.com/<strong>{azCliOrgInput || 'your-org'}</strong></p>
					</div>

					<button
						class="w-full rounded-lg bg-primary-500 py-2.5 text-sm font-medium text-white hover:bg-primary-600 disabled:opacity-50"
						onclick={handleAzCliLogin}
						disabled={app.isLoading || !azCliOrgInput.trim()}
					>
						{app.isLoading ? 'Checking access...' : 'Connect with Azure CLI'}
					</button>
				</div>
			{:else if authChoice === 'pat'}
				<!-- PAT login -->
				<div class="space-y-4 text-left">
					<button class="text-sm text-primary-600 hover:text-primary-700" onclick={() => { authChoice = null; app.clearError(); }}>← Back</button>

					<div class="rounded-lg bg-amber-50 dark:bg-amber-900/20 p-4 text-sm text-amber-800 dark:text-amber-300">
						<p class="font-semibold">Personal Access Token</p>
						<p class="mt-1">Create a PAT at <strong>Azure DevOps → User Settings → Personal Access Tokens</strong>. Grant at least <em>Work Items (Read & Write)</em> scope.</p>
					</div>

					<div class="space-y-3">
						<div>
							<label for="pat-org" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Organization</label>
							<input
								id="pat-org"
								type="text"
								class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20"
								placeholder="your-organization"
								bind:value={patOrgInput}
							/>
							<p class="mt-1 text-xs text-surface-400 dark:text-surface-500">From https://dev.azure.com/<strong>{patOrgInput || 'your-org'}</strong></p>
						</div>
						<div>
							<label for="pat-input" class="block text-sm font-medium text-surface-700 dark:text-surface-200">Personal Access Token</label>
							<input
								id="pat-input"
								type="password"
								class="mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm font-mono text-surface-900 dark:text-surface-100 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20"
								placeholder="Paste your PAT here..."
								bind:value={patInput}
								onkeydown={(e) => { if (e.key === 'Enter') handlePatLogin(); }}
							/>
						</div>
					</div>

					<button
						class="w-full rounded-lg bg-primary-500 py-2.5 text-sm font-medium text-white hover:bg-primary-600 disabled:opacity-50"
						onclick={handlePatLogin}
						disabled={app.isLoading || !patInput.trim() || !patOrgInput.trim()}
					>
						{app.isLoading ? 'Validating...' : 'Connect with PAT'}
					</button>
				</div>
			{/if}
		</div>
	</div>
{:else if !app.isSetupComplete}
	<!-- Setup: Choose org/project/team -->
	<div class="flex h-screen items-center justify-center bg-surface-50 dark:bg-surface-950">
		{#if app.error}
			<div class="fixed top-4 right-4 rounded-lg bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-700 dark:text-red-300 shadow-md">
				{app.error}
				<button class="ml-2 text-red-500 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300" onclick={() => app.clearError()}>✕</button>
			</div>
		{/if}

		<ProjectSelector
			organization={app.organization}
			selectedProject={app.project}
			selectedTeam={app.team}
			isLoading={app.isLoading}
			onOrgChange={handleOrgChange}
			onProjectSelect={handleProjectSelect}
			onTeamSelect={handleTeamSelect}
			onComplete={handleSetupComplete}
			searchProjects={(q) => app.searchProjects(q)}
			searchTeams={(q) => app.searchTeams(q)}
		/>
	</div>
{:else}
	<!-- Board view -->
	{#if app.error || workItemsStore.error}
		<div class="fixed top-4 right-4 z-50 rounded-lg bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-700 dark:text-red-300 shadow-md">
			{app.error || workItemsStore.error}
			<button class="ml-2 text-red-500 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300" onclick={() => { app.clearError(); workItemsStore.clearError(); }}>✕</button>
		</div>
	{/if}

	{#if isInitialBoardLoad}
		<div class="flex h-full items-center justify-center">
			<div class="rounded-2xl border border-surface-200 bg-white/80 px-8 py-10 text-center shadow-sm dark:border-surface-800 dark:bg-surface-900/80">
				<div class="mx-auto h-10 w-10 animate-spin rounded-full border-2 border-primary-500 border-t-transparent"></div>
				<p class="mt-4 text-sm font-medium text-surface-700 dark:text-surface-200">{initialLoadMessage}</p>
				<p class="mt-1 text-xs text-surface-500 dark:text-surface-400">This can take a few seconds while Azure DevOps responds.</p>
			</div>
		</div>
	{:else if app.selectedSprint}
		<div class="flex h-full flex-col">
			<div class="border-b border-surface-200 bg-surface-50/90 px-4 py-3 dark:border-surface-800 dark:bg-surface-950/90">
				<div class="flex flex-col gap-3 xl:flex-row xl:items-end xl:justify-between">
					<div>
						<h2 class="text-sm font-semibold uppercase tracking-wide text-surface-600 dark:text-surface-300">Board controls</h2>
						<p class="text-xs text-surface-500 dark:text-surface-400">
							Showing {boardView.allItems.length} of {workItemsStore.workItems.length} work items for {app.selectedSprint.name}.
						</p>
					</div>
					<button
						class="rounded-lg border border-surface-300 px-3 py-2 text-sm text-surface-700 transition-colors hover:border-surface-400 hover:bg-surface-100 dark:border-surface-700 dark:text-surface-200 dark:hover:border-surface-600 dark:hover:bg-surface-900"
						onclick={clearBoardControls}
					>
						Reset view
					</button>
				</div>

				<div class="mt-3 grid gap-3 md:grid-cols-2 xl:grid-cols-6">
					<label class="flex flex-col gap-1 text-sm text-surface-600 dark:text-surface-300 xl:col-span-2">
						<span class="text-xs font-medium uppercase tracking-wide">Search</span>
						<input
							type="text"
							class="rounded-lg border border-surface-300 bg-white px-3 py-2 text-sm text-surface-900 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20 dark:border-surface-700 dark:bg-surface-900 dark:text-surface-100"
							placeholder="Title, ID, assignee, tags"
							bind:value={searchQuery}
						/>
					</label>

					<label class="flex flex-col gap-1 text-sm text-surface-600 dark:text-surface-300">
						<span class="text-xs font-medium uppercase tracking-wide">Type</span>
						<select
							class="rounded-lg border border-surface-300 bg-white px-3 py-2 text-sm text-surface-900 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20 dark:border-surface-700 dark:bg-surface-900 dark:text-surface-100"
							bind:value={typeFilter}
						>
							<option value="all">All types</option>
							<option value="Product Backlog Item">PBIs</option>
							<option value="Bug">Bugs</option>
							<option value="Task">Tasks</option>
						</select>
					</label>

					<label class="flex flex-col gap-1 text-sm text-surface-600 dark:text-surface-300">
						<span class="text-xs font-medium uppercase tracking-wide">Assignee</span>
						<select
							class="rounded-lg border border-surface-300 bg-white px-3 py-2 text-sm text-surface-900 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20 dark:border-surface-700 dark:bg-surface-900 dark:text-surface-100"
							bind:value={assigneeFilter}
						>
							<option value="all">Everyone</option>
							<option value="Unassigned">Unassigned</option>
							{#each assignees as assignee}
								<option value={assignee}>{assignee}</option>
							{/each}
						</select>
					</label>

					<label class="flex flex-col gap-1 text-sm text-surface-600 dark:text-surface-300">
						<span class="text-xs font-medium uppercase tracking-wide">Sort</span>
						<select
							class="rounded-lg border border-surface-300 bg-white px-3 py-2 text-sm text-surface-900 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20 dark:border-surface-700 dark:bg-surface-900 dark:text-surface-100"
							bind:value={sortBy}
						>
							<option value="backlog">Backlog order</option>
							<option value="priority">Priority</option>
							<option value="storyPoints">Story points</option>
							<option value="remainingWork">Remaining work</option>
							<option value="title">Title</option>
							<option value="id">Work item ID</option>
						</select>
					</label>

					<label class="flex items-center gap-2 rounded-lg border border-surface-300 px-3 py-2 text-sm text-surface-700 dark:border-surface-700 dark:text-surface-200">
						<input type="checkbox" class="h-4 w-4" bind:checked={mineOnly} />
						<span>Only my items</span>
					</label>
				</div>

				<p class="mt-2 text-xs text-surface-500 dark:text-surface-400">
					Drag PBIs or Bugs as groups, or drag individual task cards on their own.
				</p>
			</div>

			<div class="min-h-0 flex-1">
				<Board
					groupedByColumn={boardView.groupedByColumn}
					newItems={boardView.newItems}
					activeItems={boardView.activeItems}
					doneItems={boardView.doneItems}
					iterationPath={app.selectedSprint.path}
					onMoveItem={handleMoveItem}
					onCreateItem={handleCreateItem}
					onUpdateItem={handleUpdateItem}
					isMovePending={workItemsStore.isMovePending}
					movingItemId={workItemsStore.movingItemId}
				/>
			</div>
		</div>
	{:else}
		<div class="flex h-full items-center justify-center">
			<p class="text-sm text-surface-500 dark:text-surface-400">No sprint selected. Choose a sprint from the sidebar.</p>
		</div>
	{/if}
{/if}
