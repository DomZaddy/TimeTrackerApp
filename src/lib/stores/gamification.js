import { writable, derived, get } from "svelte/store";
import { storeGet, storeSet } from "../tauri.js";

const XP_PER_LEVEL = 500;
const LEVEL_NAMES = [
  "Intern", "Apprentice", "Journeyman", "Specialist", "Expert",
  "Veteran", "Master", "Grandmaster", "Legend", "Mythic",
];

const ACHIEVEMENTS = {
  first_clockin: { name: "First Steps", desc: "Clock in for the first time", icon: "🎯" },
  early_bird: { name: "Early Bird", desc: "Clock in before 7 AM", icon: "🌅" },
  night_owl: { name: "Night Owl", desc: "Clock in after 10 PM", icon: "🦉" },
  note_taker: { name: "Scribe", desc: "Add 10+ notes in a session", icon: "📝" },
  marathon: { name: "Marathon", desc: "Work 8+ hours in one session", icon: "🏃" },
  perfect_week: { name: "Perfect Week", desc: "Hit your weekly hour goal", icon: "⭐" },
  streak_3: { name: "3-Day Streak", desc: "Push sessions 3 days in a row", icon: "🔥" },
  streak_7: { name: "Weekly Warrior", desc: "7-day push streak", icon: "⚔️" },
  streak_14: { name: "Fortnight Force", desc: "14-day push streak", icon: "💪" },
  streak_30: { name: "Monthly Monster", desc: "30-day push streak", icon: "👹" },
  month_crusher: { name: "Month Crusher", desc: "Log 160+ hours in a month", icon: "💎" },
  pay_day: { name: "Pay Day", desc: "Track your first payment", icon: "💰" },
};

// State
export const gamificationData = writable({
  xp: 0,
  level: 0,
  streaks: { daily: 0, lastPushDate: null, weeklyCount: 0 },
  achievements: [],
  monthStats: {},
});

export const soundsEnabled = writable(true);
export const toastQueue = writable([]);
export const confettiTrigger = writable(0);

// Derived
export const levelName = derived(gamificationData, ($d) =>
  LEVEL_NAMES[Math.min($d.level, LEVEL_NAMES.length - 1)]
);

export const xpProgress = derived(gamificationData, ($d) =>
  ($d.xp % XP_PER_LEVEL) / XP_PER_LEVEL
);

// Init
export async function loadGamification() {
  const data = await storeGet("gamification-data");
  if (data) gamificationData.set(data);
  const sounds = await storeGet("gamification-sounds");
  if (sounds !== null && sounds !== undefined) soundsEnabled.set(sounds);
}

async function save() {
  await storeSet("gamification-data", get(gamificationData));
}

function addXp(amount) {
  gamificationData.update((d) => {
    d.xp += amount;
    const newLevel = Math.floor(d.xp / XP_PER_LEVEL);
    if (newLevel > d.level) {
      d.level = newLevel;
      showToast({
        type: "levelup",
        title: `Level ${d.level}!`,
        desc: LEVEL_NAMES[Math.min(d.level, LEVEL_NAMES.length - 1)],
        icon: "⬆️",
      });
      triggerConfetti();
    }
    return d;
  });
  save();
}

function unlock(key) {
  const d = get(gamificationData);
  if (d.achievements.includes(key)) return;
  const ach = ACHIEVEMENTS[key];
  if (!ach) return;

  gamificationData.update((d) => {
    d.achievements = [...d.achievements, key];
    return d;
  });

  showToast({
    type: "achievement",
    title: ach.name,
    desc: ach.desc,
    icon: ach.icon,
  });
  triggerConfetti();
  addXp(100);
}

function showToast(toast) {
  toastQueue.update((q) => [...q, { ...toast, id: Date.now() }]);
}

function triggerConfetti() {
  confettiTrigger.update((n) => n + 1);
}

export function dismissToast() {
  toastQueue.update((q) => q.slice(1));
}

// Events
export function onClockIn(startTime) {
  unlock("first_clockin");
  const hour = new Date(startTime).getHours();
  if (hour < 7) unlock("early_bird");
  if (hour >= 22) unlock("night_owl");
  addXp(25);
}

export function onNoteAdded() {
  addXp(5);
}

export function onSessionPush({ elapsedMs, notesCount, payAccrued, monthTab }) {
  addXp(50);

  // Streak tracking
  const today = new Date().toISOString().slice(0, 10);
  gamificationData.update((d) => {
    const yesterday = new Date(Date.now() - 86400000).toISOString().slice(0, 10);
    if (d.streaks.lastPushDate === yesterday) {
      d.streaks.daily += 1;
    } else if (d.streaks.lastPushDate !== today) {
      d.streaks.daily = 1;
    }
    d.streaks.lastPushDate = today;
    return d;
  });

  const d = get(gamificationData);

  // Streak achievements
  if (d.streaks.daily >= 3) unlock("streak_3");
  if (d.streaks.daily >= 7) unlock("streak_7");
  if (d.streaks.daily >= 14) unlock("streak_14");
  if (d.streaks.daily >= 30) unlock("streak_30");

  // Session achievements
  if (elapsedMs >= 8 * 3600000) unlock("marathon");
  if (notesCount >= 10) unlock("note_taker");
  if (payAccrued > 0) unlock("pay_day");

  save();
}

export function onWeeklyLimitReached() {
  unlock("perfect_week");
}

export function checkTimeAchievements() {
  // Placeholder for periodic checks
}

export async function toggleSounds() {
  soundsEnabled.update((v) => !v);
  await storeSet("gamification-sounds", get(soundsEnabled));
}

export async function resetGamification() {
  gamificationData.set({
    xp: 0,
    level: 0,
    streaks: { daily: 0, lastPushDate: null, weeklyCount: 0 },
    achievements: [],
    monthStats: {},
  });
  await save();
}

export { ACHIEVEMENTS, LEVEL_NAMES };
