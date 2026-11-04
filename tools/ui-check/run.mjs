#!/usr/bin/env node
// UI check tool: drives the Baytar window through WebView2's remote debugging port (CDP).
// Node 24+, no npm packages. See README.md.
import { spawn, spawnSync } from 'node:child_process';
import { readFileSync, mkdirSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, resolve, isAbsolute } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, '..', '..');
const outDir = resolve(here, 'out');

function parseArgs(argv) {
  const a = { port: 9333, scenario: 'smoke', exe: null, startupTimeout: 300000 };
  for (let i = 0; i < argv.length; i++) {
    const k = argv[i];
    const v = () => argv[++i];
    if (k === '--port') a.port = Number(v());
    else if (k === '--scenario') a.scenario = v();
    else if (k === '--exe') a.exe = v();
    else if (k === '--startup-timeout') a.startupTimeout = Number(v()) * 1000;
    else if (k === '--help' || k === '-h') a.help = true;
    else throw new Error(`unknown argument: ${k}`);
  }
  return a;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const log = (m) => console.log(m);

async function getJson(url) {
  const r = await fetch(url, { signal: AbortSignal.timeout(2000) });
  return r.json();
}

async function findPageTarget(port) {
  try {
    const list = await getJson(`http://127.0.0.1:${port}/json`);
    return list.find((t) => t.type === 'page' && t.webSocketDebuggerUrl && !t.url.startsWith('about:')) ?? null;
  } catch {
    return null;
  }
}

class Cdp {
  constructor(ws) {
    this.ws = ws;
    this.id = 0;
    this.pending = new Map();
    ws.addEventListener('message', (ev) => {
      const msg = JSON.parse(ev.data);
      if (msg.id && this.pending.has(msg.id)) {
        const { resolve, reject } = this.pending.get(msg.id);
        this.pending.delete(msg.id);
        msg.error ? reject(new Error(msg.error.message)) : resolve(msg.result);
      }
    });
  }
  static connect(url) {
    return new Promise((res, rej) => {
      const ws = new WebSocket(url);
      ws.addEventListener('open', () => res(new Cdp(ws)));
      ws.addEventListener('error', () => rej(new Error('CDP websocket error')));
    });
  }
  send(method, params = {}) {
    const id = ++this.id;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify({ id, method, params }));
    });
  }
  async eval(expr) {
    const r = await this.send('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    return r.result.value;
  }
  close() {
    try { this.ws.close(); } catch { /* ignore */ }
  }
}

// In-page finder: a CSS selector, or the smallest visible element containing the text.
const finderJs = `(sel, text) => {
  const visible = (el) => { const r = el.getBoundingClientRect(); const s = getComputedStyle(el);
    return r.width > 0 && r.height > 0 && s.visibility !== 'hidden' && s.display !== 'none'; };
  if (sel) return [...document.querySelectorAll(sel)].find(visible) ?? null;
  let best = null;
  for (const el of document.querySelectorAll('body *')) {
    if (!visible(el) || !(el.textContent || '').includes(text)) continue;
    if (!best || best.contains(el)) best = el;
  }
  return best;
}`;
const q = (step) => `(${finderJs})(${JSON.stringify(step.selector ?? null)}, ${JSON.stringify(step.text ?? null)})`;
const describe = (s) => (s.selector ? `selector ${s.selector}` : `text "${s.text}"`);

async function runStep(cdp, step) {
  switch (step.type) {
    case 'wait': {
      const deadline = Date.now() + (step.timeout ?? 10000);
      while (Date.now() < deadline) {
        if (await cdp.eval(`!!${q(step)}`)) return `found ${describe(step)}`;
        await sleep(200);
      }
      throw new Error(`timeout waiting for ${describe(step)}`);
    }
    case 'click': {
      const ok = await cdp.eval(`(() => { const el = ${q(step)}; if (!el) return false; el.click(); return true; })()`);
      if (!ok) throw new Error(`nothing to click for ${describe(step)}`);
      return `clicked ${describe(step)}`;
    }
    case 'eval': {
      const v = await cdp.eval(step.expr);
      if ('expect' in step && JSON.stringify(v) !== JSON.stringify(step.expect)) {
        throw new Error(`eval returned ${JSON.stringify(v)}, expected ${JSON.stringify(step.expect)}`);
      }
      return `eval => ${JSON.stringify(v)}`;
    }
    case 'screenshot': {
      const r = await cdp.send('Page.captureScreenshot', { format: 'png' });
      const p = isAbsolute(step.path) ? step.path : resolve(outDir, step.path);
      mkdirSync(dirname(p), { recursive: true });
      writeFileSync(p, Buffer.from(r.data, 'base64'));
      return `saved ${p}`;
    }
    case 'reload': {
      await cdp.send('Page.reload', { ignoreCache: true });
      await sleep(500);
      const deadline = Date.now() + 30000;
      while (Date.now() < deadline) {
        try {
          if ((await cdp.eval('document.readyState')) === 'complete') return 'reloaded';
        } catch { /* context is being replaced */ }
        await sleep(200);
      }
      throw new Error('page did not finish reloading');
    }
    case 'localStorage.clear':
      await cdp.eval('localStorage.clear()');
      return 'localStorage cleared';
    default:
      throw new Error(`unknown step type: ${step.type}`);
  }
}

// Kill only the process tree we started, by PID (never by image name).
function killTree(pid) {
  if (!pid) return;
  spawnSync('taskkill', ['/PID', String(pid), '/T', '/F'], { stdio: 'ignore' });
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    log('usage: node tools/ui-check/run.mjs [--scenario smoke] [--port 9333] [--exe <path>] [--startup-timeout <seconds>]');
    return 0;
  }
  const name = args.scenario.endsWith('.json') ? args.scenario : `${args.scenario}.json`;
  const scenarioPath = resolve(here, 'scenarios', name);
  if (!existsSync(scenarioPath)) throw new Error(`scenario not found: ${scenarioPath}`);
  const scenario = JSON.parse(readFileSync(scenarioPath, 'utf8'));

  if (await findPageTarget(args.port)) {
    throw new Error(`port ${args.port} is already served by a debuggable WebView; pick another --port`);
  }

  // Own user data folder per port: fresh localStorage, no clash with a running instance.
  const userData = resolve(outDir, `webview2-${args.port}`);
  mkdirSync(userData, { recursive: true });
  const env = {
    ...process.env,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${args.port}`,
    WEBVIEW2_USER_DATA_FOLDER: userData,
  };
  const child = args.exe
    ? spawn(args.exe, [], { env, stdio: 'ignore' })
    : spawn('npm run tauri dev', { cwd: resolve(repoRoot, 'app'), env, shell: true, stdio: 'ignore' });
  log(`[start] ${args.exe ?? 'npm run tauri dev'} (pid ${child.pid}, debug port ${args.port})`);
  let exited = false;
  child.on('exit', (c) => {
    exited = true;
    log(`[proc] launched process exited (${c})`);
  });

  let cdp;
  let failed = false;
  try {
    const deadline = Date.now() + args.startupTimeout;
    let target = null;
    while (Date.now() < deadline && !target) {
      if (exited && args.exe) throw new Error('app exited before the window appeared');
      target = await findPageTarget(args.port);
      if (!target) await sleep(1000);
    }
    if (!target) throw new Error(`no page target on port ${args.port} within ${args.startupTimeout / 1000}s`);
    log(`[window] ${target.title || '(untitled)'} ${target.url}`);
    cdp = await Cdp.connect(target.webSocketDebuggerUrl);
    await cdp.send('Page.enable');
    await cdp.send('Runtime.enable');
    const ready = `location.href !== 'about:blank' && document.readyState === 'complete' && !!document.body`;
    for (let t = 0; t < 300 && !(await cdp.eval(ready)); t++) await sleep(200);

    const steps = scenario.steps;
    for (let i = 0; i < steps.length; i++) {
      const label = `[${i + 1}/${steps.length}] ${steps[i].type}`;
      try {
        log(`${label}: ${await runStep(cdp, steps[i])}`);
      } catch (e) {
        log(`${label}: FAIL ${e.message}`);
        try {
          log(`  ${await runStep(cdp, { type: 'screenshot', path: 'failure.png' })}`);
        } catch { /* best effort */ }
        failed = true;
        break;
      }
    }
  } catch (e) {
    log(`[error] ${e.message}`);
    failed = true;
  } finally {
    cdp?.close();
    log(`[stop] killing process tree of pid ${child.pid}`);
    killTree(child.pid);
  }
  log(failed ? 'RESULT: FAIL' : 'RESULT: PASS');
  return failed ? 1 : 0;
}

main().then(
  (c) => process.exit(c),
  (e) => {
    console.error(`[fatal] ${e.message}`);
    process.exit(2);
  },
);
