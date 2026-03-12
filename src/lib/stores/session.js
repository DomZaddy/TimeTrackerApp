import { writable, derived, get } from "svelte/store";

export const session = writable(null);
export const notes = writable([]);
export const breaks = writable([]);
export const tasks = writable([]);

export const currentTask = derived(tasks, ($tasks) => {
  if ($tasks.length === 0) return "";
  return $tasks[$tasks.length - 1].name;
});

export function startSession(startTime) {
  session.set({ startTime: startTime || Date.now(), endTime: null });
  notes.set([]);
  breaks.set([]);
  tasks.set([]);
}

export function endSession(endTime) {
  session.update((s) => (s ? { ...s, endTime: endTime || Date.now() } : null));
}

export function addNote(text) {
  notes.update((prev) => [...prev, { timestamp: Date.now(), text }]);
}

export function commitTask(name) {
  const trimmed = (name || "").trim();
  tasks.update((prev) => {
    const last = prev.length > 0 ? prev[prev.length - 1].name : "";
    if (trimmed === last) return prev;
    return [...prev, { timestamp: Date.now(), name: trimmed }];
  });
}

export function startBreak() {
  breaks.update((prev) => [...prev, { start: Date.now(), end: null }]);
}

export function endBreak() {
  breaks.update((prev) => {
    const updated = [...prev];
    const last = updated[updated.length - 1];
    if (last && !last.end) {
      updated[updated.length - 1] = { ...last, end: Date.now() };
    }
    return updated;
  });
}

export function addManualBreak(startMs, endMs) {
  breaks.update((prev) => [...prev, { start: startMs, end: endMs }]);
}

export function removeBreak(index) {
  breaks.update((prev) => prev.filter((_, i) => i !== index));
}

export function getSessionData() {
  const s = get(session);
  if (!s) return null;
  return {
    ...s,
    notes: get(notes),
    breaks: get(breaks),
    tasks: get(tasks),
  };
}

export function clearSession() {
  session.set(null);
  notes.set([]);
  breaks.set([]);
  tasks.set([]);
}

export function restoreSession({
  startTime,
  notes: savedNotes,
  breaks: savedBreaks,
  tasks: savedTasks,
}) {
  session.set({ startTime, endTime: null });
  notes.set(savedNotes || []);
  breaks.set(savedBreaks || []);
  tasks.set(savedTasks || []);
}
