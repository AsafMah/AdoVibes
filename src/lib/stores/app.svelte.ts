import { invoke } from '@tauri-apps/api/core';

// --- Types matching Rust backend ---

export interface Project {
	id: string;
	name: string;
	description?: string;
}

export interface Team {
	id: string;
	name: string;
	description?: string;
}

export interface Sprint {
	id: string;
	name: string;
	path: string;
	startDate?: string;
	finishDate?: string;
	timeFrame?: string;
}

export interface WorkItem {
	id: number;
	title: string;
	state: string;
	workItemType: string;
	assignedTo?: string;
	iterationPath?: string;
	areaPath?: string;
	priority?: number;
	storyPoints?: number;
	remainingWork?: number;
	description?: string;
	tags?: string;
	parentId?: number;
	boardColumn: string;
}

export interface CreateWorkItemRequest {
	workItemType: string;
	title: string;
	description?: string;
	assignedTo?: string;
	iterationPath?: string;
	areaPath?: string;
	priority?: number;
	storyPoints?: number;
	parentId?: number;
	tags?: string;
}

export interface UpdateWorkItemRequest {
	id: number;
	state?: string;
	assignedTo?: string;
	title?: string;
	description?: string;
	priority?: number;
	storyPoints?: number;
	remainingWork?: number;
	tags?: string;
}

export interface UserProfile {
	displayName: string;
	email?: string;
	id: string;
}

// --- App-level state ---

export type AuthMethod = 'azcli' | 'pat';

let organization = $state('');
let project = $state('');
let team = $state('');
let authMethod = $state<AuthMethod>('azcli');
let isAuthenticated = $state(false);
let currentUser = $state<UserProfile | null>(null);
let sprints = $state<Sprint[]>([]);
let currentSprint = $state<Sprint | null>(null);
let selectedSprint = $state<Sprint | null>(null);
let isLoading = $state(false);
let error = $state<string | null>(null);
let isSetupComplete = $state(false);
let storedPat = $state('');

// Persist/restore config from localStorage
function loadConfig() {
	try {
		const saved = localStorage.getItem('adovibes-config');
		if (saved) {
			const config = JSON.parse(saved);
			organization = config.organization || '';
			project = config.project || '';
			team = config.team || '';
			authMethod = config.authMethod || 'azcli';
			storedPat = config.pat || '';
			if (organization && project && team) {
				isSetupComplete = true;
			}
		}
	} catch {
		// ignore
	}
}

function saveConfig() {
	localStorage.setItem(
		'adovibes-config',
		JSON.stringify({ organization, project, team, authMethod, pat: storedPat })
	);
}

export function getAppState() {
	loadConfig();

	return {
		get organization() { return organization; },
		set organization(v: string) { organization = v; },
		get project() { return project; },
		set project(v: string) { project = v; },
		get team() { return team; },
		set team(v: string) { team = v; },
		get authMethod() { return authMethod; },
		set authMethod(v: AuthMethod) { authMethod = v; },
		get isAuthenticated() { return isAuthenticated; },
		get currentUser() { return currentUser; },
		get sprints() { return sprints; },
		get currentSprint() { return currentSprint; },
		get selectedSprint() { return selectedSprint; },
		set selectedSprint(v: Sprint | null) { selectedSprint = v; },
		get isLoading() { return isLoading; },
		get error() { return error; },
		get isSetupComplete() { return isSetupComplete; },

		async checkAuth() {
			try {
				if (authMethod === 'pat') {
					// PAT is always "authenticated" — validation happens when connecting
					isAuthenticated = true;
				} else {
					isAuthenticated = await invoke<boolean>('check_auth_status');
				}
			} catch (e) {
				isAuthenticated = false;
				error = `Auth check failed: ${e}`;
			}
		},

		async loginWithPat(pat: string, org: string) {
			isLoading = true;
			error = null;
			try {
				const valid = await invoke<boolean>('validate_pat', { pat, organization: org });
				if (!valid) {
					error = 'Invalid PAT or organization. Please check your token and try again.';
					isAuthenticated = false;
					return false;
				}
				await invoke('set_auth_pat', { pat });
				authMethod = 'pat';
				storedPat = pat;
				isAuthenticated = true;
				organization = org;
				return true;
			} catch (e) {
				error = `PAT validation failed: ${e}`;
				isAuthenticated = false;
				return false;
			} finally {
				isLoading = false;
			}
		},

		async loginWithAzCli() {
			isLoading = true;
			error = null;
			try {
				await invoke('set_auth_azcli');
				authMethod = 'azcli';
				const ok = await invoke<boolean>('check_auth_status');
				isAuthenticated = ok;
				if (!ok) {
					error = 'Azure CLI not logged in. Please run `az login` first.';
				}
				return ok;
			} catch (e) {
				error = `Az CLI auth failed: ${e}`;
				isAuthenticated = false;
				return false;
			} finally {
				isLoading = false;
			}
		},

		async fetchUser() {
			if (!organization) return;
			try {
				currentUser = await invoke<UserProfile>('get_current_user', { organization });
			} catch (e) {
				error = `Failed to get user: ${e}`;
			}
		},

		async searchProjects(query: string): Promise<Project[]> {
			if (!organization) return [];
			try {
				return await invoke<Project[]>('search_projects', { organization, query });
			} catch (e) {
				error = `Failed to search projects: ${e}`;
				return [];
			}
		},

		async searchTeams(query: string): Promise<Team[]> {
			if (!organization || !project) return [];
			try {
				return await invoke<Team[]>('search_teams', { organization, project, query });
			} catch (e) {
				error = `Failed to search teams: ${e}`;
				return [];
			}
		},

		async fetchSprints() {
			if (!organization || !project || !team) return;
			isLoading = true;
			error = null;
			try {
				sprints = await invoke<Sprint[]>('list_iterations', {
					organization,
					project,
					team
				});
				const current = await invoke<Sprint | null>('get_current_iteration', {
					organization,
					project,
					team
				});
				currentSprint = current;
				if (current && !selectedSprint) {
					selectedSprint = current;
				}
			} catch (e) {
				error = `Failed to load sprints: ${e}`;
			} finally {
				isLoading = false;
			}
		},

		async restoreAuth(): Promise<boolean> {
			try {
				if (authMethod === 'pat' && storedPat) {
					await invoke('set_auth_pat', { pat: storedPat });
					isAuthenticated = true;
					return true;
				} else if (authMethod === 'azcli') {
					await invoke('set_auth_azcli');
					const ok = await invoke<boolean>('check_auth_status');
					isAuthenticated = ok;
					return ok;
				}
			} catch {
				isAuthenticated = false;
			}
			return false;
		},

		completeSetup() {
			if (organization && project && team) {
				isSetupComplete = true;
				saveConfig();
			}
		},

		resetSetup() {
			isSetupComplete = false;
			isAuthenticated = false;
			authMethod = 'azcli';
			organization = '';
			project = '';
			team = '';
			storedPat = '';
			sprints = [];
			currentSprint = null;
			selectedSprint = null;
			localStorage.removeItem('adovibes-config');
		},

		clearError() {
			error = null;
		}
	};
}
