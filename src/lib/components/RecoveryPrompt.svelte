<script>
  import { formatElapsed } from "../utils/formatTime.js";

  let { data, onResume, onDiscard } = $props();

  let elapsedSince = $derived(data ? Date.now() - data.startTime : 0);
  let lastTask = $derived(
    data?.tasks?.length > 0
      ? data.tasks[data.tasks.length - 1].name
      : "None"
  );
</script>

<div class="modal-overlay">
  <div class="modal">
    <h2>Session Recovery</h2>
    <p style="font-size: 13px; color: var(--text-dim); margin-bottom: 16px">
      A previous session was interrupted. Would you like to resume?
    </p>

    <div class="recovery-info">
      <div class="recovery-stat">
        <span class="recovery-stat-label">Time elapsed</span>
        <span class="recovery-stat-value">{formatElapsed(elapsedSince)}</span>
      </div>
      <div class="recovery-stat">
        <span class="recovery-stat-label">Notes</span>
        <span class="recovery-stat-value">{data?.notes?.length || 0}</span>
      </div>
      <div class="recovery-stat">
        <span class="recovery-stat-label">Last task</span>
        <span class="recovery-stat-value">{lastTask}</span>
      </div>
    </div>

    <div class="controls" style="padding: 0">
      <button class="btn btn-primary" onclick={onResume}>Resume</button>
      <button class="btn btn-stop" onclick={onDiscard}>Discard</button>
    </div>
  </div>
</div>
