<script>
  import { formatElapsed, formatHoursDecimal } from "../utils/formatTime.js";

  let { elapsed = 0, breakElapsed = 0, isRunning = false, isOnBreak = false, compact = false } = $props();

  let statusText = $derived(
    !isRunning ? "Ready" : isOnBreak ? "On Break" : "Working"
  );

  let statusClass = $derived(
    !isRunning ? "ready" : isOnBreak ? "on-break" : "working"
  );
</script>

<div class="timer-display">
  <div class="timer-time" class:on-break={isOnBreak}>
    {formatElapsed(elapsed)}
  </div>

  <div class="timer-status">
    <span class="status-dot {statusClass}"></span>
    <span>{statusText}</span>
  </div>

  {#if !compact && isRunning}
    <div class="timer-meta">
      <span>{formatHoursDecimal(elapsed)}h worked</span>
      {#if breakElapsed > 0}
        <span>{formatHoursDecimal(breakElapsed)}h break</span>
      {/if}
    </div>
  {/if}
</div>
