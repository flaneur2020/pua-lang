import CodeMirror from 'codemirror';
import 'codemirror/addon/mode/simple';

// This is still profoundly broken.
// * The keyword part barely works. I have no idea why \b is doing that, but at least it's acting the same way as the regex.
// * The operator and the Unicode identifier patterns just don't work. I have no idea why. The regexes match!
CodeMirror.defineSimpleMode('monkey', {
  start: [
    { regex: /".*"/, token: 'string' },
    {
      regex: /(?:fn|let|return|if|else|抓手|赋能|细分|路径|反哺)(?:\b|(?=\s|[()]))/,
      token: 'keyword',
    },
    { regex: /true|false|null|三七五|三二五/, token: 'atom' },
    { regex: /\d+|[-+]?(?:\.\d+|\d+\.?\d*)/, token: 'number' },
    { regex: /[-+\/*=<>!]|对齐|联动|差异|倾斜/, token: 'operator' },
    { regex: /[\{\[\(]/, indent: true },
    { regex: /[\}\]\)]/, dedent: true },
    { regex: /[\p{XID_Start}$¥_][\p{XID_Continue}$¥]*/u, token: 'variable' },
  ],
  comment: [],
  meta: {},
});
