/** Keyboard navigation handler for the Kanban board */

export type BoardAction =
	| { type: 'navigate'; direction: 'up' | 'down' | 'left' | 'right' }
	| { type: 'move'; direction: 'left' | 'right' }
	| { type: 'open' }
	| { type: 'create' }
	| { type: 'edit' }
	| { type: 'done' }
	| { type: 'escape' };

export function handleBoardKeydown(event: KeyboardEvent): BoardAction | null {
	// Don't handle when typing in inputs
	const target = event.target as HTMLElement;
	if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT' || target.isContentEditable) {
		return null;
	}

	switch (event.key) {
		case 'ArrowUp':
		case 'k':
			event.preventDefault();
			return { type: 'navigate', direction: 'up' };
		case 'ArrowDown':
		case 'j':
			event.preventDefault();
			return { type: 'navigate', direction: 'down' };
		case 'ArrowLeft':
			event.preventDefault();
			return { type: 'navigate', direction: 'left' };
		case 'ArrowRight':
			event.preventDefault();
			return { type: 'navigate', direction: 'right' };
		case 'h':
			if (event.shiftKey) {
				event.preventDefault();
				return { type: 'move', direction: 'left' };
			}
			return null;
		case 'l':
			if (event.shiftKey) {
				event.preventDefault();
				return { type: 'move', direction: 'right' };
			}
			return null;
		case 'H':
			event.preventDefault();
			return { type: 'move', direction: 'left' };
		case 'L':
			event.preventDefault();
			return { type: 'move', direction: 'right' };
		case 'Enter':
			event.preventDefault();
			return { type: 'open' };
		case 'n':
			if (!event.ctrlKey && !event.metaKey) {
				event.preventDefault();
				return { type: 'create' };
			}
			return null;
		case 'e':
			if (!event.ctrlKey && !event.metaKey) {
				event.preventDefault();
				return { type: 'edit' };
			}
			return null;
		case 'd':
			if (!event.ctrlKey && !event.metaKey) {
				event.preventDefault();
				return { type: 'done' };
			}
			return null;
		case 'Escape':
			event.preventDefault();
			return { type: 'escape' };
		default:
			return null;
	}
}

const COLUMNS = ['new', 'active', 'done'] as const;
export type Column = (typeof COLUMNS)[number];

export function getNextColumn(current: Column, direction: 'left' | 'right'): Column | null {
	const idx = COLUMNS.indexOf(current);
	if (direction === 'left' && idx > 0) return COLUMNS[idx - 1];
	if (direction === 'right' && idx < COLUMNS.length - 1) return COLUMNS[idx + 1];
	return null;
}

export function getPrevColumn(current: Column): Column | null {
	return getNextColumn(current, 'left');
}
