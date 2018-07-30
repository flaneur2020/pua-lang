export const Module = {
  _memory: null,
  _alloc: null,
  _dealloc: null,
  _eval: null,
  _format: null,
  _textEncoder: new TextEncoder('UTF-8'),
  _textDecoder: new TextDecoder('UTF-8'),

  isReady: () => Module._memory != null,

  load: async (path, env = {}) => {
    try {
      const imports = {
        env,
      };

      const {
        instance: { exports },
      } = await WebAssembly.instantiateStreaming(fetch(path), imports);

      Module._memory = exports.memory;
      Module._alloc = exports.alloc;
      Module._dealloc = exports.dealloc;
      Module._eval = exports.eval;
      Module._format = exports.format;
    } catch (e) {
      console.error(e);
    }
  },

  allocStr: (str) => {
    const buf = Module._textEncoder.encode(str);
    const ptr = Module._alloc(buf.length + 1);
    const heap = new Uint8Array(Module._memory.buffer);

    for (let i = 0; i < buf.length; i++) {
      heap[ptr + i] = buf[i];
    }

    heap[ptr + buf.length] = 0;

    return { buf, ptr };
  },

  dealloc: (ptr, len) => {
    Module._dealloc(ptr);
  },

  copyCStr: (ptr) => {
    const buf = new Uint8Array(Module._memory.buffer, ptr);

    let i = 0;
    while (buf[i] !== 0) {
      if (buf[i] === undefined) {
        throw new Error('Access to memory that does not exist!!');
      }
      i++;
    }

    return Module._textDecoder.decode(buf.slice(0, i));
  },

  eval: (str) => {
    if (!Module.isReady()) return;
    const { buf, ptr } = Module.allocStr(str);
    const resultPtr = Module._eval(ptr);
    Module.dealloc(resultPtr, buf.length);
    return Module.copyCStr(resultPtr);
  },

  format: (str) => {
    if (!Module.isReady()) return;
    const { buf, ptr } = Module.allocStr(str);
    const resultPtr = Module._format(ptr);
    Module.dealloc(resultPtr, buf.length);
    return Module.copyCStr(resultPtr);
  },
};
