<script>
  import { minimizeWindow, closeWindow } from "../tauri.js";

  let {
    widgetMode = false,
    onToggleWidget,
    updateStatus = "",
    updateVersion = "",
    updateProgress = 0,
    onInstallUpdate,
  } = $props();
</script>

<div class="title-bar">
  <div class="title-bar-left">
    <span>TimeTracker</span>

    {#if updateStatus === "available"}
      <button class="update-badge" onclick={onInstallUpdate} title="Update to v{updateVersion}">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M6 2v6M3.5 5.5L6 8l2.5-2.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M2.5 10h7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        <span>v{updateVersion}</span>
      </button>
    {:else if updateStatus === "downloading"}
      <div class="update-badge downloading" title="Downloading... {updateProgress}%">
        <div class="update-progress-ring">
          <svg width="14" height="14" viewBox="0 0 14 14">
            <circle cx="7" cy="7" r="5.5" fill="none" stroke="var(--text-dim)" stroke-width="1.5" opacity="0.3" />
            <circle cx="7" cy="7" r="5.5" fill="none" stroke="var(--green)" stroke-width="1.5"
              stroke-dasharray={2 * Math.PI * 5.5}
              stroke-dashoffset={2 * Math.PI * 5.5 * (1 - updateProgress / 100)}
              stroke-linecap="round"
              transform="rotate(-90 7 7)" />
          </svg>
        </div>
        <span>{updateProgress}%</span>
      </div>
    {:else if updateStatus === "ready"}
      <div class="update-badge ready">
        <span>Restarting...</span>
      </div>
    {/if}
  </div>
  <div class="title-bar-buttons">
    <button class="title-bar-btn" onclick={onToggleWidget} title={widgetMode ? "Full mode" : "Widget mode"}>
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
        {#if widgetMode}
          <rect x="1" y="1" width="12" height="12" rx="2" stroke="currentColor" stroke-width="1.5" />
        {:else}
          <rect x="1" y="1" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.5" />
          <path d="M10 4v6.5a1.5 1.5 0 01-1.5 1.5H4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        {/if}
      </svg>
    </button>
    <button class="title-bar-btn" onclick={minimizeWindow} title="Minimize">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
        <path d="M3 7h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
    </button>
    <button class="title-bar-btn close" onclick={closeWindow} title="Close">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
        <path d="M3.5 3.5l7 7M10.5 3.5l-7 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
    </button>
  </div>
</div>

<style>
  .update-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: 8px;
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 10px;
    font-weight: 600;
    cursor: pointer;
    border: none;
    background: var(--green);
    color: #000;
    transition: transform 0.15s, box-shadow 0.15s;
    animation: pulse-glow 2s ease-in-out infinite;
  }

  .update-badge:hover {
    transform: scale(1.05);
    box-shadow: 0 0 8px var(--green);
  }

  .update-badge.downloading {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-dim);
    cursor: default;
    animation: none;
  }

  .update-badge.ready {
    background: var(--accent);
    color: #fff;
    cursor: default;
    animation: none;
  }

  .update-progress-ring {
    display: flex;
    align-items: center;
  }

  @keyframes pulse-glow {
    0%, 100% { box-shadow: 0 0 4px rgba(0, 214, 143, 0.3); }
    50% { box-shadow: 0 0 12px rgba(0, 214, 143, 0.6); }
  }
</style>
