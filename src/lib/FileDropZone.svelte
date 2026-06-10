<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { open } from '@tauri-apps/plugin-dialog';

  type Props = {
    onFiles: (paths: string[]) => void;
    label?: string;
  };

  let { onFiles, label = 'Drop files here' }: Props = $props();

  let zoneEl: HTMLDivElement | undefined = $state();
  let phase: 'idle' | 'hovering' | 'rejected' = $state('idle');

  // Tauri reports drop position in window-relative physical pixels.
  // getBoundingClientRect is in CSS pixels, so multiply by devicePixelRatio.
  function isInside(x: number, y: number): boolean {
    if (!zoneEl) return false;
    const r = zoneEl.getBoundingClientRect();
    const dpr = window.devicePixelRatio;
    return (
      x >= r.left * dpr &&
      x <= r.right * dpr &&
      y >= r.top * dpr &&
      y <= r.bottom * dpr
    );
  }

  onMount(() => {
    const wv = getCurrentWebviewWindow();
    const unlistenPromise = wv.onDragDropEvent(({ payload }) => {
      if (payload.type === 'enter' || payload.type === 'over') {
        phase = isInside(payload.position.x, payload.position.y)
          ? 'hovering'
          : 'rejected';
      } else if (payload.type === 'leave') {
        phase = 'idle';
      } else if (payload.type === 'drop') {
        const inside = isInside(payload.position.x, payload.position.y);
        phase = 'idle';
        if (inside && payload.paths.length) onFiles(payload.paths);
      }
    });
    return () => {
      unlistenPromise.then((u) => u());
    };
  });

  async function browse() {
    const result = await open({ multiple: true });
    if (!result) return;
    const paths = Array.isArray(result) ? result : [result];
    if (paths.length) onFiles(paths);
  }
</script>

<div
  bind:this={zoneEl}
  class="zone"
  class:hovering={phase === 'hovering'}
  class:rejected={phase === 'rejected'}
>
  <p class="label">{label}</p>
  <button type="button" onclick={browse}>Browse…</button>
</div>

<style>
  .zone {
    border: 2px dashed #888;
    border-radius: 12px;
    padding: 2rem;
    text-align: center;
    transition:
      border-color 0.15s ease,
      background-color 0.15s ease;
  }
  .zone.hovering {
    border-color: #24c8db;
    background-color: rgba(36, 200, 219, 0.08);
  }
  .zone.rejected {
    border-color: #c84a4a;
    background-color: rgba(200, 74, 74, 0.06);
  }
  .label {
    margin: 0 0 1rem;
    color: inherit;
    opacity: 0.85;
  }
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.5em 1.1em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    cursor: pointer;
    transition: border-color 0.2s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }
  button:hover {
    border-color: #396cd8;
  }
  @media (prefers-color-scheme: dark) {
    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
  }
</style>
