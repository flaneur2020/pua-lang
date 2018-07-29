import { Module } from '../module';
import { Command } from './';

const format = document.getElementById('format');

format.addEventListener(
  'click',
  (e) => {
    e.preventDefault();
    Command.setValue(Module.format(Command.getValue()));
  },
  false,
);
