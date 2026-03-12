<script>
  import { onMount } from "svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import ActivityRings from "./lib/components/ActivityRings.svelte";
  import Controls from "./lib/components/Controls.svelte";
  import NoteInput from "./lib/components/NoteInput.svelte";
  import TimePicker from "./lib/components/TimePicker.svelte";
  import SessionSummary from "./lib/components/SessionSummary.svelte";
  import ReminderToast from "./lib/components/ReminderToast.svelte";
  import RecoveryPrompt from "./lib/components/RecoveryPrompt.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import WeeklyProgressBar from "./lib/components/WeeklyProgressBar.svelte";
  import MoneyJar from "./lib/components/MoneyJar.svelte";
  import Confetti from "./lib/components/Confetti.svelte";
  import AchievementToast from "./lib/components/AchievementToast.svelte";
  import WelcomeModal from "./lib/components/WelcomeModal.svelte";
  import TutorialOverlay from "./lib/components/TutorialOverlay.svelte";
  import * as timer from "./lib/stores/timer.js";
  import * as session from "./lib/stores/session.js";
  import * as gamification from "./lib/stores/gamification.js";
  import * as weeklyHours from "./lib/stores/weeklyHours.js";
  import * as nudges from "./lib/stores/nudges.js";
  import * as tauri from "./lib/tauri.js";
  import { parseSessionToBlocks } from "./lib/utils/parseSession.js";
  import { formatElapsed, formatHoursDecimal } from "./lib/utils/formatTime.js";
  import { playClockIn, playClockOut, playBreakStart } from "./lib/utils/microFeedback.js";
  import { get } from "svelte/store";

  const SAVE_INTERVAL_MS = 30000;
  const MAX_SESSION_AGE_MS = 24 * 60 * 60 * 1000;
  const MAX_CACHED_SESSIONS = 35;
  const OOO_CHECK_INTERVAL_MS = 30 * 60 * 1000; // 30 minutes

  let view = $state("tracker");
  let widgetMode = $state(false);
  let taskInput = $state("");
  let showTimePicker = $state(false);
  let timePickerMode = $state('start');
  let showSummary = $state(false);
  let summaryBlocks = $state([]);
  let isPushing = $state(false);
  let pushStatus = $state(null);
  let recoveryData = $state(null);
  let showWelcome = $state(false);
  let showTutorial = $state(false);
  let userName = $state("");
  let taskCommitTimeout = null;
  let saveInterval = null;
  let oooCheckInterval = null;
  let lastOooCheckDay = null;
  let showOooMenu = $state(false);
  let oooStatus = $state(""); // "", "marking", "done"
  let oooCustomReason = $state("");
  let showOooCustomInput = $state(false);

  // Subscribe to stores
  let elapsed = $state(0);
  let breakElapsed = $state(0);
  let isRunning = $state(false);
  let isOnBreak = $state(false);
  let notes = $state([]);
  let toastQueue = $state([]);
  let confettiTrigger = $state(0);
  let totalWeekMs = $state(0);
  let weekPercent = $state(0);
  let weekLimitH = $state(40);
  let isOvertime = $state(false);
  let overtimeMs = $state(0);
  let overtimeModeVal = $state(false);
  let mPay = $state(0);
  let mHours = $state(0);
  let soundsOn = $state(true);

  // Updater state
  let updateStatus = $state(""); // "", "checking", "available", "downloading", "ready", "error"
  let updateVersion = $state("");
  let updateProgress = $state(0);
  let updateRef = null;
  let updateError = $state("");

  // Taskbar badge: cache badge icons so we don't regenerate every tick
  let workingBadge = null;
  let breakBadge = null;
  let badgesReady = $state(false);

  // Pre-generate badge icons once
  async function initBadges() {
    workingBadge = await tauri.makeBadgeIcon('#00d68f');
    breakBadge = await tauri.makeBadgeIcon('#f0932b');
    badgesReady = true;
  }
  initBadges();

  // Taskbar progress + overlay badge updates
  $effect(() => {
    if (!badgesReady) return;
    if (isRunning) {
      // Overlay badge: green = working, orange = break
      tauri.setOverlayIcon(isOnBreak ? breakBadge : workingBadge);

      // Taskbar progress bar: daily progress percentage
      const dailyGoalMs = (weekLimitH / 5) * 3600000;
      const pct = dailyGoalMs > 0 ? (elapsed / dailyGoalMs) * 100 : 0;
      tauri.setTaskbarProgress(pct, isOnBreak ? 'paused' : 'normal');
    } else {
      tauri.clearTaskbarBadge();
    }
  });

  onMount(() => {
    const unsubs = [
      timer.elapsed.subscribe((v) => (elapsed = v)),
      timer.breakElapsed.subscribe((v) => (breakElapsed = v)),
      timer.isRunning.subscribe((v) => (isRunning = v)),
      timer.isOnBreak.subscribe((v) => (isOnBreak = v)),
      session.notes.subscribe((v) => (notes = v)),
      gamification.toastQueue.subscribe((v) => (toastQueue = v)),
      gamification.confettiTrigger.subscribe((v) => (confettiTrigger = v)),
      weeklyHours.totalWeekMs.subscribe((v) => (totalWeekMs = v)),
      weeklyHours.weekPercent.subscribe((v) => (weekPercent = v)),
      weeklyHours.weekLimitHours.subscribe((v) => (weekLimitH = v)),
      weeklyHours.isOvertime.subscribe((v) => (isOvertime = v)),
      weeklyHours.overtimeMs.subscribe((v) => (overtimeMs = v)),
      weeklyHours.overtimeMode.subscribe((v) => (overtimeModeVal = v)),
      weeklyHours.monthlyPay.subscribe((v) => (mPay = v)),
      weeklyHours.monthlyHours.subscribe((v) => (mHours = v)),
      gamification.soundsEnabled.subscribe((v) => (soundsOn = v)),
    ];

    init();

    // Auto-check for updates on launch (delay 3s so UI loads first)
    setTimeout(autoCheckUpdate, 3000);
    // Re-check for updates every 10 minutes
    const updateCheckInterval = setInterval(autoCheckUpdate, 10 * 60 * 1000);
    // Also check when window regains focus (throttled to once per 2 minutes)
    let lastFocusCheck = 0;
    function onFocusCheck() {
      const now = Date.now();
      if (now - lastFocusCheck > 2 * 60 * 1000 && updateStatus !== "available" && updateStatus !== "downloading") {
        lastFocusCheck = now;
        autoCheckUpdate();
      }
    }
    window.addEventListener("focus", onFocusCheck);
    document.addEventListener("visibilitychange", () => {
      if (!document.hidden) onFocusCheck();
    });

    // Backfill OOO on launch (delay 5s so auth loads first)
    setTimeout(backfillOoo, 5000);

    // Idle day watcher: check every 30min if we crossed midnight without clocking in
    lastOooCheckDay = new Date().toDateString();
    oooCheckInterval = setInterval(idleDayCheck, OOO_CHECK_INTERVAL_MS);

    return () => {
      unsubs.forEach((u) => u());
      if (saveInterval) clearInterval(saveInterval);
      if (oooCheckInterval) clearInterval(oooCheckInterval);
      if (updateCheckInterval) clearInterval(updateCheckInterval);
      window.removeEventListener("focus", onFocusCheck);
      nudges.stopNudgeChecks();
    };
  });

  async function init() {
    await gamification.loadGamification();
    await weeklyHours.loadWeeklySettings();
    await nudges.loadNudgeSettings();
    await weeklyHours.fetchWeekHours();

    const name = await tauri.storeGet("user-name");
    if (name) {
      userName = name;
    } else {
      showWelcome = true;
    }

    const saved = await tauri.storeGet("active-session");
    if (saved && Date.now() - saved.savedAt < MAX_SESSION_AGE_MS) {
      recoveryData = saved;
    } else if (saved) {
      await tauri.storeDelete("active-session");
    }
  }

  function handleTaskChange(value) {
    taskInput = value;
    if (taskCommitTimeout) clearTimeout(taskCommitTimeout);
    taskCommitTimeout = setTimeout(() => {
      session.commitTask(value);
    }, 2000);
  }

  function handleTaskBlur() {
    if (taskCommitTimeout) clearTimeout(taskCommitTimeout);
    session.commitTask(taskInput);
  }

  function buildSnapshot() {
    const snap = timer.getSnapshot();
    return {
      version: 1,
      savedAt: Date.now(),
      startTime: snap.startTime,
      tasks: get(session.tasks),
      taskInput,
      notes: get(session.notes),
      breaks: get(session.breaks),
      totalBreakMs: snap.totalBreakMs,
      isOnBreak: snap.isOnBreak,
      breakStartTime: snap.breakStartTime,
    };
  }

  async function saveSession() {
    if (!isRunning) return;
    const snapshot = buildSnapshot();
    await tauri.storeSet("active-session", snapshot);
    await tauri.sheetsStatusUpdate({
      task: get(session.currentTask),
      start_time: snapshot.startTime,
      elapsed,
    });
  }

  function handleClockIn() {
    timePickerMode = 'start';
    showTimePicker = true;
  }

  function handleClockOut() {
    timePickerMode = 'stop';
    showTimePicker = true;
  }

  function handleToggleBreakPicker() {
    timePickerMode = isOnBreak ? 'break-end' : 'break-start';
    showTimePicker = true;
  }

  async function handleTimeConfirm(customTime) {
    showTimePicker = false;

    if (timePickerMode === 'start') {
      const startTime = customTime || Date.now();
      timer.start(customTime);
      session.startSession(startTime);
      // Preserve any task text entered before clocking in
      if (taskInput.trim()) {
        session.commitTask(taskInput);
      }
      gamification.onClockIn(startTime);
      nudges.startNudgeChecks();
      if (soundsOn) playClockIn();

      tauri.sheetsStatusStart({ task: taskInput || "", start_time: startTime, elapsed: 0 });
      weeklyHours.fetchWeekHours(true);

      if (saveInterval) clearInterval(saveInterval);
      saveInterval = setInterval(saveSession, SAVE_INTERVAL_MS);
    } else if (timePickerMode === 'break-start') {
      // Start a break (with optional custom time)
      const breakTime = customTime || Date.now();
      session.startBreak();
      if (soundsOn) playBreakStart();
      // If custom time, adjust the timer to account for the difference
      if (customTime) {
        const diff = Date.now() - customTime;
        timer.addBreakMs(diff);
      }
      timer.toggleBreak();
    } else if (timePickerMode === 'break-end') {
      // End a break (with optional custom time)
      if (customTime) {
        // Calculate how much break time to remove (we over-counted from break start to now, should be break start to custom time)
        const breakStartMs = timer.getSnapshot().breakStartTime;
        if (breakStartMs) {
          const actualBreak = customTime - breakStartMs;
          const overcounted = Date.now() - breakStartMs;
          // Timer will add overcounted when toggleBreak is called, so pre-subtract the difference
          timer.removeBreakMs(overcounted - actualBreak);
        }
      }
      session.endBreak();
      nudges.onBreakEnd();
      timer.toggleBreak();
    } else {
      // Clock out
      if (taskCommitTimeout) clearTimeout(taskCommitTimeout);
      session.commitTask(taskInput);

      const endTime = customTime || Date.now();
      const timerData = timer.stop(endTime);
      session.endSession(endTime);
      nudges.stopNudgeChecks();
      if (soundsOn) playClockOut();

      if (saveInterval) clearInterval(saveInterval);
      saveInterval = null;

      const sessionData = session.getSessionData();
      if (sessionData) {
        sessionData.endTime = endTime;
        summaryBlocks = parseSessionToBlocks(sessionData);

        // Cache the session before pushing (backup/memory)
        await cacheSession(buildSnapshot(), summaryBlocks);

        // Push session blocks
        await handlePush(summaryBlocks);
      }

      // Only clear active-session after successful push
      if (pushStatus && !pushStatus.startsWith("Error") && !pushStatus.startsWith("Push failed")) {
        await tauri.storeDelete("active-session");
      }
    }
  }


  async function cacheSession(snapshot, blocks) {
    const entry = {
      snapshot,
      blocks,
      cachedAt: Date.now(),
      date: new Date(snapshot.startTime).toLocaleDateString(),
    };
    const cache = (await tauri.storeGet("session-cache")) || [];
    cache.unshift(entry); // newest first
    if (cache.length > MAX_CACHED_SESSIONS) cache.length = MAX_CACHED_SESSIONS;
    await tauri.storeSet("session-cache", cache);
  }

  async function rePushLastSession() {
    const cache = (await tauri.storeGet("session-cache")) || [];
    if (cache.length === 0) return;
    const last = cache[0];
    if (last.blocks && last.blocks.length > 0) {
      await handlePush(last.blocks);
    }
  }

  async function handlePush(blocks) {
    isPushing = true;
    pushStatus = null;

    let result;
    try {
      result = await tauri.sheetsPush(blocks);
    } catch (err) {
      console.error("sheetsPush invoke error:", err);
      isPushing = false;
      pushStatus = `Push failed: ${err}`;
      return;
    }
    isPushing = false;

    if (result.success) {
      pushStatus = `Pushed ${result.row_count} rows`;
      weeklyHours.fetchWeekHours(true);

      const sessionData = session.getSessionData();
      const elapsedMs = sessionData ? sessionData.endTime - sessionData.startTime : 0;
      gamification.onSessionPush({
        elapsedMs,
        notesCount: notes.length,
        payAccrued: mPay,
        monthTab: "",
      });

      setTimeout(() => {
        showSummary = false;
        session.clearSession();
        taskInput = "";
        pushStatus = null;
      }, 1500);
    } else {
      pushStatus = `Error: ${result.error}`;
    }
  }

  function handleDiscard() {
    showSummary = false;
    session.clearSession();
    taskInput = "";
    tauri.storeDelete("active-session");
    tauri.sheetsStatusClear();
  }

  async function handleResumeSession() {
    if (!recoveryData) return;
    try {
      timer.restore({
        startTime: recoveryData.startTime,
        totalBreakMs: recoveryData.totalBreakMs || 0,
        breakStartTime: recoveryData.breakStartTime,
        isOnBreak: recoveryData.isOnBreak || false,
      });
      session.restoreSession({
        startTime: recoveryData.startTime,
        notes: Array.isArray(recoveryData.notes) ? recoveryData.notes : [],
        breaks: Array.isArray(recoveryData.breaks) ? recoveryData.breaks : [],
        tasks: Array.isArray(recoveryData.tasks) ? recoveryData.tasks : [],
      });
      const lastTask =
        recoveryData.taskInput ||
        (Array.isArray(recoveryData.tasks) && recoveryData.tasks.length > 0
          ? recoveryData.tasks[recoveryData.tasks.length - 1].name
          : "");
      taskInput = lastTask;
      nudges.startNudgeChecks();

      // Clear any stale live status, then write fresh "Currently working" row
      await tauri.sheetsStatusClear();
      await tauri.sheetsStatusStart({
        task: lastTask,
        start_time: recoveryData.startTime,
        elapsed: 0,
      });

      if (saveInterval) clearInterval(saveInterval);
      saveInterval = setInterval(saveSession, SAVE_INTERVAL_MS);
    } catch (e) {
      console.error("Recovery failed:", e);
    }
    recoveryData = null;
  }

  function handleDiscardRecovery() {
    tauri.storeDelete("active-session").catch(() => {});
    recoveryData = null;
  }

  async function autoCheckUpdate() {
    // Don't interrupt an active download
    if (updateStatus === "downloading" || updateStatus === "ready") return;
    try {
      updateStatus = "checking";
      updateError = "";
      console.log("[updater] checking for updates...");

      // Try Tauri updater plugin first
      const result = await tauri.checkForUpdate();
      if (result.available) {
        updateStatus = "available";
        updateVersion = result.version;
        updateRef = result;
        console.log(`[updater] update available via plugin: v${result.version}`);
        return;
      }

      // Fallback: direct HTTP fetch of latest.json to verify
      if (!result.available) {
        console.log("[updater] plugin says no update, double-checking via HTTP...");
        try {
          const resp = await fetch("https://github.com/DomZaddy/TimeTrackerApp/releases/latest/download/latest.json");
          if (resp.ok) {
            const data = await resp.json();
            const currentVersion = __APP_VERSION__;
            console.log(`[updater] current: v${currentVersion}, latest: v${data.version}`);
            if (data.version && data.version !== currentVersion && isNewerVersion(data.version, currentVersion)) {
              // Plugin missed it — set status to available with download link
              updateStatus = "available";
              updateVersion = data.version;
              updateRef = result.available ? result : null;
              console.log(`[updater] HTTP check found update: v${data.version}`);
              return;
            }
          }
        } catch (httpErr) {
          console.warn("[updater] HTTP fallback failed:", httpErr);
        }
      }

      updateStatus = "";
      if (result.error) {
        console.warn("[updater] check returned error:", result.error);
      } else {
        console.log("[updater] app is up to date");
      }
    } catch (e) {
      updateStatus = "";
      console.error("[updater] autoCheckUpdate failed:", e);
    }
  }

  function isNewerVersion(latest, current) {
    const l = latest.split(".").map(Number);
    const c = current.split(".").map(Number);
    for (let i = 0; i < Math.max(l.length, c.length); i++) {
      if ((l[i] || 0) > (c[i] || 0)) return true;
      if ((l[i] || 0) < (c[i] || 0)) return false;
    }
    return false;
  }

  async function installUpdate() {
    if (updateRef && updateRef.download) {
      // Tauri plugin update path
      updateStatus = "downloading";
      updateProgress = 0;
      try {
        await updateRef.download(({ downloaded, total }) => {
          updateProgress = total > 0 ? Math.round((downloaded / total) * 100) : 0;
        });
        updateStatus = "ready";
        await updateRef.relaunch();
      } catch (e) {
        updateStatus = "error";
        updateError = e.toString();
        console.error("[updater] plugin install failed:", e);
        setTimeout(autoCheckUpdate, 30000);
      }
    } else {
      // Fallback: open releases page in browser
      try {
        const { open } = await import("@tauri-apps/plugin-shell");
        await open(`https://github.com/DomZaddy/TimeTrackerApp/releases/tag/v${updateVersion}`);
      } catch {
        window.open(`https://github.com/DomZaddy/TimeTrackerApp/releases/tag/v${updateVersion}`);
      }
    }
  }

  async function backfillOoo() {
    try {
      const result = await tauri.sheetsBackfillOoo();
      if (result.success && result.row_count > 0) {
        console.log(`[ooo] backfilled ${result.row_count} days`);
      }
    } catch (e) {
      // Silently fail — user may not be signed in yet
      console.warn("[ooo] backfill skipped:", e);
    }
  }

  function idleDayCheck() {
    const today = new Date().toDateString();
    if (today !== lastOooCheckDay && !isRunning) {
      lastOooCheckDay = today;
      backfillOoo();
    }
  }

  function handleGlobalClick(e) {
    if (showOooMenu && !e.target.closest(".ooo-wrapper")) {
      showOooMenu = false;
      showOooCustomInput = false;
      oooCustomReason = "";
    }
  }

  function submitCustomOoo() {
    const reason = oooCustomReason.trim();
    if (reason) {
      markTodayOoo(reason);
    }
    showOooCustomInput = false;
    oooCustomReason = "";
  }

  async function markTodayOoo(reason) {
    showOooMenu = false;
    showOooCustomInput = false;
    oooCustomReason = "";
    oooStatus = "marking";
    const today = new Date();
    const dateStr = `${String(today.getMonth() + 1).padStart(2, "0")}/${String(today.getDate()).padStart(2, "0")}/${today.getFullYear()}`;
    try {
      const result = await tauri.sheetsMarkOoo([dateStr], reason);
      if (result.success) {
        oooStatus = "done";
        setTimeout(() => (oooStatus = ""), 2000);
      } else {
        oooStatus = "";
        console.error("[ooo] mark failed:", result.error);
      }
    } catch (e) {
      oooStatus = "";
      console.error("[ooo] mark failed:", e);
    }
  }

  function toggleWidget() {
    widgetMode = !widgetMode;
    if (widgetMode) view = "tracker";
    tauri.setWidgetMode(widgetMode);
  }

  async function handleWelcomeComplete(name) {
    userName = name;
    await tauri.storeSet("user-name", name);
    showWelcome = false;
    const tutDone = await tauri.storeGet("tutorial-completed");
    if (!tutDone) showTutorial = true;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class={widgetMode ? "widget-mode" : ""} onclick={handleGlobalClick}>
  <TitleBar {widgetMode} onToggleWidget={toggleWidget} {updateStatus} {updateVersion} {updateProgress} onInstallUpdate={installUpdate} />

  {#if updateStatus === "available"}
    <button class="update-banner" onclick={installUpdate}>
      <span class="update-banner-icon">⬇</span>
      <span>Update v{updateVersion} is ready — click to {updateRef?.download ? "install" : "download"}</span>
    </button>
  {:else if updateStatus === "downloading"}
    <div class="update-banner update-banner--downloading">
      <div class="update-banner-progress" style="width: {updateProgress}%"></div>
      <span class="update-banner-text">Downloading update... {updateProgress}%</span>
    </div>
  {:else if updateStatus === "ready"}
    <div class="update-banner update-banner--ready">
      <span>Restarting...</span>
    </div>
  {:else if updateStatus === "error"}
    <button class="update-banner update-banner--error" onclick={autoCheckUpdate}>
      <span>Update failed — click to retry</span>
    </button>
  {/if}

  <ReminderToast />

  <div class="app-content">
    <div class="view-tabs">
      <button
        class="view-tab"
        class:active={view === "tracker"}
        onclick={() => (view = "tracker")}
      >
        {widgetMode ? "⏱" : "Tracker"}
      </button>
      <button
        class="view-tab"
        class:active={view === "earnings"}
        onclick={() => {
          if (widgetMode) toggleWidget();
          view = "earnings";
        }}
      >
        {widgetMode ? "💰" : "Earnings"}
      </button>
      <button
        class="view-tab"
        class:active={view === "settings"}
        onclick={() => {
          if (widgetMode) toggleWidget();
          view = "settings";
        }}
      >
        {widgetMode ? "⚙️" : "Settings"}
      </button>
    </div>

    <div class="tab-content">
      {#if view === "tracker"}
        <div class="tracker-layout">
          <div class="tracker-main">
            <ActivityRings
              {elapsed}
              {breakElapsed}
              {isRunning}
              {isOnBreak}
              dailyGoalHours={weekLimitH / 5}
              {weekPercent}
              compact={widgetMode}
            />

            {#if !widgetMode}
              <div class="task-section">
                <div class="task-label">Current Task</div>
                <input
                  class="task-input"
                  value={taskInput}
                  oninput={(e) => handleTaskChange(e.target.value)}
                  onblur={handleTaskBlur}
                  placeholder="What are you working on?"
                />
              </div>
            {/if}

            <Controls
              {isRunning}
              {isOnBreak}
              onClockIn={handleClockIn}
              onClockOut={handleClockOut}
              onToggleBreak={handleToggleBreakPicker}
              compact={widgetMode}
            />

            {#if !widgetMode && !isRunning}
              <div class="ooo-section">
                {#if oooStatus === "marking"}
                  <div class="ooo-badge marking">Marking OOO...</div>
                {:else if oooStatus === "done"}
                  <div class="ooo-badge done">Marked Out of Office</div>
                {:else}
                  <div class="ooo-wrapper">
                    <button class="ooo-btn" onclick={() => (showOooMenu = !showOooMenu)} title="Mark today as Out of Office">
                      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                        <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" opacity="0.5"/>
                        <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.2" stroke-dasharray="3 2"/>
                      </svg>
                      <span>Out of Office</span>
                    </button>
                    {#if showOooMenu}
                      <div class="ooo-menu">
                        <button onclick={() => markTodayOoo("")}>No Reason</button>
                        <button onclick={() => markTodayOoo("PTO")}>PTO</button>
                        <button onclick={() => markTodayOoo("Sick")}>Sick</button>
                        <button onclick={() => markTodayOoo("Holiday")}>Holiday</button>
                        <button onclick={() => markTodayOoo("Personal")}>Personal</button>
                        <div class="ooo-menu-divider"></div>
                        {#if showOooCustomInput}
                          <div class="ooo-custom-input">
                            <input
                              type="text"
                              placeholder="Custom reason..."
                              bind:value={oooCustomReason}
                              onkeydown={(e) => e.key === "Enter" && submitCustomOoo()}
                              autofocus
                            />
                            <button class="ooo-custom-submit" onclick={submitCustomOoo}>Go</button>
                          </div>
                        {:else}
                          <button onclick={() => (showOooCustomInput = true)}>Custom...</button>
                        {/if}
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/if}

            {#if !widgetMode}
              <NoteInput
                {notes}
                onAddNote={(text) => {
                  session.addNote(text);
                  gamification.onNoteAdded();
                }}
                {isRunning}
              />
            {/if}
          </div>
        </div>
      {:else if view === "earnings"}
        <MoneyJar
          monthlyPay={mPay}
          monthlyHours={mHours}
          weekLimitHours={weekLimitH}
          {isRunning}
          {elapsed}
        />
      {:else if view === "settings"}
        <Settings
          {userName}
          onSaveName={async (name) => {
            userName = name;
            await tauri.storeSet("user-name", name);
          }}
          onStartTutorial={async () => {
            await tauri.storeSet("tutorial-completed", false);
            showTutorial = true;
          }}
          onAuthSuccess={() => {
            if (isRunning) {
              const snap = timer.getSnapshot();
              tauri.sheetsStatusStart({
                task: get(session.currentTask) || taskInput,
                start_time: snap.startTime,
                elapsed,
              });
              weeklyHours.fetchWeekHours(true);
            }
          }}
          {updateStatus}
          {updateVersion}
          onCheckUpdate={autoCheckUpdate}
          onInstallUpdate={installUpdate}
        />
      {/if}
    </div>
  </div>

  {#if showTimePicker}
    <TimePicker
      mode={timePickerMode}
      onConfirm={handleTimeConfirm}
      onCancel={() => (showTimePicker = false)}
    />
  {/if}


  {#if showSummary}
    <SessionSummary
      blocks={summaryBlocks}
      onPush={handlePush}
      onDiscard={handleDiscard}
      {isPushing}
    />
  {/if}

  {#if recoveryData}
    <RecoveryPrompt
      data={recoveryData}
      onResume={handleResumeSession}
      onDiscard={handleDiscardRecovery}
    />
  {/if}

  {#if pushStatus}
    <div
      class="push-status-toast"
      style="background: {pushStatus.includes('Error')
        ? 'var(--red)'
        : 'var(--green)'}"
    >
      {pushStatus.includes("Error") ? "⚠️" : "✓"}
      {pushStatus}
    </div>
  {/if}

  <WeeklyProgressBar
    {totalWeekMs}
    {weekPercent}
    {isOvertime}
    {overtimeMs}
    overtimeMode={overtimeModeVal}
    weekLimitHours={weekLimitH}
    {isRunning}
    {elapsed}
  />

  <Confetti trigger={confettiTrigger} />
  <AchievementToast queue={toastQueue} onDismiss={gamification.dismissToast} />

  {#if showTutorial}
    <TutorialOverlay {userName} onComplete={() => (showTutorial = false)} onSwitchTab={(tab) => (view = tab)} />
  {/if}

  {#if showWelcome && !recoveryData}
    <WelcomeModal onComplete={handleWelcomeComplete} />
  {/if}
</div>
