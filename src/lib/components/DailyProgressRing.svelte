<script>
  let {
    elapsed = 0,
    dailyGoalHours = 8,
    isRunning = false,
  } = $props();

  let hours = $derived(elapsed / 3600000);
  let percent = $derived(Math.min((hours / dailyGoalHours) * 100, 100));
  let circumference = $derived(2 * Math.PI * 38);
  let dashOffset = $derived(circumference - (percent / 100) * circumference);
  let displayHours = $derived(hours.toFixed(1));
  let isComplete = $derived(percent >= 100);
</script>

<div class="progress-ring-wrap">
  <svg class="progress-ring" viewBox="0 0 84 84">
    <circle
      class="progress-ring-bg"
      cx="42" cy="42" r="38"
      fill="none"
      stroke="rgba(108, 92, 231, 0.12)"
      stroke-width="5"
    />
    <circle
      class="progress-ring-fill {isComplete ? 'progress-ring-complete' : ''}"
      cx="42" cy="42" r="38"
      fill="none"
      stroke={isComplete ? 'var(--green)' : 'var(--accent)'}
      stroke-width="5"
      stroke-linecap="round"
      stroke-dasharray={circumference}
      stroke-dashoffset={dashOffset}
      transform="rotate(-90 42 42)"
    />
  </svg>
  <div class="progress-ring-inner">
    <div class="progress-ring-value">{displayHours}h</div>
    <div class="progress-ring-label">/ {dailyGoalHours}h</div>
  </div>
  {#if isRunning}
    <div class="progress-ring-pulse"></div>
  {/if}
</div>
