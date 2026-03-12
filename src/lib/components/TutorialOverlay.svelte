<script>
  import { onMount } from 'svelte';
  import * as tauri from '../tauri.js';

  let { userName = 'friend', onComplete, onSwitchTab } = $props();

  const STEPS = [
    {
      target: '.activity-rings',
      icon: '👋',
      title: 'Welcome!',
      text: "Hey {name}! This app tracks your work hours and sends them to Google Sheets. Let's set it up — it only takes a minute!",
      tab: 'tracker',
    },
    {
      target: '.activity-rings',
      icon: '⏱',
      title: 'Your Timer',
      text: "These rings show how much you've worked today and this week. They fill up as you go!",
      tab: 'tracker',
    },
    {
      target: '.controls',
      icon: '🎮',
      title: 'Start & Stop',
      text: "Tap Clock In when you start working, Clock Out when you're done. Easy!",
      tab: 'tracker',
    },
    {
      target: '.ooo-section',
      icon: '🏖',
      title: 'Out of Office',
      text: "Taking a day off? Tap the OOO button to mark today as Out of Office — pick a reason like PTO, Sick, or type your own. Missed days are auto-filled too!",
      tab: 'tracker',
    },
    {
      target: '.task-section',
      icon: '📋',
      title: 'What Are You Doing?',
      text: "Type what you're working on here. This gets saved to your timesheet so you remember later!",
      tab: 'tracker',
    },
    {
      target: '.notes-section',
      icon: '📝',
      title: 'Session Notes',
      text: "Write quick notes about what you did. These get added to your Google Sheet when you clock out!",
      tab: 'tracker',
    },
    {
      target: '.view-tab:nth-child(2)',
      icon: '💰',
      title: 'Earnings Tab',
      text: "This tab shows how much money you've earned. Watch the jar fill up as you work!",
      tab: 'earnings',
    },
    {
      target: '#google-connect',
      icon: '🔗',
      title: 'Step 1: Connect Google',
      text: "Tap 'Sign in with Google' — a browser opens, click Continue, then Allow. That links your account!",
      tab: 'settings',
      scrollTo: true,
    },
    {
      target: '#sheet-link',
      icon: '📄',
      title: 'Step 2: Paste Your Sheet Link',
      text: "Open your Google Sheet, copy the URL from your browser's address bar, paste it here, and hit Save. Done!",
      tab: 'settings',
      scrollTo: true,
    },
    {
      target: '.weekly-bar',
      icon: '📊',
      title: 'Weekly Progress',
      text: "This bar shows your weekly hours. Green = on track. Red = overtime. You got this, {name}!",
      tab: 'tracker',
    },
  ];

  let currentStep = $state(0);
  let spotlightRect = $state(null);
  let entering = $state(true);
  let panelTop = $state(null);
  let panelBottom = $state(null);

  let step = $derived(STEPS[currentStep]);
  let displayText = $derived(step.text.replace(/\{name\}/g, userName || 'friend'));

  const PANEL_H = 180; // approximate panel height
  const GAP = 12;

  function updateSpotlight() {
    const el = document.querySelector(step.target);
    if (el) {
      if (step.scrollTo) {
        el.scrollIntoView({ behavior: 'instant', block: 'center' });
      }
      // Small delay to let scroll settle
      setTimeout(() => {
        const rect = el.getBoundingClientRect();
        const pad = 8;
        spotlightRect = {
          top: rect.top - pad,
          left: rect.left - pad,
          width: rect.width + pad * 2,
          height: rect.height + pad * 2,
        };
        positionPanel();
      }, step.scrollTo ? 50 : 0);
    } else {
      spotlightRect = null;
      // Fallback: center vertically
      panelTop = null;
      panelBottom = null;
    }
  }

  function positionPanel() {
    if (!spotlightRect) return;
    const winH = window.innerHeight;
    const spotBottom = spotlightRect.top + spotlightRect.height;
    const spaceBelow = winH - spotBottom;
    const spaceAbove = spotlightRect.top;

    if (spaceBelow >= PANEL_H + GAP) {
      // Place below the spotlight
      panelTop = spotBottom + GAP;
      panelBottom = null;
    } else if (spaceAbove >= PANEL_H + GAP) {
      // Place above the spotlight
      panelTop = null;
      panelBottom = winH - spotlightRect.top + GAP;
    } else {
      // Not enough space either way — place at bottom with some margin
      panelTop = null;
      panelBottom = 16;
    }
  }

  let panelStyleStr = $derived.by(() => {
    if (panelTop != null) {
      return `top: ${panelTop}px; left: 50%; transform: translateX(-50%);`;
    } else if (panelBottom != null) {
      return `bottom: ${panelBottom}px; left: 50%; transform: translateX(-50%);`;
    }
    return `top: 50%; left: 50%; transform: translate(-50%, -50%);`;
  });

  function switchToTab() {
    if (step.tab && onSwitchTab) {
      onSwitchTab(step.tab);
      setTimeout(updateSpotlight, 80);
    } else {
      updateSpotlight();
    }
  }

  function handleNext() {
    if (currentStep < STEPS.length - 1) {
      currentStep++;
      entering = true;
      setTimeout(() => entering = false, 350);
      switchToTab();
    } else {
      handleFinish();
    }
  }

  function handleBack() {
    if (currentStep > 0) {
      currentStep--;
      entering = true;
      setTimeout(() => entering = false, 350);
      switchToTab();
    }
  }

  async function handleFinish() {
    if (onSwitchTab) onSwitchTab('tracker');
    await tauri.storeSet('tutorial-completed', true);
    onComplete();
  }

  onMount(() => {
    switchToTab();
    setTimeout(() => entering = false, 350);
    const handleResize = () => updateSpotlight();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  });
</script>

<div class="tutorial-overlay">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="tutorial-backdrop" onclick={handleFinish}></div>

  {#if spotlightRect}
    <div
      class="tutorial-spotlight"
      style="top: {spotlightRect.top}px; left: {spotlightRect.left}px; width: {spotlightRect.width}px; height: {spotlightRect.height}px;"
    ></div>
  {/if}

  <div
    class="tutorial-panel {entering ? 'tutorial-panel--entering' : ''}"
    style={panelStyleStr}
  >
    <div class="tutorial-icon-row">
      <div class="tutorial-icon">{step.icon}</div>
      <div class="tutorial-speech-bubble">
        <div class="tutorial-step-title">{step.title}</div>
        <div class="tutorial-step-text">{displayText}</div>
      </div>
    </div>

    <div class="tutorial-dots">
      {#each STEPS as _, i}
        <div class="tutorial-dot {i === currentStep ? 'active' : ''} {i < currentStep ? 'completed' : ''}"></div>
      {/each}
    </div>

    <div class="tutorial-nav">
      <button class="btn btn-ghost btn-sm tutorial-skip" onclick={handleFinish}>
        Skip Tutorial
      </button>
      <div class="tutorial-nav-btns">
        {#if currentStep > 0}
          <button class="btn btn-ghost btn-sm" onclick={handleBack}>Back</button>
        {/if}
        <button class="btn btn-primary btn-sm" onclick={handleNext}>
          {currentStep === STEPS.length - 1 ? "Let's Go!" : 'Next'}
        </button>
      </div>
    </div>
  </div>
</div>
