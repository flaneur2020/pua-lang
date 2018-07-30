import escape from 'escape-html';
import LZString from 'lz-string';
import CodeMirror from 'codemirror';
import { SHARE_QUERY_KEY, SNIPPETS } from '../constants';
import { Module } from '../module';
import { date2time } from './utils';
import './monkey-mode';
import './format';
import { snippet } from './snippet';
import './share';

const source = document.getElementById('source');
const run = document.getElementById('run');
const outputContainer = document.getElementById('output-container');
const output = document.getElementById('output');
const lastUpdated = document.getElementById('last-updated');

const noop = () => {};

const editor = CodeMirror.fromTextArea(source, {
  mode: 'monkey',
  theme: 'monkey',
  tabSize: 2,
  lineNumbers: true,
  lineWrapping: true,
  smartIndent: true,
});

export const Command = {
  set: (value) => {
    editor.setValue(value);
  },

  run: () => {
    if (!Module.isReady()) {
      return;
    }

    const value = Command.getValue();

    Command.print(value === '' ? '' : Module.eval(value));
  },

  getValue: () => editor.getValue(),
  setValue: (value) => editor.setValue(value),

  print: (str) => {
    const now = new Date();
    const time = date2time(now);
    lastUpdated.textContent = `LAST UPDATE: ${time}`;
    lastUpdated.dateTime = now.toISOString();
    lastUpdated.style.display = 'block';

    output.innerHTML += escape(`${str}\n`);

    outputContainer.scrollTop = outputContainer.scrollHeight - outputContainer.clientHeight;
  },

  clear: () => {
    lastUpdated.dateTime = '';
    lastUpdated.style.display = 'none';
    output.innerHTML = '';
    outputContainer.scrollTop = 0;
  },

  format: () => {
    const result = Module.format(Command.getValue());

    if (result !== '') {
      Command.setValue(result);
    }
  },
};

const query = new window.URLSearchParams(window.location.search);

if (query.has(SHARE_QUERY_KEY)) {
  Command.setValue(LZString.decompressFromEncodedURIComponent(query.get(SHARE_QUERY_KEY)));
  snippet.selectedIndex = 0;
} else {
  Command.setValue(SNIPPETS[0].value);
}

editor.addKeyMap({
  'Ctrl-Enter': noop,
  'Shift-Enter': noop,
  'Ctrl-L': noop,
});

document.addEventListener(
  'keydown',
  (e) => {
    const { ctrlKey, metaKey, shiftKey, key: raw } = e;
    const key = raw.toLowerCase();

    // Ctrl + Enter
    if (ctrlKey && key === 'enter') {
      Command.run();
    }

    // Shift + Enter
    if (shiftKey && key === 'enter') {
      Command.format();
    }

    // Ctrl + L
    if (ctrlKey && key === 'l') {
      Command.clear();
    }
  },
  false,
);

run.addEventListener(
  'click',
  (e) => {
    e.preventDefault();
    Command.run();
  },
  false,
);
