<script>
  import { getCurrentTimeString } from "../utils/formatTime.js";

  let { notes = [], onAddNote, isRunning = false } = $props();
  let noteText = $state("");

  function handleAdd() {
    const trimmed = noteText.trim();
    if (!trimmed) return;
    onAddNote(trimmed);
    noteText = "";
  }

  function handleKeydown(e) {
    if (e.key === "Enter" && noteText.trim()) handleAdd();
  }
</script>

<div class="notes-section">
  <div class="notes-header">
    <span class="notes-title">Session Notes</span>
    <span class="notes-count">{notes.length} note{notes.length !== 1 ? "s" : ""}</span>
  </div>

  {#if isRunning}
    <div class="note-input-row">
      <input
        class="note-input"
        bind:value={noteText}
        onkeydown={handleKeydown}
        placeholder="Add a note..."
      />
      <button
        class="note-add-btn"
        onclick={handleAdd}
        disabled={!noteText.trim()}
      >
        Add
      </button>
    </div>
  {/if}

  <div class="notes-list">
    {#each [...notes].reverse() as note}
      <div class="note-item">
        <span class="note-text">{note.text}</span>
      </div>
    {/each}
  </div>
</div>
