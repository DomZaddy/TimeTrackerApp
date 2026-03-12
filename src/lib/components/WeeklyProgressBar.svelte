<script>
  import { formatHoursDecimal } from "../utils/formatTime.js";

  let {
    totalWeekMs = 0,
    weekPercent = 0,
    isOvertime = false,
    overtimeMs = 0,
    overtimeMode = false,
    weekLimitHours = 40,
    isRunning = false,
    elapsed = 0,
  } = $props();

  // Include current session elapsed time in the totals
  let liveWeekMs = $derived(isRunning ? totalWeekMs + elapsed : totalWeekMs);
  let livePercent = $derived(liveWeekMs / (weekLimitHours * 3600000));
  let liveOvertime = $derived(livePercent > 1.0);
  let liveOvertimeMs = $derived(Math.max(0, liveWeekMs - weekLimitHours * 3600000));

  let fillPercent = $derived(Math.min(livePercent * 100, 100));
  let totalHours = $derived(liveWeekMs / 3600000);

  let fillColor = $derived(
    livePercent >= 1
      ? 'linear-gradient(90deg, #ff8080, var(--red), #e55555)'
      : '#6c5ce7'
  );
</script>

<div class="weekly-bar">
  <div
    class="weekly-bar-fill"
    style="width: {fillPercent}%; background: {fillColor}"
  ></div>

  <div class="weekly-bar-content">
    <div class="weekly-bar-left">
      <span class="weekly-bar-hours">{totalHours.toFixed(1)}h</span>
      <span class="weekly-bar-separator">/</span>
      <span class="weekly-bar-limit">{weekLimitHours}h</span>
      {#if liveOvertime && overtimeMode}
        <span class="weekly-bar-overtime">+{(liveOvertimeMs / 3600000).toFixed(1)}h OT</span>
      {/if}
      {#if liveOvertime && !overtimeMode}
        <span class="weekly-bar-overtime">OVERTIME</span>
      {/if}
    </div>

    <div class="weekly-bar-center">
      <span class="weekly-bar-percent">{Math.round(livePercent * 100)}%</span>
    </div>

    <div class="weekly-bar-right">
      <span class="weekly-bar-week-label">This Week</span>
    </div>
  </div>

  {#if liveOvertime}
    <div class="weekly-bar-overtime-stripe"></div>
  {/if}
</div>
