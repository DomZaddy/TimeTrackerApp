<script>
  import { onMount, onDestroy } from 'svelte';
  import { reminderToast } from '../stores/reminderToast.js';
  import { playTone } from '../utils/tones.js';
  import { storeGet } from '../tauri.js';

  let visible = $state(false);
  let message = $state('');
  let dismissTimer = null;

  const unsub = reminderToast.subscribe((val) => {
    if (val) {
      message = val;
      visible = true;
      // Play the selected alarm tone
      storeGet('alarm-tone').then((tone) => {
        playTone(tone || 'chime');
      });
      if (dismissTimer) clearTimeout(dismissTimer);
      dismissTimer = setTimeout(() => {
        visible = false;
        reminderToast.set(null);
      }, 5000);
    }
  });

  onDestroy(() => {
    unsub();
    if (dismissTimer) clearTimeout(dismissTimer);
  });

  function dismiss() {
    visible = false;
    reminderToast.set(null);
    if (dismissTimer) clearTimeout(dismissTimer);
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="reminder-toast" onclick={dismiss}>
    <div class="reminder-toast-icon">🔔</div>
    <div class="reminder-toast-content">
      <div class="reminder-toast-title">Check-in Reminder</div>
      <div class="reminder-toast-message">{message}</div>
    </div>
    <button class="reminder-toast-close" onclick={dismiss}>✕</button>
  </div>
{/if}
