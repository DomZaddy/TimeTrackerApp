import { writable, derived, get } from "svelte/store";
import { storeGet, storeSet, sheetsFetchWeekHours } from "../tauri.js";

const CACHE_TTL = 15 * 60 * 1000; // 15 minutes

export const totalWeekMs = writable(0);
export const monthlyPay = writable(0);
export const monthlyHours = writable(0);
export const weekLimitHours = writable(40);
export const overtimeMode = writable(false);

export const weekPercent = derived(
  [totalWeekMs, weekLimitHours],
  ([$ms, $limit]) => $ms / ($limit * 3600000)
);

export const isOvertime = derived(weekPercent, ($p) => $p > 1.0);

export const overtimeMs = derived(
  [totalWeekMs, weekLimitHours],
  ([$ms, $limit]) => Math.max(0, $ms - $limit * 3600000)
);

export async function loadWeeklySettings() {
  const limit = await storeGet("weekly-hour-limit");
  if (limit) weekLimitHours.set(limit);
  const ot = await storeGet("overtime-mode");
  if (ot !== null && ot !== undefined) overtimeMode.set(ot);
}

export async function fetchWeekHours(force = false) {
  // Check cache
  if (!force) {
    const cache = await storeGet("weekly-hours-cache");
    if (cache && Date.now() - cache.fetchedAt < CACHE_TTL) {
      totalWeekMs.set(cache.weekMs || 0);
      monthlyPay.set(cache.monthlyPay || 0);
      monthlyHours.set(cache.monthlyHours || 0);
      return;
    }
  }

  const result = await sheetsFetchWeekHours();
  if (result.success) {
    totalWeekMs.set(result.week_ms || 0);
    monthlyPay.set(result.monthly_pay || 0);
    monthlyHours.set(result.monthly_hours || 0);

    await storeSet("weekly-hours-cache", {
      weekMs: result.week_ms,
      monthlyPay: result.monthly_pay,
      monthlyHours: result.monthly_hours,
      fetchedAt: Date.now(),
    });
  }
}

export async function setWeekLimit(hours) {
  weekLimitHours.set(hours);
  await storeSet("weekly-hour-limit", hours);
}

export async function setOvertimeMode(enabled) {
  overtimeMode.set(enabled);
  await storeSet("overtime-mode", enabled);
}
