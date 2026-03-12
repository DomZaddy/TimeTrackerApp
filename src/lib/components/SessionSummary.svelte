<script>
  let { blocks = [], onPush, onDiscard, isPushing = false } = $props();
</script>

<div class="modal-overlay">
  <div class="modal" style="max-width: 400px">
    <h2>Session Summary</h2>

    <div class="summary-blocks">
      {#each blocks as block, i}
        <div class="summary-block">
          <div class="summary-block-header">
            <span>{block.date}</span>
            <span>{block.checkIn} - {block.checkOut}</span>
          </div>
          <div class="summary-block-task">
            {block.task || "(no task)"}
          </div>
          {#if block.breakHours > 0}
            <div style="font-size: 11px; color: var(--orange); margin-top: 4px">
              Break: {block.breakHours}h
            </div>
          {/if}
        </div>
      {/each}
    </div>

    <div class="summary-actions">
      <button
        class="btn btn-primary"
        style="flex: 1"
        onclick={() => onPush(blocks)}
        disabled={isPushing}
      >
        {isPushing ? "Pushing..." : `Push ${blocks.length} Block${blocks.length > 1 ? "s" : ""}`}
      </button>
      <button class="btn btn-break" style="flex: 1" onclick={onDiscard} disabled={isPushing}>
        Discard
      </button>
    </div>
  </div>
</div>
