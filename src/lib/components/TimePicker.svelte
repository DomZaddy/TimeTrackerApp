<script>
  let { onConfirm, onCancel, mode = 'start' } = $props();

  const labels = {
    start: { title: 'When did you start?', now: 'Start Now' },
    stop: { title: 'When did you stop?', now: 'Stop Now' },
    'break-start': { title: 'When did your break start?', now: 'Break Now' },
    'break-end': { title: 'When did your break end?', now: 'Resume Now' },
  };

  let label = $derived(labels[mode] || labels.start);

  const now = new Date();
  let hours = $state(now.getHours() % 12 || 12);
  let minutes = $state(now.getMinutes());
  let ampm = $state(now.getHours() >= 12 ? "PM" : "AM");

  function useNow() {
    onConfirm(null);
  }

  function confirmCustom() {
    let h = hours;
    if (ampm === "PM" && h !== 12) h += 12;
    if (ampm === "AM" && h === 12) h = 0;

    const d = new Date();
    d.setHours(h, minutes, 0, 0);

    // Don't allow future times
    if (d.getTime() > Date.now()) {
      return;
    }

    onConfirm(d.getTime());
  }

  function incHour() { hours = hours >= 12 ? 1 : hours + 1; }
  function decHour() { hours = hours <= 1 ? 12 : hours - 1; }
  function incMin() { minutes = minutes >= 59 ? 0 : minutes + 1; }
  function decMin() { minutes = minutes <= 0 ? 59 : minutes - 1; }
  function toggleAmPm() { ampm = ampm === "AM" ? "PM" : "AM"; }
</script>

<div class="modal-overlay">
  <div class="modal">
    <h2>{label.title}</h2>

    <div class="time-picker-grid">
      <div class="time-picker-col">
        <button class="time-picker-btn" onclick={incHour}>▲</button>
        <span class="time-picker-value">{String(hours).padStart(2, "0")}</span>
        <button class="time-picker-btn" onclick={decHour}>▼</button>
      </div>

      <span class="time-picker-separator">:</span>

      <div class="time-picker-col">
        <button class="time-picker-btn" onclick={incMin}>▲</button>
        <span class="time-picker-value">{String(minutes).padStart(2, "0")}</span>
        <button class="time-picker-btn" onclick={decMin}>▼</button>
      </div>

      <div class="time-picker-col">
        <button class="time-picker-btn" onclick={toggleAmPm}>▲</button>
        <span class="time-picker-value" style="font-size: 20px">{ampm}</span>
        <button class="time-picker-btn" onclick={toggleAmPm}>▼</button>
      </div>
    </div>

    <div class="controls" style="padding: 0">
      <button class="btn btn-primary" onclick={useNow}>
        {label.now}
      </button>
      <button class="btn btn-primary" onclick={confirmCustom}>Use This Time</button>
    </div>

    <button
      class="btn btn-break"
      style="width: 100%; margin-top: 8px"
      onclick={onCancel}
    >
      Cancel
    </button>
  </div>
</div>
