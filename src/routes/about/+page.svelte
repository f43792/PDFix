<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion, getName } from '@tauri-apps/api/app';
  import { invoke } from '@tauri-apps/api/core';
  import { makeSequenceMatcher } from '$lib/utils';

  let version = $state('');
  let name = $state('pdfix');
  let commit = $state('');
  let signatureRevealed = $state(false);

  const matcher = makeSequenceMatcher(['f', 'c', 'n']);

  function onKey(e: KeyboardEvent) {
    if (signatureRevealed) return;
    if (matcher.feed(e.key)) signatureRevealed = true;
  }

  onMount(() => {
    getVersion()
      .then((v) => (version = v))
      .catch((e) => console.error('getVersion failed:', e));
    getName()
      .then((n) => (name = n))
      .catch((e) => console.error('getName failed:', e));
    invoke<string>('get_git_commit')
      .then((c) => (commit = c))
      .catch((e) => console.error('get_git_commit failed:', e));
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });
</script>

<main class="about">
  <img src="/app_icon_128x128.png" alt="application icon" class="icon" />
  <h1>{name}</h1>
  <p class="version">
    Version {version}{#if commit}
      <span class="commit">[{commit}]</span>
    {/if}
  </p>
  <p class="tagline">A simple line-width batch editor for PDFs.</p>
  <p class="powered">
    PDF rendering powered by
    <a href="https://github.com/messense/mupdf-rs" target="_blank" rel="noreferrer">mupdf-rs</a>
    (Rust bindings to <a href="https://mupdf.com/" target="_blank" rel="noreferrer">MuPDF</a>).
  </p>
  <hr />
  <p class="license">
    Licensed under the
    <a
      href="https://www.gnu.org/licenses/agpl-3.0.html"
      target="_blank"
      rel="noreferrer">GNU Affero General Public License v3.0</a
    >
    <br />
    MuPDF is also distributed under AGPL-3.0.
  </p>

  {#if signatureRevealed}
    <p class="signature">by Fabio Nascimento</p>
  {/if}
</main>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    background-color: #f6f6f6;
    color: #0f0f0f;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 13px;
  }

  .about {
    position: relative;
    min-height: 100vh;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 1.5rem 1.25rem;
    gap: 0.4rem;
  }

  .signature {
    position: absolute;
    bottom: 1.0rem;
    left: 0;
    right: 0;
    margin: 0;
    font-size: 0.7rem;
    opacity: 0.55;
    font-style: italic;
    letter-spacing: 0.02em;
  }

  .commit {
    font-family: ui-monospace, Consolas, Menlo, monospace;
    font-size: 0.95em;
    opacity: 0.7;
    margin-left: 0.25rem;
  }

  .icon {
    width: 96px;
    height: 96px;
    image-rendering: -webkit-optimize-contrast;
  }

  h1 {
    margin: 0.25rem 0 0;
    font-size: 1.4rem;
  }

  .version {
    margin: 0;
    opacity: 0.7;
    font-variant-numeric: tabular-nums;
  }

  .tagline {
    margin: 0.4rem 0 0;
  }

  .powered {
    margin: 0.5rem 0 0;
    max-width: 32rem;
    line-height: 1.45;
  }

  hr {
    width: 80%;
    border: none;
    border-top: 1px solid rgba(0, 0, 0, 0.12);
    margin: 0.6rem 0 0.4rem;
  }

  .license {
    margin: 0;
    max-width: 32rem;
    line-height: 1.45;
    opacity: 0.85;
  }

  a {
    color: #396cd8;
  }

  @media (prefers-color-scheme: dark) {
    :global(html),
    :global(body) {
      background-color: #2f2f2f;
      color: #f6f6f6;
    }
    hr {
      border-top-color: rgba(255, 255, 255, 0.15);
    }
    a {
      color: #8ab0ff;
    }
  }
</style>
