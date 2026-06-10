<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { Menu, Submenu, MenuItem } from '@tauri-apps/api/menu';
  import { open } from '@tauri-apps/plugin-dialog';
  import { openPath } from '@tauri-apps/plugin-opener';
  import FileDropZone from '$lib/FileDropZone.svelte';
  import { isPdf, basename, formatSize, formatDate } from '$lib/utils';

  type Stat = { size: number; modified_ms: number };

  let files: string[] = $state([]);
  let stats: Record<string, Stat> = $state({});

  function addFiles(paths: string[]) {
    const set = new Set(files);
    const fresh: string[] = [];
    for (const p of paths) {
      if (isPdf(p) && !set.has(p)) {
        set.add(p);
        fresh.push(p);
      }
    }
    files = [...set];
    for (const p of fresh) {
      invoke<Stat>('file_stat', { path: p })
        .then((s) => {
          stats = { ...stats, [p]: s };
        })
        .catch(() => {});
    }
  }

  function remove(path: string) {
    files = files.filter((p) => p !== path);
    const { [path]: _s, ...restStats } = stats;
    stats = restStats;
    const { [path]: _st, ...restStatuses } = statuses;
    statuses = restStatuses;
    const { [path]: _e, ...restErrors } = errors;
    errors = restErrors;
  }

  function sortFiles() {
    files = [...files].sort((a, b) =>
      basename(a).localeCompare(basename(b)),
    );
  }

  function clearFiles() {
    files = [];
    stats = {};
    statuses = {};
    errors = {};
  }

  type Rule = { id: string; page: string; lineWidth: number };
  let rules: Rule[] = $state([]);
  type OriginalMode = 'rename' | 'delete';
  let originalMode: OriginalMode = $state('delete');
  let renameSuffix = $state('_old');
  let randomColors = $state(false);
  let saveLog = $state(false);

  // Hairlines settings (matches reference UI screenshot)
  const UNITS = ['Points', 'Picas', 'Milimiters', 'Centimeters', 'Inches'] as const;
  type Unit = (typeof UNITS)[number];
  let hairlineThreshold = $state(0);
  let hairlineReplace = $state(0);
  let units: Unit = $state('Milimiters');
  let includeType3 = $state(false);
  let includePatterns = $state(false);
  let ignoreFilledShapes = $state(true);

  // Page range
  type PageRangeMode = 'all' | 'current' | 'range' | 'rule';
  let pageRangeMode: PageRangeMode = $state('current');
  let pageRangeCurrent: number | null = $state(1);
  let pageRangeFrom: number | null = $state(null);
  let pageRangeTo: number | null = $state(null);

  // Unit suffix shown in the rules table header.
  const UNIT_ABBREV: Record<Unit, string> = {
    Points: 'pt',
    Picas: 'pc',
    Milimiters: 'mm',
    Centimeters: 'cm',
    Inches: 'in',
  };

  type Defaults = {
    hairlines: {
      narrowerThan: number;
      replaceWith: number;
      units: Unit;
      includeType3: boolean;
      includePatterns: boolean;
      ignoreFilledShapes: boolean;
    };
    pageRange: { mode: PageRangeMode; current: number | null; from: number | null; to: number | null };
    rules: { items: { page: string; lineWidth: number }[] };
    originalMode: OriginalMode;
    renameSuffix: string;
    randomColors: boolean;
    saveLog: boolean;
  };

  function applyDefaults(d: Defaults) {
    hairlineThreshold = d.hairlines.narrowerThan;
    hairlineReplace = d.hairlines.replaceWith;
    units = d.hairlines.units;
    includeType3 = d.hairlines.includeType3;
    includePatterns = d.hairlines.includePatterns;
    ignoreFilledShapes = d.hairlines.ignoreFilledShapes ?? true;
    pageRangeMode = d.pageRange.mode;
    pageRangeCurrent = d.pageRange.current ?? 1;
    pageRangeFrom = d.pageRange.from;
    pageRangeTo = d.pageRange.to;
    rules = d.rules.items.map((r) => ({
      id: nextId(),
      page: r.page,
      lineWidth: r.lineWidth,
    }));
    originalMode = d.originalMode;
    renameSuffix = d.renameSuffix;
    randomColors = d.randomColors;
    saveLog = d.saveLog;
  }

  function snapshotSettings(): Defaults {
    return {
      hairlines: {
        narrowerThan: hairlineThreshold,
        replaceWith: hairlineReplace,
        units,
        includeType3,
        includePatterns,
        ignoreFilledShapes,
      },
      pageRange: {
        mode: pageRangeMode,
        current: pageRangeCurrent,
        from: pageRangeFrom,
        to: pageRangeTo,
      },
      rules: {
        items: rules.map((r) => ({ page: r.page, lineWidth: r.lineWidth })),
      },
      originalMode,
      renameSuffix,
      randomColors,
      saveLog,
    };
  }

  // Persistence: settings are written to disk on every change, debounced
  // so quick edits don't thrash. The first run after applyDefaults() is
  // skipped so we don't immediately re-save what we just loaded.
  let settingsLoaded = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  const SAVE_DEBOUNCE_MS = 500;

  $effect(() => {
    // Read all settings so Svelte tracks them as dependencies of this
    // effect. The snapshot also conveniently is what we'll persist.
    const snap = snapshotSettings();
    if (!settingsLoaded) return; // skip the very first run after load
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      invoke('save_user_settings', { settings: snap }).catch((e) =>
        console.error('save_user_settings failed:', e),
      );
    }, SAVE_DEBOUNCE_MS);
  });

  function nextId(): string {
    return crypto.randomUUID();
  }

  function addRule() {
    rules = [...rules, { id: nextId(), page: '1', lineWidth: 0.25 }];
  }

  function removeRule(id: string) {
    rules = rules.filter((r) => r.id !== id);
  }

  // Edit > Reset settings to defaults: deletes the user's persisted
  // settings.json and reloads bundled defaults into the form.
  async function resetSettings() {
    try {
      const d = await invoke<Defaults>('reset_user_settings');
      applyDefaults(d);
    } catch (e) {
      console.error('reset_user_settings failed:', e);
    }
  }

  type FileStatus = 'pending' | 'converting' | 'done' | 'error';
  type ProgressEvent = {
    index: number;
    total: number;
    path: string;
    file: string;
    phase: 'start' | 'done' | 'error';
    error?: string;
    output?: string;
  };

  let converting = $state(false);
  let convertMessage = $state('');
  let statuses: Record<string, FileStatus> = $state({});
  let errors: Record<string, string> = $state({});
  let progress: { current: number; total: number; file: string } | null =
    $state(null);

  async function convert() {
    if (!files.length || converting) return;
    converting = true;
    convertMessage = '';
    errors = {};
    const init: Record<string, FileStatus> = {};
    for (const p of files) init[p] = 'pending';
    statuses = init;
    progress = { current: 0, total: files.length, file: '' };

    try {
      const out = await invoke<string[]>('convert_pdfs', {
        paths: files,
        options: {
          originalMode,
          suffix: renameSuffix,
          randomColors,
          saveLog,
          hairlineThreshold: hairlineThreshold,
          replaceWith: hairlineReplace,
          units,
          pageMode: pageRangeMode,
          currentPage: pageRangeCurrent,
          rangeFrom: pageRangeFrom,
          rangeTo: pageRangeTo,
          rules: rules.map((r) => ({ page: r.page, lineWidth: r.lineWidth })),
          ignoreFilledShapes,
        },
      });
      const failed = files.length - out.length;
      convertMessage =
        failed === 0
          ? `Converted ${out.length}/${files.length}.`
          : `Converted ${out.length}/${files.length} (${failed} error${failed === 1 ? '' : 's'}).`;
    } catch (e) {
      convertMessage = `Error: ${e}`;
    } finally {
      converting = false;
      progress = null;
    }
  }

  async function openFile(path: string) {
    try {
      await openPath(path);
    } catch (e) {
      console.error('open failed:', e);
    }
  }

  function viewerLabel(): string {
    return `viewer-${crypto.randomUUID().replace(/-/g, '').slice(0, 12)}`;
  }

  async function openInViewer(path: string) {
    try {
      const url = convertFileSrc(path);
      const label = viewerLabel();
      const win = new WebviewWindow(label, {
        url,
        title: basename(path),
        width: 1000,
        height: 1200,
      });

      win.once('tauri://error', (e) => {
        console.error('viewer window error:', e.payload);
        openFile(path);
      });

      win.once('tauri://created', async () => {
        try {
          const closeItem = await MenuItem.new({
            id: `close::${label}`,
            text: 'Close',
            action: () => {
              win.close().catch((err) => console.error('close failed:', err));
            },
          });
          const fileMenu = await Submenu.new({
            text: 'File',
            items: [closeItem],
          });
          const menu = await Menu.new({ items: [fileMenu] });
          await menu.setAsWindowMenu(win);
        } catch (err) {
          console.error('viewer menu attach failed:', err);
        }
      });
    } catch (e) {
      console.error('viewer open failed, falling back to system:', e);
      openFile(path);
    }
  }

  function onNameClick(path: string) {
    if (statuses[path] === 'done') openInViewer(path);
    else openFile(path);
  }

  async function pickAndAdd() {
    const result = await open({ multiple: true });
    if (!result) return;
    const paths = Array.isArray(result) ? result : [result];
    if (paths.length) addFiles(paths);
  }

  type Theme = 'light' | 'dark';
  const THEME_KEY = 'pdfix:theme';

  function resolveInitialTheme(): Theme {
    try {
      const saved = localStorage.getItem(THEME_KEY);
      if (saved === 'light' || saved === 'dark') return saved;
    } catch {}
    return window.matchMedia?.('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light';
  }

  function applyTheme(theme: Theme) {
    document.documentElement.dataset.theme = theme;
    // Mirror to the menu so the label reflects the *next* action.
    const next = theme === 'dark' ? 'Light' : 'Dark';
    invoke('set_theme_menu_label', { label: `Switch to ${next} Mode` }).catch(
      () => {},
    );
  }

  function toggleTheme() {
    const current =
      (document.documentElement.dataset.theme as Theme | undefined) ??
      resolveInitialTheme();
    const next: Theme = current === 'dark' ? 'light' : 'dark';
    try {
      localStorage.setItem(THEME_KEY, next);
    } catch {}
    applyTheme(next);
  }

  onMount(() => {
    applyTheme(resolveInitialTheme());

    // Load chain: try persisted user settings first; on any failure
    // (missing file, corrupt JSON, schema drift) fall back to bundled
    // defaults so the UI is always usable.
    invoke<Defaults>('load_user_settings')
      .catch(() => invoke<Defaults>('load_defaults'))
      .then((d) => {
        applyDefaults(d);
        settingsLoaded = true;
      })
      .catch((e) => console.error('settings load failed:', e));

    const subs = [
      listen('menu:add-pdf', pickAndAdd),
      listen('menu:sort-list', sortFiles),
      listen('menu:clear-list', clearFiles),
      listen('menu:reset-settings', resetSettings),
      listen('menu:toggle-theme', toggleTheme),
      listen<ProgressEvent>('convert:progress', (e) => {
        const { index, total, path, file, phase, error } = e.payload;
        if (phase === 'start') {
          statuses = { ...statuses, [path]: 'converting' };
          progress = { current: index, total, file };
        } else if (phase === 'done') {
          statuses = { ...statuses, [path]: 'done' };
          progress = { current: index + 1, total, file };
          invoke<Stat>('file_stat', { path })
            .then((s) => {
              stats = { ...stats, [path]: s };
            })
            .catch(() => {});
        } else if (phase === 'error') {
          statuses = { ...statuses, [path]: 'error' };
          if (error) errors = { ...errors, [path]: error };
          progress = { current: index + 1, total, file };
        }
      }),
    ];
    return () => {
      subs.forEach((p) => p.then((u) => u()));
    };
  });
</script>

<main class="container">
  <h1>pdfix</h1>

  <FileDropZone onFiles={addFiles} label="Drop PDFs here" />

  <div class="settings-row">
  <fieldset class="group">
    <legend>Hairlines</legend>
    <div class="grid">
      <label for="hl-th">If line &lt;&equals; to</label>
      <input id="hl-th" type="number" step="0.1" bind:value={hairlineThreshold} />

      <label for="hl-rw" class:disabled={pageRangeMode === 'rule'}>Replace with</label>
      <input
        id="hl-rw"
        type="number"
        step="0.1"
        bind:value={hairlineReplace}
        disabled={pageRangeMode === 'rule'}
      />

      <label for="hl-u">Units</label>
      <select id="hl-u" bind:value={units}>
        {#each UNITS as u}
          <option value={u}>{u}</option>
        {/each}
      </select>
    </div>

    <div class="checks two-col">
      <div class="col">
        <label><input type="checkbox" bind:checked={includeType3} /> Include Type3 fonts</label>
        <label><input type="checkbox" bind:checked={includePatterns} /> Include Patterns</label>
        <label><input type="checkbox" bind:checked={ignoreFilledShapes} /> Ignore filled shapes</label>
      </div>
      <div class="col">
        <label><input type="checkbox" bind:checked={randomColors} /> Use random colors</label>
        <label><input type="checkbox" bind:checked={saveLog} /> Save log</label>
      </div>
    </div>
  </fieldset>

  <fieldset class="group">
    <legend>Page Range</legend>

    <div class="radios">
      <label class="radio">
        <input type="radio" name="pagerange" value="all" bind:group={pageRangeMode} />
        All Pages in document
      </label>

      <label class="radio">
        <input type="radio" name="pagerange" value="current" bind:group={pageRangeMode} />
        Current Page:
        <span class="inline">
          <input
            type="number"
            min="1"
            step="1"
            bind:value={pageRangeCurrent}
            disabled={pageRangeMode !== 'current'}
          />
        </span>
      </label>

      <label class="radio">
        <input type="radio" name="pagerange" value="range" bind:group={pageRangeMode} />
        Pages range:
        <span class="inline">
          From:
          <input
            type="number"
            min="1"
            step="1"
            bind:value={pageRangeFrom}
            disabled={pageRangeMode !== 'range'}
          />
          To:
          <input
            type="number"
            min="1"
            step="1"
            bind:value={pageRangeTo}
            disabled={pageRangeMode !== 'range'}
          />
        </span>
      </label>

      <label class="radio">
        <input type="radio" name="pagerange" value="rule" bind:group={pageRangeMode} />
        Use page rule:
      </label>
    </div>

    <div class="rules-block" class:disabled={pageRangeMode !== 'rule'}>
      <table class="rules-table">
        <thead>
          <tr>
            <th>Pages</th>
            <th>Line Width ({UNIT_ABBREV[units]})</th>
            <th aria-label="Remove"></th>
          </tr>
        </thead>
        <tbody>
          {#each rules as rule (rule.id)}
            <tr>
              <td>
                <input
                  type="text"
                  bind:value={rule.page}
                  disabled={pageRangeMode !== 'rule'}
                />
              </td>
              <td>
                <input
                  type="number"
                  min="0"
                  step="0.1"
                  bind:value={rule.lineWidth}
                  disabled={pageRangeMode !== 'rule'}
                />
              </td>
              <td>
                <button
                  type="button"
                  class="remove"
                  onclick={() => removeRule(rule.id)}
                  disabled={pageRangeMode !== 'rule'}
                  aria-label="Remove rule">×</button
                >
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      <button
        type="button"
        class="add"
        onclick={addRule}
        disabled={pageRangeMode !== 'rule'}>+ Add rule</button
      >
    </div>
  </fieldset>
  </div>

  <section class="original">
    <span class="original-legend">Original file:</span>
    <label class="radio">
      <input type="radio" bind:group={originalMode} value="delete" />
      Delete original
    </label>
    <label class="radio">
      <input type="radio" bind:group={originalMode} value="rename" />
      Rename to
      <span class="hint"
        ><code>name</code><input
          type="text"
          class="suffix"
          bind:value={renameSuffix}
          placeholder="_old"
          disabled={originalMode !== 'rename'}
        /><code>.pdf</code></span
      >
    </label>
  </section>

  <div class="convert-row">
    <button
      type="button"
      class="primary"
      onclick={convert}
      disabled={!files.length || converting}
    >
      {converting ? 'Converting…' : 'Convert'}
    </button>

    {#if progress}
      <div class="progress" aria-live="polite">
        <div class="bar">
          <div
            class="fill"
            style="width: {(progress.current / progress.total) * 100}%"
          ></div>
        </div>
        <div class="label">
          {progress.current}/{progress.total}
          {#if progress.file}— {progress.file}{/if}
        </div>
      </div>
    {/if}

    {#if convertMessage}
      <span class="status">{convertMessage}</span>
    {/if}
  </div>

  <section class="file-list">
    <header>
      <h2>Files</h2>
      <span class="count">{files.length}</span>
    </header>

    {#if files.length === 0}
      <p class="empty">No files yet. Drop some above or click Browse…</p>
    {:else}
      <ul class="files">
        {#each files as path (path)}
          <li class={statuses[path] ?? ''}>
            <span class="status-icon" title={errors[path] ?? statuses[path] ?? ''}>
              {#if statuses[path] === 'converting'}⏳
              {:else if statuses[path] === 'done'}✓
              {:else if statuses[path] === 'error'}✗
              {:else if statuses[path] === 'pending'}·
              {/if}
            </span>
            <button
              type="button"
              class="name"
              title="Open {path}"
              onclick={() => onNameClick(path)}>{basename(path)}</button
            >
            <span class="mtime">{stats[path] ? formatDate(stats[path].modified_ms) : ''}</span>
            <span class="size">{stats[path] ? formatSize(stats[path].size) : '—'}</span>
            <button
              type="button"
              class="remove"
              onclick={() => remove(path)}
              aria-label="Remove {basename(path)}">×</button
            >
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</main>

<style>
  :root {
    /* === Background gradient — tweak these to taste === */
    --bg-gradient-from: #ffffff;     /* top-left  (light mode) */
    --bg-gradient-to:   #bebebe;     /* bottom-right (light mode) */
    --bg-gradient-angle: 180deg;     /* full diagonal: top-left → bottom-right */
    /* ================================================= */

    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 14px;
    line-height: 20px;
    font-weight: 400;
    color: #0f0f0f;
    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  :global(html),
  :global(body) {
    margin: 0;
    min-height: 100vh;
    background: linear-gradient(
      var(--bg-gradient-angle),
      var(--bg-gradient-from),
      var(--bg-gradient-to)
    ) fixed;
  }

  .container {
    max-width: 960px;
    margin: 0 auto;
    padding: 1.25rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  h1 {
    margin: 0;
    font-size: 1.4rem;
  }

  .convert-row {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 0.4rem;
  }

  button.primary {
    width: 100%;
    border-radius: 6px;
    border: 1px solid transparent;
    padding: 0.5em 1.1em;
    font-size: 0.95em;
    font-weight: 600;
    font-family: inherit;
    color: #ffffff;
    background-color: #396cd8;
    cursor: pointer;
    transition:
      background-color 0.15s,
      opacity 0.15s;
  }
  button.primary:hover:not(:disabled) {
    background-color: #2f5cc4;
  }
  button.primary:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .status {
    font-size: 0.9rem;
    opacity: 0.75;
  }

  .progress {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .progress .bar {
    height: 6px;
    background-color: rgba(0, 0, 0, 0.12);
    border-radius: 3px;
    overflow: hidden;
  }
  .progress .fill {
    height: 100%;
    background-color: #396cd8;
    transition: width 0.15s ease;
  }
  .progress .label {
    font-size: 0.85rem;
    opacity: 0.75;
    font-variant-numeric: tabular-nums;
  }

  .files .status-icon {
    width: 1.2rem;
    text-align: center;
    font-size: 0.95rem;
    opacity: 0.85;
  }
  .files li.done .status-icon {
    color: #2f9e44;
  }
  .files li.error .status-icon {
    color: #c84a4a;
  }
  .files li.converting {
    background-color: rgba(57, 108, 216, 0.12) !important;
  }

  .settings-row {
    display: flex;
    align-items: stretch;
    gap: 0.6rem;
  }
  .settings-row > .group {
    flex: 1 1 0;
    min-width: 0;
  }

  .group {
    border: 1px solid rgba(0, 0, 0, 0.2);
    border-radius: 5px;
    padding: 0.5rem 0.75rem 0.65rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .group legend {
    padding: 0 0.3rem;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .group .grid {
    display: grid;
    grid-template-columns: max-content max-content;
    column-gap: 0.5rem;
    row-gap: 0.3rem;
    align-items: center;
  }
  .group .grid label {
    justify-self: end;
  }

  .group input[type='number'],
  .group input[type='text'],
  .group select {
    padding: 0.2em 0.4em;
    border-radius: 4px;
    border: 1px solid rgba(0, 0, 0, 0.25);
    font: inherit;
    background-color: rgba(255, 255, 255, 0.85);
    color: inherit;
    font-variant-numeric: tabular-nums;
  }
  .group select {
    background-color: #d8e4ff;
  }
  .group input[type='number'] {
    width: 6rem;
  }
  .group input[type='number']:disabled,
  .group input[type='text']:disabled,
  .group select:disabled {
    opacity: 0.55;
    background-color: rgba(0, 0, 0, 0.05);
    color: rgba(0, 0, 0, 0.55);
    cursor: not-allowed;
  }

  .group .checks {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.3rem;
    margin-top: 0.6rem;
    padding-top: 0.6rem;
    border-top: 1px solid rgba(0, 0, 0, 0.12);
  }
  .group .radios {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .group .checks.two-col {
    flex-direction: row;
    align-items: flex-start;
    gap: 1.5rem;
  }
  .group .checks.two-col .col {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .group .checks input[type='checkbox'] {
    width: 1rem;
    height: 1rem;
    cursor: pointer;
  }
  .group .checks label,
  .group .radio {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    cursor: pointer;
  }

  .group .radio .inline {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    margin-left: 0.4rem;
  }
  .group .radio .inline input[type='number'] {
    width: 4rem;
  }

  .rules-block {
    margin-top: 0.15rem;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .rules-block.disabled {
    opacity: 0.55;
  }

  label.disabled {
    opacity: 0.55;
  }

  .rules-table {
    border-collapse: collapse;
    width: max-content;
    border: 1px solid rgba(0, 0, 0, 0.25);
  }
  .rules-table th,
  .rules-table td {
    padding: 0.15rem 0.4rem;
    border: 1px solid rgba(0, 0, 0, 0.15);
  }
  .rules-table th {
    text-align: left;
    font-size: 0.85rem;
    font-weight: 500;
    background-color: rgba(0, 0, 0, 0.05);
  }
  .rules-table input[type='number'],
  .rules-table input[type='text'] {
    width: 4.75rem;
    border: none;
    background: transparent;
    padding: 0.1em 0.25em;
  }
  .rules-table input[type='text'] {
    width: 9rem;
  }
  .rules-table tr:has(input:focus) {
    background-color: #2962d8;
    color: white;
  }

  .add {
    border: 1px solid currentColor;
    background: transparent;
    color: inherit;
    border-radius: 5px;
    padding: 0.2em 0.65em;
    font-size: 0.8rem;
    cursor: pointer;
    opacity: 0.8;
    align-self: flex-start;
  }
  .add:hover:not(:disabled) {
    opacity: 1;
  }
  .add:disabled {
    cursor: not-allowed;
    opacity: 0.4;
  }

  .option {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.95rem;
  }

  .original {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 1rem;
    font-size: 0.9rem;
  }
  .original-legend {
    font-weight: 500;
  }
  .original .hint {
    display: inline-flex;
    align-items: center;
    gap: 0.15rem;
    margin-left: 0.25rem;
    opacity: 0.85;
  }
  .original code {
    font-family: ui-monospace, Menlo, Consolas, monospace;
    font-size: 0.9em;
    opacity: 0.7;
  }
  input.suffix {
    width: 4.5rem;
    padding: 0.2em 0.4em;
    border-radius: 5px;
    border: 1px solid rgba(0, 0, 0, 0.15);
    font: inherit;
    background-color: rgba(255, 255, 255, 0.5);
    color: inherit;
    text-align: center;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .file-list header {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
  }

  .file-list h2 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .file-list .count {
    font-size: 0.85rem;
    opacity: 0.6;
  }

  .empty {
    margin: 0;
    padding: 0.5rem 0.25rem;
    opacity: 0.6;
    font-size: 0.85rem;
  }

  .files {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .files li {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.35rem 0.55rem;
    border-radius: 6px;
  }

  .files li:nth-child(odd) {
    background-color: rgba(0, 0, 0, 0.06);
  }
  .files li:nth-child(even) {
    background-color: rgba(0, 0, 0, 0.02);
  }

  .files .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: left;
    background: transparent;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  .files .name:hover {
    text-decoration: underline;
    color: #396cd8;
  }

  .files .size {
    min-width: 5.5rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
    font-size: 0.85rem;
    opacity: 0.75;
  }

  .files .mtime {
    min-width: 11rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
    font-size: 0.85rem;
    opacity: 0.75;
    white-space: nowrap;
  }

  .remove {
    border: none;
    background: transparent;
    color: inherit;
    font-size: 1.1rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 0.3rem;
    border-radius: 5px;
  }
  .remove:hover {
    background-color: rgba(200, 74, 74, 0.15);
    color: #c84a4a;
  }

  /* Dark mode: applies when <html data-theme="dark"> is set. The frontend
     resolves the initial value from saved preference or system setting on
     mount, so we don't need a prefers-color-scheme media query here. */
  :global(html[data-theme='dark']) {
    --bg-gradient-from: #2f2f2f;
    --bg-gradient-to:   #1a2240;
    color: #f6f6f6;
  }
  :global(html[data-theme='dark']) .files li:nth-child(odd) {
    background-color: rgba(255, 255, 255, 0.08);
  }
  :global(html[data-theme='dark']) .files li:nth-child(even) {
    background-color: rgba(255, 255, 255, 0.03);
  }
  :global(html[data-theme='dark']) input.suffix {
    background-color: rgba(255, 255, 255, 0.06);
    border-color: rgba(255, 255, 255, 0.15);
  }
  :global(html[data-theme='dark']) .group {
    border-color: rgba(255, 255, 255, 0.2);
  }
  :global(html[data-theme='dark']) .group input[type='number'],
  :global(html[data-theme='dark']) .group input[type='text'],
  :global(html[data-theme='dark']) .group select {
    background-color: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.2);
  }
  :global(html[data-theme='dark']) .group select {
    background-color: #2c4a92;
  }
  :global(html[data-theme='dark']) .rules-table {
    border-color: rgba(255, 255, 255, 0.2);
  }
  :global(html[data-theme='dark']) .rules-table th,
  :global(html[data-theme='dark']) .rules-table td {
    border-color: rgba(255, 255, 255, 0.12);
  }
  :global(html[data-theme='dark']) .rules-table th {
    background-color: rgba(255, 255, 255, 0.06);
  }
</style>
