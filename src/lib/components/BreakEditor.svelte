<script>
  let { onClose, onAdd } = $props();

  const now = new Date();
  // Default: 1 hour break ending now
  let startHours = $state(((now.getHours() - 1 + 12) % 12) || 12);
  let startMinutes = $state(now.getMinutes());
  let startAmpm = $state((now.getHours() - 1) >= 12 ? "PM" : "AM");

  let endHours = $state(now.getHours() % 12 || 12);
  let endMinutes = $state(now.getMinutes());
  let endAmpm = $state(now.getHours() >= 12 ? "PM" : "AM");

  let error = $state("");

  function toMs(h, m, ap) {
    let hour = h;
    if (ap === "PM" && hour !== 12) hour += 12;
    if (ap === "AM" && hour === 12) hour = 0;
    const d = new Date();
    d.setHours(hour, m, 0, 0);
    return d.getTime();
  }

  function confirm() {
    const startMs = toMs(startHours, startMinutes, startAmpm);
    const endMs = toMs(endHours, endMinutes, endAmpm);

    if (endMs <= startMs) {
      error = "End time must be after start time";
      return;
    }
    if (endMs > Date.now()) {
      error = "Break can't end in the future";
      return;
    }

    error = "";
    onAdd(startMs, endMs);
  }

  function inc(val, max) { return val >= max ? (max === 12 ? 1 : 0) : val + 1; }
  function dec(val, min, max) { return val <= min ? max : val - 1; }
</script>

<div class="modal-overlay">
  <div class="modal">
    <h2>Add Break</h2>

    <div class="break-editor-section">
      <label class="break-editor-label">Break Start</label>
      <div class="time-picker-grid">
        <div class="time-picker-col">
          <button class="time-picker-btn" onclick={() => startHours = inc(startHours, 12)}>▲</button>
          <span class="time-picker-value">{String(startHours).padStart(2, "0")}</span>
          <button class="time-picker-btn" onclick={() => startHours = dec(startHours, 1, 12)}>▼</button>
        </div>
        <span class="time-picker-separator">:</span>
        <div class="time-picker-col">
          <button class="time-picker-btn" onclick={() => startMinutes = inc(startMinutes, 59)}>▲</button>
          <span class="time-picker-value">{String(startMinutes).padStart(2, "0")}</span>
          <button class="time-picker-btn" onclick={() => startMinutes = dec(startMinutes, 0, 59)}>▼</button>
        </div>
        <div class="time-picker-col">
          <button class="time-picker-btn" onclick={() => startAmpm = startAmpm === "AM" ? "PM" : "AM"}>▲</button>
          <span class="time-picker-value" style="font-size: 20px">{startAmpm}</span>
          <button class="time-picker-btn" onclick={() => startAmpm = startAmpm === "AM" ? "PM" : "AM"}>▼</button>
        </div>
      </div>
    </div>

    <div class="break-editor-section">
      <label class="break-editor-label">Break End</label>
      <div class="time-picker-grid">
        <div class="time-picker-col">
          <button class="time-picker-btn" onclick={() => endHours = inc(endHours, 12)}>▲</button>
          <span class="time-picker-value">{String(endHours).padStart(2, "0")}</span>
          <button class="time-picker-btn" onclick={() => endHours = dec(endHours, 1, 12)}>▼</button>
        </div>
        <span class="time-picker-separator">:</span>
        <div class="time-picker-col">
          <button class="time-picker-btn" onclick={() => endMinutes = inc(endMinutes, 59)}>▲</button>
          <span class="time-picker-value">{String(endMinutes).padStart(2, "0")}</span>
          <button class="time-picker-btn" onclick={() => endMinutes = dec(endMinutes, 0, 59)}>▼</button>
        </div>
        <div class="time-picker-col">
          <button class="time-picker-btn" onclick={() => endAmpm = endAmpm === "AM" ? "PM" : "AM"}>▲</button>
          <span class="time-picker-value" style="font-size: 20px">{endAmpm}</span>
          <button class="time-picker-btn" onclick={() => endAmpm = endAmpm === "AM" ? "PM" : "AM"}>▼</button>
        </div>
      </div>
    </div>

    {#if error}
      <p class="break-editor-error">{error}</p>
    {/if}

    <div class="controls" style="padding: 0">
      <button class="btn btn-primary" onclick={confirm}>Add Break</button>
    </div>

    <button
      class="btn btn-break"
      style="width: 100%; margin-top: 8px"
      onclick={onClose}
    >
      Cancel
    </button>
  </div>
</div>

<style>
  .break-editor-section {
    margin-bottom: 12px;
  }
  .break-editor-label {
    display: block;
    font-size: 13px;
    color: var(--text-dim);
    margin-bottom: 6px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .break-editor-error {
    color: var(--red, #e55555);
    font-size: 13px;
    margin: 8px 0;
  }
</style>
