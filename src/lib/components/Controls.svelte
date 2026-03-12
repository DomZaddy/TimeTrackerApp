<script>
  let { isRunning = false, isOnBreak = false, onClockIn, onClockOut, onToggleBreak, compact = false } = $props();
</script>

<div class="controls">
  {#if !isRunning}
    <button class="btn btn-primary" onclick={onClockIn}>
      Clock In
    </button>
  {:else}
    <button class="btn btn-stop" onclick={onClockOut}>
      {compact ? "Stop" : "Clock Out"}
    </button>
    <button
      class="btn {isOnBreak ? 'btn-resume' : 'btn-break'}"
      onclick={onToggleBreak}
    >
      {isOnBreak ? "Resume" : "Break"}
    </button>
  {/if}
</div>
