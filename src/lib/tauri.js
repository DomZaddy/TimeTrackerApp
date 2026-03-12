import { invoke } from "@tauri-apps/api/core";
import { Store } from "@tauri-apps/plugin-store";

let store = null;

async function getStore() {
  if (!store) {
    store = await Store.load("timetracker-store.json");
  }
  return store;
}

// Storage
export async function storeGet(key) {
  const s = await getStore();
  return await s.get(key);
}

export async function storeSet(key, value) {
  const s = await getStore();
  await s.set(key, value);
  await s.save();
}

export async function storeDelete(key) {
  const s = await getStore();
  await s.delete(key);
  await s.save();
}

// Google Sheets
export async function sheetsAuth() {
  return await invoke("sheets_auth");
}

export async function sheetsSignOut() {
  return await invoke("sheets_sign_out");
}

export async function sheetsPush(rows) {
  return await invoke("sheets_push", { rows });
}

export async function sheetsStatusStart(data) {
  return await invoke("sheets_status_start", { data });
}

export async function sheetsStatusUpdate(data) {
  return await invoke("sheets_status_update", { data });
}

export async function sheetsStatusClear() {
  return await invoke("sheets_status_clear");
}

export async function sheetsFetchWeekHours() {
  return await invoke("sheets_fetch_week_hours");
}

// Out of Office
export async function sheetsMarkOoo(dates, reason = "") {
  return await invoke("sheets_mark_ooo", { request: { dates, reason } });
}

export async function sheetsBackfillOoo() {
  return await invoke("sheets_backfill_ooo");
}

// Notifications
export async function sendNudge(data) {
  return await invoke("send_nudge", { data });
}

export async function sendReminder() {
  return await invoke("send_reminder");
}

// Window management
export async function minimizeWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().minimize();
}

export async function closeWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().hide();
}

// Auto-launch
export async function getAutoLaunch() {
  return await invoke("get_auto_launch");
}

export async function setAutoLaunch(enabled) {
  return await invoke("set_auto_launch", { enabled });
}

export async function setWidgetMode(enabled) {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const win = getCurrentWindow();
  if (enabled) {
    await win.setAlwaysOnTop(true);
    await win.setSize(new (await import("@tauri-apps/api/dpi")).LogicalSize(400, 400));
    await win.setResizable(false);
  } else {
    await win.setAlwaysOnTop(false);
    await win.setSize(new (await import("@tauri-apps/api/dpi")).LogicalSize(420, 700));
    await win.setResizable(true);
  }
}

// Taskbar badge/progress
let _cachedWindow = null;
async function getWindow() {
  if (!_cachedWindow) {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    _cachedWindow = getCurrentWindow();
  }
  return _cachedWindow;
}

/**
 * Set taskbar progress bar (Windows).
 * @param {number} progress 0-100, or -1 to clear
 * @param {'normal'|'paused'|'error'} status
 */
export async function setTaskbarProgress(progress, status = 'normal') {
  try {
    const win = await getWindow();
    if (progress < 0) {
      await win.setProgressBar({ status: 'none', progress: 0 });
    } else {
      const statusMap = { normal: 'Normal', paused: 'Paused', error: 'Error' };
      await win.setProgressBar({
        status: statusMap[status] || 'Normal',
        progress: Math.min(Math.max(Math.round(progress), 0), 100),
      });
    }
  } catch {}
}

/**
 * Set overlay icon on taskbar (Windows badge).
 * Pass a Tauri Image or undefined to clear.
 */
export async function setOverlayIcon(icon) {
  try {
    const win = await getWindow();
    await win.setOverlayIcon(icon);
  } catch {}
}

/**
 * Generate a colored circle badge as a Tauri Image for overlay icon.
 * Uses an offscreen canvas to draw a 16x16 colored dot.
 */
export async function makeBadgeIcon(color) {
  const { Image } = await import("@tauri-apps/api/image");
  const size = 16;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d');

  // Colored circle
  ctx.beginPath();
  ctx.arc(size / 2, size / 2, size / 2 - 1, 0, Math.PI * 2);
  ctx.fillStyle = color;
  ctx.fill();

  // White border for visibility
  ctx.lineWidth = 1.5;
  ctx.strokeStyle = 'white';
  ctx.stroke();

  // Convert to Tauri Image (RGBA + dimensions)
  const imageData = ctx.getImageData(0, 0, size, size);
  return await Image.new(imageData.data, size, size);
}

/** Clear the overlay icon and progress bar */
export async function clearTaskbarBadge() {
  await setOverlayIcon(undefined);
  await setTaskbarProgress(-1);
}

// Updater
export async function checkForUpdate() {
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (update) {
      return {
        available: true,
        version: update.version,
        body: update.body,
        date: update.date,
        download: async (onProgress) => {
          let downloaded = 0;
          let total = 0;
          await update.downloadAndInstall((event) => {
            if (event.event === "Started") {
              total = event.data.contentLength || 0;
            } else if (event.event === "Progress") {
              downloaded += event.data.chunkLength;
              if (onProgress) onProgress({ downloaded, total });
            } else if (event.event === "Finished") {
              if (onProgress) onProgress({ downloaded: total, total });
            }
          });
        },
        relaunch: async () => {
          const { relaunch } = await import("@tauri-apps/plugin-process");
          await relaunch();
        },
      };
    }
    return { available: false };
  } catch (e) {
    console.warn("[updater] check failed:", e);
    return { available: false, error: e.toString() };
  }
}
