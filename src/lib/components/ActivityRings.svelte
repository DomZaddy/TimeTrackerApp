<script>
  import { formatElapsed, formatHoursDecimal } from '../utils/formatTime.js';
  import { gamificationData } from '../stores/gamification.js';
  import { onDestroy } from 'svelte';

  let {
    elapsed = 0,
    breakElapsed = 0,
    isRunning = false,
    isOnBreak = false,
    dailyGoalHours = 8,
    weekPercent = 0,
    compact = false,
  } = $props();

  // Streak
  let streak = $state(0);
  const unsub = gamificationData.subscribe(d => { streak = d.streaks?.daily || 0; });
  onDestroy(() => unsub());

  // Daily ring — visual scale goes to 10h so 8h goal doesn't look "full"
  // Overtime orange starts beyond 10h
  let dailyHours = $derived(elapsed / 3600000);
  let dailyVisualMax = $derived(dailyGoalHours * 1.25); // 8h goal → 10h ring max
  let dailyComplete = $derived(dailyHours >= dailyGoalHours);
  let dailyOvertime = $derived(dailyHours > dailyVisualMax);
  let dailyPercent = $derived(Math.min((dailyHours / dailyVisualMax) * 100, 100));
  let dailyOTPercent = $derived(
    dailyOvertime ? Math.min(((dailyHours - dailyVisualMax) / dailyVisualMax) * 100, 100) : 0
  );

  // Weekly ring — include current session elapsed in the weekly total
  // Visual scale goes to 125% of limit so 100% doesn't look full
  let weekLimitMs = $derived(dailyGoalHours * 5 * 3600000);
  let weekVisualMax = $derived(1.25); // 125% of limit before overtime wraps
  let liveWeekPercent = $derived(
    isRunning ? weekPercent + (elapsed / weekLimitMs) : weekPercent
  );
  let weekComplete = $derived(liveWeekPercent >= 1);
  let weekOvertime = $derived(liveWeekPercent > weekVisualMax);
  let weekFill = $derived(Math.min((liveWeekPercent / weekVisualMax) * 100, 100));
  let weekOTPercent = $derived(
    weekOvertime ? Math.min(((liveWeekPercent - weekVisualMax) / weekVisualMax) * 100, 100) : 0
  );

  // Ring math
  const outerR = 92;
  const innerR = 78;
  const strokeW = 7;
  const outerCirc = 2 * Math.PI * outerR;
  const innerCirc = 2 * Math.PI * innerR;

  let outerOffset = $derived(outerCirc - (weekFill / 100) * outerCirc);
  let outerOTOffset = $derived(outerCirc - (weekOTPercent / 100) * outerCirc);
  let innerOffset = $derived(innerCirc - (dailyPercent / 100) * innerCirc);
  let innerOTOffset = $derived(innerCirc - (dailyOTPercent / 100) * innerCirc);

  // Timer
  let statusText = $derived(!isRunning ? 'Ready' : isOnBreak ? 'On Break' : 'Working');
  let statusClass = $derived(!isRunning ? 'ready' : isOnBreak ? 'on-break' : 'working');

  // Streak size
  let flameSize = $derived(
    streak >= 30 ? 'flame-legendary'
    : streak >= 14 ? 'flame-large'
    : streak >= 7 ? 'flame-medium'
    : streak >= 3 ? 'flame-small'
    : ''
  );
</script>

<div class="activity-rings" class:compact>
  <div class="rings-container">
    <svg class="rings-svg" viewBox="0 0 200 200">
      <!-- Outer ring bg (weekly) -->
      <circle cx="100" cy="100" r={outerR}
        fill="none" stroke="rgba(108, 92, 231, 0.1)" stroke-width={strokeW} />
      <!-- Outer ring fill (weekly) -->
      <circle cx="100" cy="100" r={outerR}
        fill="none"
        stroke={weekComplete ? 'var(--green)' : 'var(--accent)'}
        stroke-width={strokeW}
        stroke-linecap="round"
        stroke-dasharray={outerCirc}
        stroke-dashoffset={outerOffset}
        transform="rotate(-90 100 100)"
        class="ring-fill"
        class:ring-complete={weekComplete}
      />
      <!-- Outer ring overtime lap (orange) -->
      {#if weekOvertime}
        <circle cx="100" cy="100" r={outerR}
          fill="none"
          stroke="#ff9f43"
          stroke-width={strokeW}
          stroke-linecap="round"
          stroke-dasharray={outerCirc}
          stroke-dashoffset={outerOTOffset}
          transform="rotate(-90 100 100)"
          class="ring-fill ring-overtime"
        />
      {/if}

      <!-- Inner ring bg (daily) -->
      <circle cx="100" cy="100" r={innerR}
        fill="none" stroke="rgba(0, 214, 143, 0.1)" stroke-width={strokeW} />
      <!-- Inner ring fill (daily) -->
      <circle cx="100" cy="100" r={innerR}
        fill="none"
        stroke={dailyComplete ? '#00e69a' : 'var(--green)'}
        stroke-width={strokeW}
        stroke-linecap="round"
        stroke-dasharray={innerCirc}
        stroke-dashoffset={innerOffset}
        transform="rotate(-90 100 100)"
        class="ring-fill"
        class:ring-complete={dailyComplete}
      />
      <!-- Inner ring overtime lap (orange) -->
      {#if dailyOvertime}
        <circle cx="100" cy="100" r={innerR}
          fill="none"
          stroke="#ff9f43"
          stroke-width={strokeW}
          stroke-linecap="round"
          stroke-dasharray={innerCirc}
          stroke-dashoffset={innerOTOffset}
          transform="rotate(-90 100 100)"
          class="ring-fill ring-overtime"
        />
      {/if}
    </svg>

    <!-- Timer text centered in rings -->
    <div class="rings-center">
      <div class="rings-timer" class:on-break={isOnBreak}>
        {formatElapsed(elapsed)}
      </div>
      <div class="rings-status">
        <span class="status-dot {statusClass}"></span>
        <span>{statusText}</span>
      </div>
    </div>

    {#if isRunning}
      <div class="rings-pulse"></div>
    {/if}
  </div>

  <!-- Ring legend + streak -->
  {#if !compact}
    <div class="rings-footer">
      <div class="rings-legend">
        <div class="legend-item">
          <span class="legend-dot" style="background: var(--accent)"></span>
          <span>Weekly</span>
        </div>
        <div class="legend-item">
          <span class="legend-dot" style="background: var(--green)"></span>
          <span>Daily</span>
        </div>
      </div>

      {#if streak > 0}
        <div class="rings-streak {flameSize}">
          <div class="streak-fire-mini">
            <div class="streak-fire-mini-inner"></div>
            <div class="streak-fire-mini-outer"></div>
          </div>
          <span class="streak-label">{streak}</span>
        </div>
      {/if}
    </div>

    {#if isRunning}
      <div class="rings-meta">
        <span>{formatHoursDecimal(elapsed)}h worked</span>
        {#if breakElapsed > 0}
          <span class="rings-meta-sep">·</span>
          <span>{formatHoursDecimal(breakElapsed)}h break</span>
        {/if}
      </div>
    {/if}
  {/if}
</div>
