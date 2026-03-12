<script>
  import { gamificationData } from '../stores/gamification.js';

  let streak = $state(0);
  const unsub = gamificationData.subscribe(d => { streak = d.streaks?.daily || 0; });

  import { onDestroy } from 'svelte';
  onDestroy(() => unsub());

  let flameSize = $derived(
    streak >= 30 ? 'flame-legendary'
    : streak >= 14 ? 'flame-large'
    : streak >= 7 ? 'flame-medium'
    : streak >= 3 ? 'flame-small'
    : ''
  );
</script>

{#if streak > 0}
  <div class="streak-flame {flameSize}">
    <div class="streak-fire">
      <div class="streak-fire-inner"></div>
      <div class="streak-fire-outer"></div>
    </div>
    <span class="streak-count">{streak}</span>
  </div>
{:else}
  <div class="streak-flame streak-inactive">
    <div class="streak-fire streak-fire-dim">
      <div class="streak-fire-inner"></div>
    </div>
    <span class="streak-count">0</span>
  </div>
{/if}
