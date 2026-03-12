import { writable, derived, get } from "svelte/store";

// Internal state
let startTime = null;
let breakStartTime = null;
let totalBreakMs = 0;
let intervalId = null;

export const elapsed = writable(0);
export const breakElapsed = writable(0);
export const isRunning = writable(false);
export const isOnBreak = writable(false);

function tick() {
  if (!startTime) return;
  const now = Date.now();
  const totalElapsed = now - startTime;
  const currentBreak = breakStartTime ? now - breakStartTime : 0;
  elapsed.set(totalElapsed - totalBreakMs - currentBreak);
  breakElapsed.set(totalBreakMs + currentBreak);
}

export function start(customStartTime) {
  const startMs = customStartTime ? new Date(customStartTime).getTime() : Date.now();
  startTime = startMs;
  totalBreakMs = 0;
  breakStartTime = null;
  isRunning.set(true);
  isOnBreak.set(false);
  elapsed.set(Date.now() - startMs);
  breakElapsed.set(0);
  if (intervalId) clearInterval(intervalId);
  intervalId = setInterval(tick, 1000);
}

export function stop(customEndTime) {
  if (intervalId) clearInterval(intervalId);
  intervalId = null;
  const endTime = customEndTime || Date.now();
  if (breakStartTime) {
    totalBreakMs += endTime - breakStartTime;
    breakStartTime = null;
  }
  const finalElapsed = endTime - startTime - totalBreakMs;
  elapsed.set(finalElapsed);
  breakElapsed.set(totalBreakMs);
  const result = {
    startTime,
    endTime,
    totalBreakMs,
  };
  isRunning.set(false);
  isOnBreak.set(false);
  return result;
}

export function toggleBreak() {
  if (get(isOnBreak)) {
    totalBreakMs += Date.now() - breakStartTime;
    breakStartTime = null;
    isOnBreak.set(false);
  } else {
    breakStartTime = Date.now();
    isOnBreak.set(true);
  }
}

export function getSnapshot() {
  return {
    startTime,
    totalBreakMs,
    breakStartTime,
    isOnBreak: get(isOnBreak),
    isRunning: get(isRunning),
  };
}

export function restore({ startTime: st, totalBreakMs: tbm, breakStartTime: bst, isOnBreak: wasOnBreak }) {
  startTime = st;
  totalBreakMs = tbm || 0;
  breakStartTime = wasOnBreak ? bst : null;
  isRunning.set(true);
  isOnBreak.set(!!wasOnBreak);
  const now = Date.now();
  const currentBreak = breakStartTime ? now - breakStartTime : 0;
  elapsed.set(now - startTime - totalBreakMs - currentBreak);
  breakElapsed.set(totalBreakMs + currentBreak);
  if (intervalId) clearInterval(intervalId);
  intervalId = setInterval(tick, 1000);
}

export function addBreakMs(ms) {
  totalBreakMs += ms;
  tick();
}

export function removeBreakMs(ms) {
  totalBreakMs = Math.max(0, totalBreakMs - ms);
  tick();
}

export function getStartTime() {
  return startTime;
}
