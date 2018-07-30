import { Command } from './';

const format = document.getElementById('format');

format.addEventListener(
  'click',
  (e) => {
    e.preventDefault();
    Command.format();
  },
  false,
);
