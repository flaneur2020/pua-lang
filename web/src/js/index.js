import { Icon } from './custom-elements/x-icon';
import { Module } from './module';
import { Command } from './editor';
import '../css/style.css';

customElements.define('x-icon', Icon);

Module.load('./pua-lang.wasm', {
  print: (value) => {
    const str = Module.copyCStr(value);
    console.log(str);
    Command.print(str);
  },
}).catch((e) => console.error(e));
