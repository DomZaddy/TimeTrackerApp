<script>
  import { onMount } from "svelte";
  import * as tauri from "../tauri.js";
  import * as gamification from "../stores/gamification.js";
  import * as weeklyHoursStore from "../stores/weeklyHours.js";
  import * as nudgesStore from "../stores/nudges.js";
  import { TONES, TONE_KEYS, playTone } from "../utils/tones.js";
  import { showReminder } from "../stores/reminderToast.js";
  import { get } from "svelte/store";

  let { userName = "", onSaveName, onStartTutorial, onAuthSuccess, updateStatus = "", updateVersion = "", onCheckUpdate, onInstallUpdate } = $props();

  let editingName = $state(false);
  let nameInput = $state("");
  let nameSaved = $state(false);
  let sheetId = $state("");
  let sheetIdSaved = $state(false);
  let isSignedIn = $state(false);
  let signingIn = $state(false);
  let authStatus = $state("");
  let autoLaunch = $state(true);
  let reminderHours = $state(4);
  let alarmTone = $state("chime");
  let weekLimit = $state(40);
  let overtimeMode = $state(false);
  let showResetConfirm = $state(false);


  // Gamification data
  let gData = $state({ xp: 0, level: 0, streaks: { daily: 0 }, achievements: [] });
  let levelName = $state("Intern");
  let xpProgress = $state(0);
  let soundsOn = $state(true);

  // Nudge settings — defaults true (except overtime above)
  let nudgeDailyEnabled = $state(true);
  let nudgeDailyHours = $state(10);
  let nudgeBreakEnabled = $state(true);
  let nudgeBreakHours = $state(2);
  let nudgeClockoutEnabled = $state(true);
  let nudgeEndHour = $state(19);

  const END_HOUR_OPTIONS = [15, 16, 17, 18, 19, 20, 21, 22];

  onMount(async () => {
    const sid = await tauri.storeGet("sheet-id");
    if (sid) sheetId = sid;

    const tokens = await tauri.storeGet("oauth-tokens");
    isSignedIn = !!tokens;

    const rh = await tauri.storeGet("reminder-hours");
    if (rh) reminderHours = rh;

    const savedTone = await tauri.storeGet("alarm-tone");
    if (savedTone) alarmTone = savedTone;

    try {
      const isAL = await tauri.getAutoLaunch();
      autoLaunch = isAL;
    } catch {}

    weekLimit = get(weeklyHoursStore.weekLimitHours);
    overtimeMode = get(weeklyHoursStore.overtimeMode);

    // Load nudge settings
    const ns = nudgesStore.getNudgeSettings();
    nudgeDailyEnabled = ns.dailyCap.enabled;
    nudgeDailyHours = ns.dailyCap.hours;
    nudgeBreakEnabled = ns.breakReminder.enabled;
    nudgeBreakHours = ns.breakReminder.hours;
    nudgeClockoutEnabled = ns.forgotClockOut.enabled;
    nudgeEndHour = ns.forgotClockOut.hour;

    const unsubs = [
      gamification.gamificationData.subscribe((d) => (gData = d)),
      gamification.levelName.subscribe((n) => (levelName = n)),
      gamification.xpProgress.subscribe((p) => (xpProgress = p)),
      gamification.soundsEnabled.subscribe((s) => (soundsOn = s)),
    ];

    return () => unsubs.forEach((u) => u());
  });

  async function handleSignIn() {
    signingIn = true;
    authStatus = "";
    const result = await tauri.sheetsAuth();
    signingIn = false;
    if (result.success) {
      isSignedIn = true;
      onAuthSuccess?.();
    } else if (result.error) {
      authStatus = result.error;
    }
  }

  async function handleSignOut() {
    await tauri.sheetsSignOut();
    isSignedIn = false;
    authStatus = "";
  }

  async function handleSaveSheetId() {
    const trimmed = sheetId.trim();
    if (!trimmed) return;
    const urlMatch = trimmed.match(/\/spreadsheets\/d\/([a-zA-Z0-9_-]+)/);
    const idToSave = urlMatch ? urlMatch[1] : trimmed;
    await tauri.storeSet("sheet-id", idToSave);
    sheetId = idToSave;
    sheetIdSaved = true;
    setTimeout(() => (sheetIdSaved = false), 2000);
  }

  async function setReminderHoursVal(hours) {
    reminderHours = hours;
    await tauri.storeSet("reminder-hours", hours);
  }

  async function handleToneChange(key) {
    alarmTone = key;
    await tauri.storeSet("alarm-tone", key);
    playTone(key);
  }

  async function handleAutoLaunchToggle() {
    const next = !autoLaunch;
    try {
      await tauri.setAutoLaunch(next);
      autoLaunch = next;
    } catch {}
  }

  async function handleWeekLimitChange(e) {
    const val = parseInt(e.target.value) || 40;
    weekLimit = val;
    await weeklyHoursStore.setWeekLimit(val);
  }

  async function handleOvertimeToggle() {
    overtimeMode = !overtimeMode;
    await weeklyHoursStore.setOvertimeMode(overtimeMode);
  }

  function startEditName() {
    nameInput = userName;
    editingName = true;
  }

  async function saveName() {
    const name = nameInput.trim();
    if (name) {
      await onSaveName(name);
      nameSaved = true;
      setTimeout(() => (nameSaved = false), 2000);
    }
    editingName = false;
  }

  async function handleReset() {
    await gamification.resetGamification();
    showResetConfirm = false;
  }

  async function toggleNudgeDaily() {
    nudgeDailyEnabled = !nudgeDailyEnabled;
    await nudgesStore.saveNudgeSettings("dailyCap", { enabled: nudgeDailyEnabled, hours: nudgeDailyHours });
  }
  async function updateNudgeDailyHours(e) {
    nudgeDailyHours = Number(e.target.value);
    await nudgesStore.saveNudgeSettings("dailyCap", { enabled: nudgeDailyEnabled, hours: nudgeDailyHours });
  }
  async function toggleNudgeBreak() {
    nudgeBreakEnabled = !nudgeBreakEnabled;
    await nudgesStore.saveNudgeSettings("breakReminder", { enabled: nudgeBreakEnabled, hours: nudgeBreakHours });
  }
  async function updateNudgeBreakHours(e) {
    nudgeBreakHours = Number(e.target.value);
    await nudgesStore.saveNudgeSettings("breakReminder", { enabled: nudgeBreakEnabled, hours: nudgeBreakHours });
  }
  async function toggleNudgeClockout() {
    nudgeClockoutEnabled = !nudgeClockoutEnabled;
    await nudgesStore.saveNudgeSettings("forgotClockOut", { enabled: nudgeClockoutEnabled, hour: nudgeEndHour });
  }
  async function updateNudgeEndHour(e) {
    nudgeEndHour = Number(e.target.value);
    await nudgesStore.saveNudgeSettings("forgotClockOut", { enabled: nudgeClockoutEnabled, hour: nudgeEndHour });
  }


  function formatHour(h) {
    if (h === 12) return "12:00 PM";
    if (h > 12) return `${h - 12}:00 PM`;
    return `${h}:00 AM`;
  }
</script>

<div class="settings">
  <!-- Profile -->
  <div class="settings-section">
    <h3>Profile</h3>
    <div class="setting-row">
      <div>
        <span class="setting-label">Username</span>
        <div class="setting-hint">Personalize your experience</div>
      </div>
      {#if editingName}
        <div style="display: flex; gap: 4px">
          <input
            class="task-input"
            style="width: 100px; padding: 6px 8px; font-size: 12px"
            bind:value={nameInput}
            maxlength="20"
            onkeydown={(e) => e.key === "Enter" && saveName()}
          />
          <button class="btn btn-ghost btn-sm" style="font-size: 11px" onclick={saveName}>
            {nameSaved ? "Saved!" : "Save"}
          </button>
        </div>
      {:else}
        <button
          class="setting-value"
          style="background: none; border: none; cursor: pointer; color: var(--accent)"
          onclick={startEditName}
        >
          {userName || "Set name"}
        </button>
      {/if}
    </div>
    {#if onStartTutorial}
      <div class="setting-row">
        <div>
          <span class="setting-label">App tutorial</span>
          <div class="setting-hint">Take a quick tour of the app</div>
        </div>
        <button class="btn btn-ghost btn-sm" style="font-size: 11px" onclick={onStartTutorial}>
          Replay
        </button>
      </div>
    {/if}
  </div>

  <!-- Google Account -->
  <div class="settings-section" id="google-connect">
    <h3>Step 1: Connect Google</h3>
    <p class="setting-hint" style="margin-bottom: 8px; line-height: 1.5">
      This lets the app write your hours to Google Sheets. Tap the button below — a browser window will open. Click "Continue", then "Allow" to give access.
    </p>
    {#if !isSignedIn}
      <button
        class="btn btn-primary"
        style="width: 100%; display: flex; align-items: center; justify-content: center; gap: 8px"
        onclick={handleSignIn}
        disabled={signingIn}
      >
        {signingIn ? "Connecting..." : "Sign in with Google"}
      </button>
    {:else}
      <div class="setting-row">
        <div style="display: flex; align-items: center; gap: 8px">
          <div style="width: 8px; height: 8px; border-radius: 50%; background: var(--green)"></div>
          <span style="color: var(--text-bright); font-size: 13px">Connected</span>
        </div>
        <button class="btn btn-ghost btn-sm" style="font-size: 11px" onclick={handleSignOut}>
          Sign Out
        </button>
      </div>
    {/if}
    {#if authStatus}
      <p style="font-size: 11px; color: var(--red); margin-top: 8px">{authStatus}</p>
    {/if}
  </div>

  <!-- Spreadsheet -->
  <div class="settings-section" id="sheet-link">
    <h3>Step 2: Link Your Sheet</h3>
    <p class="setting-hint" style="margin-bottom: 8px; line-height: 1.5">
      Open your Google Sheet in a browser, copy the whole URL from the address bar, and paste it here.
    </p>
    <input
      class="task-input"
      style="font-size: 12px"
      bind:value={sheetId}
      placeholder="Paste your Google Sheets link here"
    />
    <p style="font-size: 10px; color: var(--text-dim); margin-top: 4px; line-height: 1.6">
      It looks like: <span style="font-family: monospace; word-break: break-all">docs.google.com/spreadsheets/d/<span style="color: var(--accent)">abc123...</span>/edit</span>
    </p>
    <button
      class="btn btn-primary btn-sm"
      style="margin-top: 10px; min-width: 80px"
      onclick={handleSaveSheetId}
      disabled={!sheetId.trim()}
    >
      {sheetIdSaved ? "Saved!" : "Save"}
    </button>
  </div>

  <!-- Reminder Interval -->
  <div class="settings-section">
    <h3>Reminder Interval</h3>
    <p class="setting-hint" style="margin-bottom: 10px; line-height: 1.5">
      How often to nudge you to log notes
    </p>
    <div style="display: flex; gap: 4px">
      {#each [1, 2, 4] as h}
        <button
          class="btn"
          style="padding: 6px 14px; font-size: 12px; flex: 1; background: {reminderHours === h ? 'var(--accent)' : 'var(--surface)'}; color: {reminderHours === h ? 'white' : 'var(--text-dim)'}"
          onclick={() => setReminderHoursVal(h)}
        >
          {h} hour{h > 1 ? "s" : ""}
        </button>
      {/each}
    </div>
  </div>

  <!-- Alarm Tone -->
  <div class="settings-section">
    <h3>Alarm Tone</h3>
    <p class="setting-hint" style="margin-bottom: 10px; line-height: 1.5">
      Sound played with each reminder
    </p>
    <div class="tone-list">
      {#each TONE_KEYS as key}
        <button
          class="tone-option {alarmTone === key ? 'active' : ''}"
          onclick={() => handleToneChange(key)}
        >
          <div class="tone-radio">
            {#if alarmTone === key}<div class="tone-radio-dot"></div>{/if}
          </div>
          <div>
            <div style="font-size: 13px; color: var(--text-bright)">{TONES[key].label}</div>
            <div style="font-size: 10px; color: var(--text-dim)">{TONES[key].description}</div>
          </div>
        </button>
      {/each}
    </div>
  </div>

  <!-- Notifications -->
  <div class="settings-section">
    <h3>Notifications</h3>
    <div class="setting-row">
      <div>
        <span class="setting-label">Preview reminder</span>
        <div class="setting-hint">See how your reminder will look and sound</div>
      </div>
      <button class="btn btn-ghost btn-sm" style="font-size: 11px" onclick={() => showReminder("Time to log your progress! How's work going?")}>
        Preview
      </button>
    </div>
  </div>

  <!-- Startup -->
  <div class="settings-section">
    <h3>Startup</h3>
    <div class="setting-row">
      <div>
        <span class="setting-label">Launch on Windows login</span>
        <div class="setting-hint">Automatically start TimeTracker when you log in</div>
      </div>
      <button class="toggle" class:active={autoLaunch} onclick={handleAutoLaunchToggle} aria-label="Toggle auto launch">
        <div class="toggle-knob"></div>
      </button>
    </div>
  </div>

  <!-- Hours & Limits -->
  <div class="settings-section">
    <h3>Hours & Limits</h3>
    <div class="setting-row">
      <span class="setting-label">Weekly hour limit</span>
      <div style="display: flex; align-items: center; gap: 6px">
        <input
          type="number"
          class="task-input"
          style="width: 56px; padding: 6px 8px; font-size: 13px; text-align: center"
          value={weekLimit}
          min="1"
          max="168"
          onchange={handleWeekLimitChange}
        />
        <span style="font-size: 12px; color: var(--text-dim)">h / week</span>
      </div>
    </div>
    <div class="setting-row">
      <div>
        <span class="setting-label">Overtime display</span>
        <div class="setting-hint">Show hours past limit as overtime</div>
      </div>
      <button class="toggle" class:active={overtimeMode} onclick={handleOvertimeToggle} aria-label="Toggle overtime mode">
        <div class="toggle-knob"></div>
      </button>
    </div>
  </div>

  <!-- Gentle Nudges -->
  <div class="settings-section">
    <h3>Gentle Nudges</h3>
    <p class="setting-hint" style="margin-bottom: 10px; line-height: 1.5">
      Informational notifications only — never blocking
    </p>

    <div class="setting-row">
      <div>
        <span class="setting-label">Daily hour warning</span>
        <div class="setting-hint">Notify after working X hours today</div>
      </div>
      <div style="display: flex; align-items: center; gap: 6px">
        <input
          type="number"
          class="task-input"
          style="width: 60px; text-align: center; font-size: 12px; padding: 6px 8px; opacity: {nudgeDailyEnabled ? 1 : 0.4}"
          value={nudgeDailyHours}
          min="1"
          max="24"
          disabled={!nudgeDailyEnabled}
          onchange={updateNudgeDailyHours}
        />
        <button class="toggle" class:active={nudgeDailyEnabled} onclick={toggleNudgeDaily} aria-label="Toggle daily cap nudge">
          <div class="toggle-knob"></div>
        </button>
      </div>
    </div>

    <div class="setting-row">
      <div>
        <span class="setting-label">Break reminder</span>
        <div class="setting-hint">Nudge after X hours without a break</div>
      </div>
      <div style="display: flex; align-items: center; gap: 6px">
        <input
          type="number"
          class="task-input"
          style="width: 60px; text-align: center; font-size: 12px; padding: 6px 8px; opacity: {nudgeBreakEnabled ? 1 : 0.4}"
          value={nudgeBreakHours}
          min="0.5"
          max="12"
          step="0.5"
          disabled={!nudgeBreakEnabled}
          onchange={updateNudgeBreakHours}
        />
        <button class="toggle" class:active={nudgeBreakEnabled} onclick={toggleNudgeBreak} aria-label="Toggle break reminder">
          <div class="toggle-knob"></div>
        </button>
      </div>
    </div>

    <div class="setting-row">
      <div>
        <span class="setting-label">Forgot to clock out</span>
        <div class="setting-hint">Nudge if still clocked in after set time</div>
      </div>
      <div style="display: flex; align-items: center; gap: 6px">
        <select
          class="select-input"
          style="width: 90px; opacity: {nudgeClockoutEnabled ? 1 : 0.4}"
          value={nudgeEndHour}
          disabled={!nudgeClockoutEnabled}
          onchange={updateNudgeEndHour}
        >
          {#each END_HOUR_OPTIONS as h}
            <option value={h}>{formatHour(h)}</option>
          {/each}
        </select>
        <button class="toggle" class:active={nudgeClockoutEnabled} onclick={toggleNudgeClockout} aria-label="Toggle forgot clock out nudge">
          <div class="toggle-knob"></div>
        </button>
      </div>
    </div>
  </div>

  <!-- Gamification -->
  <div class="settings-section">
    <h3>Gamification</h3>

    <div class="gamification-stats">
      <div class="gamification-stat">
        <div class="gamification-stat-value">{gData.level}</div>
        <div class="gamification-stat-label">{levelName}</div>
      </div>
      <div class="gamification-stat">
        <div class="gamification-stat-value">{gData.streaks?.daily || 0}</div>
        <div class="gamification-stat-label">Day Streak</div>
      </div>
      <div class="gamification-stat">
        <div class="gamification-stat-value">{gData.achievements?.length || 0}</div>
        <div class="gamification-stat-label">Badges</div>
      </div>
    </div>

    <div style="margin-top: 10px">
      <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-dim); margin-bottom: 4px">
        <span>XP: {gData.xp % 500} / 500</span>
        <span>Total: {gData.xp}</span>
      </div>
      <div class="xp-bar">
        <div class="xp-fill" style="width: {xpProgress * 100}%"></div>
      </div>
    </div>

    <div class="setting-row" style="margin-top: 12px">
      <span class="setting-label">Sound effects</span>
      <button class="toggle" class:active={soundsOn} onclick={gamification.toggleSounds} aria-label="Toggle sound effects">
        <div class="toggle-knob"></div>
      </button>
    </div>

    <div style="margin-top: 10px">
      {#if showResetConfirm}
        <div style="display: flex; gap: 8px; align-items: center">
          <span style="font-size: 11px; color: var(--red)">Are you sure?</span>
          <button class="btn btn-stop" style="padding: 6px 12px; font-size: 11px" onclick={handleReset}>
            Yes, reset
          </button>
          <button class="btn btn-ghost btn-sm" style="font-size: 11px" onclick={() => (showResetConfirm = false)}>
            Cancel
          </button>
        </div>
      {:else}
        <button
          class="btn btn-ghost btn-sm"
          style="font-size: 11px; color: var(--text-dim)"
          onclick={() => (showResetConfirm = true)}
        >
          Reset Stats
        </button>
      {/if}
    </div>
  </div>

  <!-- App Info & Updates -->
  <div class="settings-section">
    <h3>About & Updates</h3>
    <div class="setting-row">
      <div>
        <span class="setting-label">Current version</span>
        <div class="setting-hint">TimeTracker v{__APP_VERSION__}</div>
      </div>
      {#if updateStatus === "available"}
        <button class="btn btn-start" style="padding: 6px 14px; font-size: 12px" onclick={onInstallUpdate}>
          Update to v{updateVersion}
        </button>
      {:else if updateStatus === "checking"}
        <span style="font-size: 11px; color: var(--text-dim)">Checking...</span>
      {:else if updateStatus === "downloading"}
        <span style="font-size: 11px; color: var(--green)">Downloading...</span>
      {:else if updateStatus === "ready"}
        <span style="font-size: 11px; color: var(--accent)">Restarting...</span>
      {:else}
        <button class="btn btn-ghost btn-sm" style="font-size: 11px" onclick={onCheckUpdate}>
          Check for Updates
        </button>
      {/if}
    </div>
  </div>
</div>
