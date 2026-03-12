import { writable } from 'svelte/store';

// Set to a string message to show the reminder toast, null to hide
export const reminderToast = writable(null);

export function showReminder(message = "Time to log your progress!") {
  reminderToast.set(message);
}
