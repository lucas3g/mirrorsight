#!/usr/bin/env node
/**
 * Gera os prints dos READMEs a partir do modo demo, um conjunto por idioma da interface:
 *   docs/screenshots/<idioma>/<tela>.png   (1440×900 @1.5x = 2160×1350)
 *
 * Compila o frontend (`npm run build` em apps/desktop), sobe o `vite preview` e dirige o
 * Edge/Chrome headless via DevTools Protocol. Os cliques usam seletores da estrutura
 * (não o texto dos botões), então funcionam em qualquer idioma.
 *
 * Uso:  node tools/screenshots/capture.mjs [--locales en,pt-BR] [--browser <exe>] [--no-build]
 * Requer Node ≥ 22 (WebSocket nativo) e Edge ou Chrome instalado.
 */
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const APP = join(ROOT, 'apps', 'desktop');
const OUT = join(ROOT, 'docs', 'screenshots');

/** Os mesmos idiomas de `LOCALES` (apps/desktop/src/lib/i18n/core.svelte.ts). */
const ALL_LOCALES = ['en', 'pt-BR', 'es', 'de', 'zh-CN', 'ja', 'ko'];
const VIEWPORT = { width: 1440, height: 900, deviceScaleFactor: 1.5, mobile: false };
const PREVIEW_PORT = 4179;
const CDP_PORT = 9349;
/** Build do PoB do demo usada no veredito de upgrade (ver DEMO_BUILDS em demo-fixtures.ts). */
const DEMO_BUILD = 'demo://RF 3.27.xml';
/** Vendedor do demo cujo anúncio mostra o "excelente, mas pior para a build". */
const PEEK_SELLER = 'Oriath_Merchant';

const args = process.argv.slice(2);
const opt = (name) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const locales = (opt('--locales')?.split(',') ?? ALL_LOCALES).map((l) => l.trim());
for (const l of locales) if (!ALL_LOCALES.includes(l)) throw new Error(`idioma desconhecido: ${l}`);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function findBrowser() {
  const explicit = opt('--browser');
  if (explicit) return explicit;
  const pf = [process.env['ProgramFiles(x86)'], process.env.ProgramFiles, process.env.LOCALAPPDATA].filter(Boolean);
  const candidates = pf.flatMap((dir) => [
    join(dir, 'Microsoft', 'Edge', 'Application', 'msedge.exe'),
    join(dir, 'Google', 'Chrome', 'Application', 'chrome.exe'),
  ]);
  candidates.push('/usr/bin/google-chrome', '/usr/bin/chromium', '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome');
  const hit = candidates.find((p) => existsSync(p));
  if (!hit) throw new Error('Edge/Chrome não encontrado; informe com --browser <exe>');
  return hit;
}

async function waitHttp(url, timeoutMs) {
  const end = Date.now() + timeoutMs;
  while (Date.now() < end) {
    try {
      const res = await fetch(url);
      if (res.ok) return res;
    } catch {}
    await sleep(250);
  }
  throw new Error(`sem resposta de ${url}`);
}

/** Conexão CDP mínima com a primeira aba do navegador. */
async function connect() {
  const res = await waitHttp(`http://127.0.0.1:${CDP_PORT}/json`, 15_000);
  const target = (await res.json()).find((t) => t.type === 'page');
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((r, j) => ((ws.onopen = r), (ws.onerror = j)));
  let id = 0;
  const pending = new Map();
  const errors = [];
  ws.onmessage = (m) => {
    const d = JSON.parse(m.data);
    if (d.id && pending.has(d.id)) {
      pending.get(d.id)(d);
      pending.delete(d.id);
    }
    if (d.method === 'Runtime.exceptionThrown') errors.push(d.params.exceptionDetails?.exception?.description ?? 'erro de JS');
  };
  const send = (method, params = {}) =>
    new Promise((r) => {
      const i = ++id;
      pending.set(i, r);
      ws.send(JSON.stringify({ id: i, method, params }));
    });
  const ev = async (expression) => {
    const r = await send('Runtime.evaluate', { expression, awaitPromise: true });
    if (r.result?.exceptionDetails) throw new Error(`falha no script da página: ${expression.slice(0, 120)}`);
    return r.result?.result?.value;
  };
  await send('Runtime.enable');
  await send('Page.enable');
  await send('Emulation.setDeviceMetricsOverride', VIEWPORT);
  return { send, ev, errors, close: () => ws.close() };
}

async function captureLocale(page, base, locale) {
  const { send, ev } = page;
  const dir = join(OUT, locale);
  mkdirSync(dir, { recursive: true });

  const shot = async (name) => {
    const r = await send('Page.captureScreenshot', { format: 'png' });
    writeFileSync(join(dir, `${name}.png`), Buffer.from(r.result.data, 'base64'));
    console.log(`  ${locale}/${name}.png`);
  };
  const waitFor = async (selector, timeoutMs = 8000) => {
    const end = Date.now() + timeoutMs;
    while (Date.now() < end) {
      if (await ev(`!!document.querySelector(${JSON.stringify(selector)})`)) return;
      await sleep(100);
    }
    throw new Error(`${locale}: não apareceu ${selector}`);
  };
  /** Clica no n-ésimo elemento do seletor. */
  const click = async (selector, n = 0) => {
    const ok = await ev(`(()=>{const el=document.querySelectorAll(${JSON.stringify(selector)})[${n}];if(!el)return false;el.click();return true})()`);
    if (!ok) throw new Error(`${locale}: não achei ${selector}[${n}]`);
  };
  /** Esc no elemento focado (a paleta escuta no próprio diálogo) e na janela. */
  const escape = async () => {
    await ev(`(document.activeElement||document.body).dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}))`);
    await sleep(400);
  };
  const peek = (type) =>
    ev(`(()=>{const r=[...document.querySelectorAll('.listing')].find(r=>r.textContent.includes(${JSON.stringify(PEEK_SELLER)}));const b=r?.querySelector('.peek')||r?.querySelector('button');b?.dispatchEvent(new MouseEvent('${type}',{bubbles:false}));return !!b})()`);

  // Estado limpo: só o idioma e a build do demo.
  await send('Page.navigate', { url: base });
  await waitFor('.demo button');
  await ev(`localStorage.clear();localStorage.setItem('mirrorsight.demo.settings',JSON.stringify({ui_language:${JSON.stringify(locale)},build_path:${JSON.stringify(DEMO_BUILD)}}))`);
  await send('Page.reload');
  await waitFor('.demo button');
  await sleep(800);

  // Price check de item raro + avaliação (olho) + distribuição.
  await click('.demo button', 0);
  await waitFor('.listing');
  await sleep(1200);
  await shot('price-check');
  if (!(await peek('mouseenter'))) throw new Error(`${locale}: anúncio de ${PEEK_SELLER} não encontrado`);
  await sleep(800);
  await shot('evaluation');
  await peek('mouseleave');
  await sleep(400);
  await click('.market [role=tab]', 1);
  await sleep(1200);
  await shot('distribution');
  await escape();

  // Currency em bulk e quick price.
  await click('.demo button', 1);
  await sleep(1800);
  await shot('bulk-currency');
  await escape();
  await click('.demo button', 2);
  await sleep(1600);
  await shot('quick-price');
  await escape();

  // Histórico e paleta de comandos por cima dele.
  await click('.rail button.nav', 1);
  await sleep(900);
  await shot('history');
  await ev(`window.dispatchEvent(new KeyboardEvent('keydown',{key:'k',ctrlKey:true,bubbles:true}))`);
  await sleep(700);
  await shot('command-palette');
  await escape();

  // Configurações: Geral, Price check (build do PoB) e Hotkeys.
  await click('.rail button.nav', 2);
  await waitFor('.nav-item');
  await sleep(700);
  await shot('settings-general');
  await click('.nav-item', 2);
  await sleep(800);
  await shot('settings-price-check');
  await click('.nav-item', 1);
  await sleep(600);
  await shot('settings-hotkeys');
}

// ── main ────────────────────────────────────────────────────────────────────
if (!args.includes('--no-build')) {
  console.log('compilando o frontend…');
  const r = spawnSync('npm', ['run', 'build'], { cwd: APP, stdio: 'inherit', shell: process.platform === 'win32' });
  if (r.status !== 0) process.exit(r.status ?? 1);
}

const vite = join(APP, 'node_modules', 'vite', 'bin', 'vite.js');
const preview = spawn(process.execPath, [vite, 'preview', '--port', String(PREVIEW_PORT), '--strictPort'], { cwd: APP, stdio: 'ignore' });
const profile = mkdtempSync(join(tmpdir(), 'ms-shots-'));
const browser = spawn(findBrowser(), [
  '--headless=new',
  `--remote-debugging-port=${CDP_PORT}`,
  `--user-data-dir=${profile}`,
  '--hide-scrollbars',
  'about:blank',
]);

let failed = false;
let page;
try {
  const base = `http://localhost:${PREVIEW_PORT}/`;
  await waitHttp(base, 20_000);
  page = await connect();
  for (const locale of locales) {
    console.log(`capturando ${locale}…`);
    await captureLocale(page, base, locale);
  }
  if (page.errors.length) {
    failed = true;
    console.error('erros de JS na página:\n' + page.errors.join('\n'));
  }
} catch (e) {
  failed = true;
  console.error(e.message ?? e);
} finally {
  page?.close();
  browser.kill();
  preview.kill();
  await sleep(500);
  rmSync(profile, { recursive: true, force: true, maxRetries: 5 });
}
process.exit(failed ? 1 : 0);
