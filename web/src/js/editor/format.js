import { Module } from '../module';
import { Command } from './';

const format = document.getElementById('format');

format.addEventListener(
  'click',
  (e) => {
    e.preventDefault();

    const result = Module.format(Command.getValue());

    if (result !== '') {
      Command.setValue(result);
    }
  },
  false,
);
