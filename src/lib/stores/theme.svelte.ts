export type Theme = 'light' | 'dark' | 'system';

let theme = $state<Theme>('system');
let resolved = $state<'light' | 'dark'>('light');

function applyTheme() {
	const isDark =
		theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
	resolved = isDark ? 'dark' : 'light';
	document.documentElement.classList.toggle('dark', isDark);
}

function init() {
	const saved = localStorage.getItem('adovibes-theme') as Theme | null;
	if (saved === 'light' || saved === 'dark' || saved === 'system') {
		theme = saved;
	}
	applyTheme();

	window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
		if (theme === 'system') applyTheme();
	});
}

function setTheme(t: Theme) {
	theme = t;
	localStorage.setItem('adovibes-theme', t);
	applyTheme();
}

function toggle() {
	setTheme(resolved === 'dark' ? 'light' : 'dark');
}

export function getThemeState() {
	return {
		get theme() { return theme; },
		get resolved() { return resolved; },
		get isDark() { return resolved === 'dark'; },
		init,
		setTheme,
		toggle
	};
}
