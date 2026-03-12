export function parseSessionToBlocks(session) {
  const { startTime, endTime, breaks, notes, tasks } = session;
  const start = new Date(startTime);
  const end = new Date(endTime);

  const BLOCK_MS = 2 * 60 * 60 * 1000;
  const blocks = [];
  let cursor = new Date(start);

  // First pass: build blocks without notes
  while (cursor < end) {
    let blockEnd = new Date(Math.min(cursor.getTime() + BLOCK_MS, end.getTime()));

    let blockBreakMs = 0;
    for (const b of breaks) {
      const bStart = new Date(b.start);
      const bEnd = b.end ? new Date(b.end) : end;
      const overlapStart = Math.max(cursor.getTime(), bStart.getTime());
      const overlapEnd = Math.min(blockEnd.getTime(), bEnd.getTime());
      if (overlapStart < overlapEnd) {
        blockBreakMs += overlapEnd - overlapStart;
      }
    }

    const blockTasks = getTasksForBlock(tasks, cursor.getTime(), blockEnd.getTime());
    const breakHours = Math.round((blockBreakMs / (1000 * 60 * 60)) * 100) / 100;

    blocks.push({
      cursor: new Date(cursor),
      blockEnd,
      blockTasks,
      breakHours,
    });

    cursor = blockEnd;
  }

  // Distribute notes evenly across blocks (round-robin, ignore timestamps)
  const allNotes = (notes || []).filter((n) => n.text);
  const blockNoteMap = blocks.map(() => []);
  for (let i = 0; i < allNotes.length; i++) {
    blockNoteMap[i % blocks.length].push(allNotes[i]);
  }

  // Second pass: format blocks with distributed notes
  return blocks.map((block, idx) => {
    const blockNotes = blockNoteMap[idx];
    const taskField = formatTaskField(block.blockTasks, blockNotes);
    const noteTexts = blockNotes.map((n) => n.text).filter(Boolean).join("; ");

    return {
      date: formatDate(block.cursor),
      task: taskField,
      checkIn: formatTime(block.cursor),
      checkOut: formatTime(block.blockEnd),
      breakHours: block.breakHours,
      notes: noteTexts,
    };
  });
}

function getTasksForBlock(tasks, blockStart, blockEnd) {
  if (!tasks || tasks.length === 0) return [{ name: "" }];

  const result = [];
  for (let i = 0; i < tasks.length; i++) {
    const taskStart = tasks[i].timestamp;
    const taskEnd = i + 1 < tasks.length ? tasks[i + 1].timestamp : Infinity;
    if (taskStart < blockEnd && taskEnd > blockStart) {
      result.push(tasks[i]);
    }
  }

  if (result.length === 0 && tasks.length > 0) {
    result.push(tasks[0]);
  }

  return result;
}

function formatTaskField(blockTasks, blockNotes) {
  if (blockTasks.length === 1) {
    const taskName = blockTasks[0].name || "";
    const noteTexts = blockNotes.map((n) => n.text).filter(Boolean);
    if (noteTexts.length === 0) return taskName;
    if (!taskName) return noteTexts.join("; ");
    return `${taskName}: ${noteTexts.join("; ")}`;
  }

  const parts = [];
  for (let i = 0; i < blockTasks.length; i++) {
    const task = blockTasks[i];
    const taskName = task.name || "";
    // With distributed notes, just attach all block notes to the first task
    if (i === 0 && blockNotes.length > 0) {
      const noteTexts = blockNotes.map((n) => n.text).filter(Boolean);
      parts.push(taskName ? `${taskName}: ${noteTexts.join("; ")}` : noteTexts.join("; "));
    } else if (taskName) {
      parts.push(taskName);
    }
  }

  return parts.join(" | ");
}

function formatDate(date) {
  const d = new Date(date);
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${month}/${day}/${d.getFullYear()}`;
}

function formatTime(date) {
  const d = new Date(date);
  let hours = d.getHours();
  const minutes = String(d.getMinutes()).padStart(2, "0");
  const ampm = hours >= 12 ? "PM" : "AM";
  hours = hours % 12 || 12;
  return `${hours}:${minutes} ${ampm}`;
}
