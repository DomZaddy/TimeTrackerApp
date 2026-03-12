<script>
  let { onComplete } = $props();

  let step = $state(0);
  let nameInput = $state("");

  function handleSubmit() {
    const name = nameInput.trim();
    if (name) onComplete(name);
  }
</script>

<div class="modal-overlay">
  <div class="modal" style="text-align: center">
    {#if step === 0}
      <div class="welcome-content">
        <div style="font-size: 48px; margin-bottom: 16px">⏱️</div>
        <h2 class="welcome-title">Welcome to TimeTracker</h2>
        <p class="welcome-subtitle">
          Track your hours, earn XP, and watch your goblins mine coins.
        </p>
        <button class="btn btn-primary" style="width: 100%" onclick={() => (step = 1)}>
          Get Started
        </button>
      </div>
    {:else}
      <div class="welcome-content">
        <h2 class="welcome-title">What's your name?</h2>
        <p class="welcome-subtitle">This is how your goblins will address you.</p>
        <input
          class="name-input"
          bind:value={nameInput}
          placeholder="Enter your name"
          onkeydown={(e) => e.key === "Enter" && handleSubmit()}
        />
        <button
          class="btn btn-primary"
          style="width: 100%"
          onclick={handleSubmit}
          disabled={!nameInput.trim()}
        >
          Start Tracking
        </button>
      </div>
    {/if}
  </div>
</div>
