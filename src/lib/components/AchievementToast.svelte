<script>
  let { queue = [], onDismiss } = $props();

  let current = $derived(queue.length > 0 ? queue[0] : null);

  $effect(() => {
    if (current) {
      const t = setTimeout(onDismiss, 4000);
      return () => clearTimeout(t);
    }
  });
</script>

{#if current}
  <div class="achievement-toast">
    <span class="achievement-icon">{current.icon}</span>
    <div class="achievement-text">
      <h4>{current.title}</h4>
      <p>{current.desc}</p>
    </div>
  </div>
{/if}
