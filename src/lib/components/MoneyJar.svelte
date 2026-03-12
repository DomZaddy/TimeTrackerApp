<script>
  import { formatHoursDecimal } from '../utils/formatTime.js';

  let {
    monthlyPay = 0,
    monthlyHours = 0,
    weekLimitHours = 40,
    isRunning = false,
    elapsed = 0,
  } = $props();

  const PAY_TIERS = [
    { min: 0, label: 'Piggy Bank', emoji: '🐷' },
    { min: 500, label: 'Money Jar', emoji: '🏺' },
    { min: 1000, label: 'Cash Stack', emoji: '💵' },
    { min: 2000, label: 'Money Maker', emoji: '💰' },
    { min: 3000, label: 'Big Spender', emoji: '🤑' },
    { min: 5000, label: 'Bank Roll', emoji: '🏦' },
    { min: 7500, label: 'Money Machine', emoji: '⚙️' },
    { min: 10000, label: 'Diamond Hands', emoji: '💎' },
  ];

  function getTier(pay) {
    let tier = PAY_TIERS[0];
    for (const t of PAY_TIERS) { if (pay >= t.min) tier = t; }
    const idx = PAY_TIERS.indexOf(tier);
    const nextTier = PAY_TIERS[idx + 1] || null;
    const progress = nextTier ? ((pay - tier.min) / (nextTier.min - tier.min)) * 100 : 100;
    return { ...tier, idx, nextTier, progress: Math.min(progress, 100) };
  }

  let payRate = $derived(monthlyHours > 0 ? monthlyPay / monthlyHours : 0);
  let maxMonthlyHours = $derived(weekLimitHours * 4.33);
  let maxMonthlyPay = $derived(payRate > 0 ? maxMonthlyHours * payRate : 0);
  let jarPercent = $derived(maxMonthlyPay > 0 ? Math.min((monthlyPay / maxMonthlyPay) * 100, 100) : 0);
  let tier = $derived(getTier(monthlyPay));
  let currentSessionHours = $derived(isRunning ? formatHoursDecimal(elapsed) : 0);
</script>

<div class="earnings-page">
  <!-- Tier card -->
  <div class="earnings-tier-card">
    <span class="earnings-tier-emoji">{tier.emoji}</span>
    <div class="earnings-tier-info">
      <div class="earnings-tier-name">{tier.label}</div>
      {#if tier.nextTier}
        <div class="earnings-tier-next-text">
          Next: {tier.nextTier.emoji} {tier.nextTier.label} at ${tier.nextTier.min.toLocaleString()}
        </div>
      {/if}
    </div>
    {#if tier.nextTier}
      <div class="earnings-tier-bar">
        <div class="earnings-tier-bar-fill" style="width: {tier.progress}%"></div>
      </div>
    {/if}
  </div>

  <!-- Jar visualization -->
  <div class="earnings-jar-wrap">
    <div class="jar-visual">
      <div class="jar-body">
        <div class="jar-liquid" style="height: {jarPercent}%;">
          <div class="jar-liquid-shine"></div>
        </div>
        <div class="jar-shine"></div>
      </div>
      <div class="jar-lid"></div>
    </div>

    <div class="earnings-amount">${monthlyPay.toFixed(2)}</div>
    <div class="earnings-subtitle">
      {#if maxMonthlyPay > 0}
        of ${maxMonthlyPay.toFixed(2)} possible this month
      {:else}
        {monthlyHours.toFixed(1)} hours this month
      {/if}
    </div>
  </div>

  <!-- Stats grid -->
  <div class="earnings-stats">
    <div class="earnings-stat-card">
      <div class="earnings-stat-value">{payRate > 0 ? `$${payRate.toFixed(2)}` : '—'}</div>
      <div class="earnings-stat-label">Pay Rate /hr</div>
    </div>
    <div class="earnings-stat-card">
      <div class="earnings-stat-value">{monthlyHours.toFixed(1)}h</div>
      <div class="earnings-stat-label">Hours Worked</div>
    </div>
    <div class="earnings-stat-card">
      <div class="earnings-stat-value">{currentSessionHours}h</div>
      <div class="earnings-stat-label">This Session</div>
    </div>
    <div class="earnings-stat-card">
      <div class="earnings-stat-value">{Math.round(jarPercent)}%</div>
      <div class="earnings-stat-label">Monthly Goal</div>
    </div>
  </div>

  {#if isRunning}
    <div class="earnings-active">
      <span class="earnings-active-dot"></span>
      Currently earning...
    </div>
  {/if}
</div>
