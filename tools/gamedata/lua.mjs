/**
 * Parser mínimo de literais Lua: tabelas, strings (inclusive longas `[[...]]`), números, booleanos e nil.
 * Cobre os arquivos de dados do Path of Building (`return { ... }` e `itemBases["X"] = { ... }`),
 * que são só tabelas literais — sem precisar de um interpretador Lua.
 *
 * Tabelas viram `{ list: [...posicionais], map: {chave: valor} }`.
 */

export class LuaReader {
  constructor(src) {
    this.src = src;
    this.i = 0;
  }

  error(msg) {
    const line = this.src.slice(0, this.i).split('\n').length;
    return new Error(`lua: ${msg} (linha ${line})`);
  }

  skip() {
    const s = this.src;
    for (;;) {
      while (this.i < s.length && /\s/.test(s[this.i])) this.i++;
      if (s.startsWith('--', this.i)) {
        const long = /^--\[(=*)\[/.exec(s.slice(this.i, this.i + 10));
        if (long) {
          const close = `]${long[1]}]`;
          const end = s.indexOf(close, this.i);
          this.i = end < 0 ? s.length : end + close.length;
        } else {
          const nl = s.indexOf('\n', this.i);
          this.i = nl < 0 ? s.length : nl + 1;
        }
        continue;
      }
      return;
    }
  }

  peek() {
    this.skip();
    return this.src[this.i];
  }

  expect(ch) {
    if (this.peek() !== ch) throw this.error(`esperava '${ch}', achou '${this.src[this.i]}'`);
    this.i++;
  }

  /** Posiciona logo após `return` e lê o valor. */
  readReturn() {
    const at = this.src.search(/\breturn\b/);
    if (at < 0) throw new Error('lua: sem return');
    this.i = at + 'return'.length;
    return this.value();
  }

  value() {
    const c = this.peek();
    const s = this.src;
    if (c === '{') return this.table();
    if (c === '"' || c === "'") return this.quoted();
    if (c === '[') {
      const long = this.longString();
      if (long !== null) return long;
    }
    if (c === '-' || /[0-9.]/.test(c)) return this.number();
    const ident = /^[A-Za-z_][A-Za-z0-9_]*/.exec(s.slice(this.i, this.i + 64));
    if (ident) {
      this.i += ident[0].length;
      if (ident[0] === 'true') return true;
      if (ident[0] === 'false') return false;
      if (ident[0] === 'nil') return null;
      // Expressões (variáveis, chamadas) não são dados: devolve o nome cru.
      return { ident: ident[0] };
    }
    throw this.error(`valor inesperado '${c}'`);
  }

  longString() {
    const m = /^\[(=*)\[/.exec(this.src.slice(this.i, this.i + 10));
    if (!m) return null;
    const close = `]${m[1]}]`;
    const start = this.i + m[0].length;
    const end = this.src.indexOf(close, start);
    if (end < 0) throw this.error('string longa sem fim');
    this.i = end + close.length;
    // Lua descarta a primeira quebra de linha logo após a abertura.
    return this.src.slice(start, end).replace(/^\r?\n/, '');
  }

  quoted() {
    const q = this.src[this.i++];
    let out = '';
    while (this.i < this.src.length) {
      const c = this.src[this.i++];
      if (c === q) return out;
      if (c === '\\') {
        const n = this.src[this.i++];
        out += n === 'n' ? '\n' : n === 't' ? '\t' : n;
      } else {
        out += c;
      }
    }
    throw this.error('string sem fim');
  }

  number() {
    const m = /^-?(0x[0-9a-fA-F]+|\d*\.?\d+(?:[eE][-+]?\d+)?)/.exec(this.src.slice(this.i, this.i + 40));
    if (!m) throw this.error('número inválido');
    this.i += m[0].length;
    return Number(m[0]);
  }

  table() {
    this.expect('{');
    const list = [];
    const map = {};
    for (;;) {
      const c = this.peek();
      if (c === '}') {
        this.i++;
        return { list, map };
      }
      if (c === '[' && this.src[this.i + 1] !== '[' && this.src[this.i + 1] !== '=') {
        this.i++;
        const key = this.value();
        this.expect(']');
        this.expect('=');
        map[String(key)] = this.value();
      } else {
        const named = /^([A-Za-z_][A-Za-z0-9_]*)\s*=(?!=)/.exec(this.src.slice(this.i, this.i + 80));
        if (named) {
          this.i += named[0].length;
          map[named[1]] = this.value();
        } else {
          list.push(this.value());
        }
      }
      const sep = this.peek();
      if (sep === ',' || sep === ';') this.i++;
    }
  }
}

/** `return { ... }` → valor. */
export function parseReturn(src) {
  return new LuaReader(src).readReturn();
}

/** Todas as atribuições `prefix["chave"] = { ... }` do arquivo (ex.: `itemBases`). */
export function parseAssignments(src, prefix) {
  const out = {};
  const re = new RegExp(`${prefix}\\["((?:[^"\\\\]|\\\\.)*)"\\]\\s*=`, 'g');
  let m;
  while ((m = re.exec(src))) {
    const reader = new LuaReader(src);
    reader.i = re.lastIndex;
    out[m[1].replace(/\\(.)/g, '$1')] = reader.value();
    re.lastIndex = reader.i;
  }
  return out;
}

/** Lista posicional de uma tabela (ou `[]`). */
export const listOf = (t) => (t && Array.isArray(t.list) ? t.list : []);
/** Parte nomeada de uma tabela (ou `{}`). */
export const mapOf = (t) => (t && t.map ? t.map : {});
