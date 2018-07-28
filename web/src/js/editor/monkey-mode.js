import CodeMirror from 'codemirror';
import 'codemirror/addon/mode/simple';

CodeMirror.defineSimpleMode('monkey', {
  start: [
    { regex: /".*"/, token: 'string' },
    { regex: /(?:fn|let|return|if|else)\b/, token: 'keyword' },
    { regex: /true|false|null/, token: 'atom' },
    { regex: /\d+|[-+]?(?:\.\d+|\d+\.?\d*)/, token: 'number' },
    { regex: /[-+\/*=<>!]/, token: 'operator' },
    { regex: /[\{\[\(]/, indent: true },
    { regex: /[\}\]\)]/, dedent: true },
    { regex: /[a-z$][\w$]*/, token: 'variable' },
  ],
  comment: [],
  meta: {},
});
