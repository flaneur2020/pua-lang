import { SNIPPETS } from '../constants';
import { Command } from './';

export const snippet = document.getElementById('snippet');

const fragment = document.createDocumentFragment();

SNIPPETS.forEach(({ label, value }) => {
  const option = document.createElement('option');
  option.label = label;
  option.value = value;
  fragment.appendChild(option);
});

snippet.appendChild(fragment);
snippet.selectedIndex = 1;

snippet.addEventListener(
  'change',
  () => {
    const value = snippet.value;

    if (value !== '') {
      Command.set(value);
    }
  },
  false,
);
