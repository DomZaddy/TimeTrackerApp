import { get } from "svelte/store";
import { storeGet, storeSet } from "../tauri.js";
import { showReminder } from "./reminderToast.js";
import { elapsed, isRunning, isOnBreak } from "./timer.js";

let nudgeSettings = {
  dailyCap: { enabled: true, hours: 10 },
  breakReminder: { enabled: true, hours: 2 },
  forgotClockOut: { enabled: true, hour: 19 }, // 7 PM
};

let nudgeFired = {
  dailyCap: false,
  breakReminder: false,
  forgotClockOut: false,
};

let checkInterval = null;
let lastBreakEnd = Date.now();

export async function loadNudgeSettings() {
  const dc = await storeGet("nudge-daily-cap");
  if (dc) nudgeSettings.dailyCap = dc;
  const br = await storeGet("nudge-break-reminder");
  if (br) nudgeSettings.breakReminder = br;
  const fc = await storeGet("nudge-forgot-clockout");
  if (fc) nudgeSettings.forgotClockOut = fc;
}

export function startNudgeChecks() {
  if (checkInterval) clearInterval(checkInterval);
  nudgeFired = { dailyCap: false, breakReminder: false, forgotClockOut: false };
  lastBreakEnd = Date.now();

  checkInterval = setInterval(() => {
    if (!get(isRunning)) return;

    const elapsedMs = get(elapsed);
    const onBreak = get(isOnBreak);

    // Daily cap
    if (
      nudgeSettings.dailyCap.enabled &&
      !nudgeFired.dailyCap &&
      elapsedMs >= nudgeSettings.dailyCap.hours * 3600000
    ) {
      nudgeFired.dailyCap = true;
      showReminder(`You've been working ${nudgeSettings.dailyCap.hours}+ hours today. Consider wrapping up.`);
    }

    // Break reminder
    if (
      nudgeSettings.breakReminder.enabled &&
      !nudgeFired.breakReminder &&
      !onBreak
    ) {
      const sinceBreak = Date.now() - lastBreakEnd;
      if (sinceBreak >= nudgeSettings.breakReminder.hours * 3600000) {
        nudgeFired.breakReminder = true;
        showReminder(`You've been working ${nudgeSettings.breakReminder.hours}+ hours without a break. Take a breather!`);
      }
    }

    // Forgot clock-out
    if (
      nudgeSettings.forgotClockOut.enabled &&
      !nudgeFired.forgotClockOut
    ) {
      const hour = new Date().getHours();
      if (hour >= nudgeSettings.forgotClockOut.hour) {
        nudgeFired.forgotClockOut = true;
        showReminder("It's getting late. Did you forget to clock out?");
      }
    }
  }, 60000);
}

export function stopNudgeChecks() {
  if (checkInterval) {
    clearInterval(checkInterval);
    checkInterval = null;
  }
}

export function onBreakEnd() {
  lastBreakEnd = Date.now();
  nudgeFired.breakReminder = false;
}

export async function saveNudgeSettings(key, value) {
  nudgeSettings[key] = value;
  const storeKey = {
    dailyCap: "nudge-daily-cap",
    breakReminder: "nudge-break-reminder",
    forgotClockOut: "nudge-forgot-clockout",
  }[key];
  if (storeKey) await storeSet(storeKey, value);
}

export function getNudgeSettings() {
  return { ...nudgeSettings };
}
