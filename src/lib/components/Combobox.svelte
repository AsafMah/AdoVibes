<script lang="ts">
	export interface ComboboxOption {
		label: string;
		value: string;
	}

	interface Props {
		id: string;
		value: string;
		placeholder?: string;
		minChars?: number;
		debounceMs?: number;
		onSearch: (query: string) => Promise<ComboboxOption[]>;
		onSelect: (value: string) => void;
	}

	let { id, value, placeholder = 'Type to search...', minChars = 1, debounceMs = 250, onSearch, onSelect }: Props = $props();

	/* svelte-ignore state_referenced_locally */
	let query = $state(value || '');
	let results = $state<ComboboxOption[]>([]);
	let isOpen = $state(false);
	let isSearching = $state(false);
	let highlightedIndex = $state(-1);
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	// Sync query when value changes externally (e.g. restored from config)
	$effect(() => {
		query = value || '';
	});

	function close() {
		setTimeout(() => {
			isOpen = false;
			// Reset query to selected value if it doesn't match
			if (value && query !== value) {
				query = value;
			}
		}, 150);
	}

	function select(opt: ComboboxOption) {
		query = opt.label;
		isOpen = false;
		results = [];
		onSelect(opt.value);
	}

	async function doSearch(q: string, allowPrefetch = false) {
		if (!allowPrefetch && q.trim().length < minChars) {
			results = [];
			return;
		}
		isSearching = true;
		try {
			results = await onSearch(q.trim());
			highlightedIndex = results.length > 0 ? 0 : -1;
		} catch {
			results = [];
		} finally {
			isSearching = false;
		}
	}

	function handleInput() {
		isOpen = true;
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => doSearch(query), debounceMs);
	}

	function handleFocus() {
		isOpen = true;
		if (results.length === 0) {
			doSearch(query, query.trim().length === 0);
			return;
		}

		if (query.trim().length < minChars && query.trim().length > 0) {
			results = [];
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!isOpen && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
			handleFocus();
			e.preventDefault();
			return;
		}

		if (!isOpen) return;

		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				highlightedIndex = Math.min(highlightedIndex + 1, results.length - 1);
				break;
			case 'ArrowUp':
				e.preventDefault();
				highlightedIndex = Math.max(highlightedIndex - 1, 0);
				break;
			case 'Enter':
				e.preventDefault();
				if (highlightedIndex >= 0 && results[highlightedIndex]) {
					select(results[highlightedIndex]);
				} else if (results.length === 1) {
					select(results[0]);
				}
				break;
			case 'Escape':
				e.preventDefault();
				isOpen = false;
				break;
		}
	}
</script>

<div class="relative">
	<input
		{id}
		type="text"
		role="combobox"
		aria-expanded={isOpen}
		aria-autocomplete="list"
		aria-controls="{id}-listbox"
		class="w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-900 dark:text-surface-100 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20"
		{placeholder}
		bind:value={query}
		oninput={handleInput}
		onfocus={handleFocus}
		onblur={close}
		onkeydown={handleKeydown}
		autocomplete="off"
	/>

	{#if isSearching}
		<div class="absolute right-3 top-2.5">
			<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary-500 border-t-transparent"></div>
		</div>
	{/if}

	{#if isOpen && results.length > 0}
		<ul
			id="{id}-listbox"
			role="listbox"
			class="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 py-1 shadow-lg"
		>
			{#each results as opt, i (opt.value)}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<li
					role="option"
					aria-selected={opt.value === value}
					class="cursor-pointer px-3 py-1.5 text-sm transition-colors
						{i === highlightedIndex ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-800 dark:text-primary-200' : 'text-surface-800 dark:text-surface-200 hover:bg-surface-100 dark:hover:bg-surface-700'}
						{opt.value === value ? 'font-medium' : ''}"
					onmousedown={() => select(opt)}
					onmouseenter={() => highlightedIndex = i}
				>
					{opt.label}
				</li>
			{/each}
		</ul>
	{:else if isOpen && (query.trim().length >= minChars || query.trim().length === 0) && !isSearching && results.length === 0}
		<div class="absolute z-50 mt-1 w-full rounded-lg border border-surface-300 dark:border-surface-600 bg-white dark:bg-surface-800 px-3 py-2 text-sm text-surface-400 shadow-lg">
			No matches found
		</div>
	{/if}
</div>
