// Micro-feedback sounds for clock-in, clock-out, milestones
// Short, satisfying Web Audio API tones (< 500ms)

let audioCtx = null;
function getCtx() {
  if (!audioCtx) audioCtx = new (window.AudioContext || window.webkitAudioContext)();
  return audioCtx;
}

export function playClockIn() {
  const ctx = getCtx();
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.type = 'sine';
  osc.frequency.setValueAtTime(523, ctx.currentTime); // C5
  osc.frequency.setValueAtTime(659, ctx.currentTime + 0.08); // E5
  osc.frequency.setValueAtTime(784, ctx.currentTime + 0.16); // G5
  gain.gain.setValueAtTime(0.15, ctx.currentTime);
  gain.gain.exponentialDecayToValueAtTime?.(0.01, ctx.currentTime + 0.35) ||
    gain.gain.setTargetAtTime(0.001, ctx.currentTime + 0.2, 0.05);
  osc.start(ctx.currentTime);
  osc.stop(ctx.currentTime + 0.35);
}

export function playClockOut() {
  const ctx = getCtx();
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.type = 'sine';
  osc.frequency.setValueAtTime(784, ctx.currentTime); // G5
  osc.frequency.setValueAtTime(659, ctx.currentTime + 0.1); // E5
  osc.frequency.setValueAtTime(523, ctx.currentTime + 0.2); // C5
  gain.gain.setValueAtTime(0.15, ctx.currentTime);
  gain.gain.setTargetAtTime(0.001, ctx.currentTime + 0.25, 0.05);
  osc.start(ctx.currentTime);
  osc.stop(ctx.currentTime + 0.4);
}

export function playMilestone() {
  const ctx = getCtx();
  const notes = [523, 659, 784, 1047]; // C5, E5, G5, C6
  notes.forEach((freq, i) => {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = 'sine';
    osc.frequency.setValueAtTime(freq, ctx.currentTime + i * 0.1);
    gain.gain.setValueAtTime(0.12, ctx.currentTime + i * 0.1);
    gain.gain.setTargetAtTime(0.001, ctx.currentTime + i * 0.1 + 0.15, 0.04);
    osc.start(ctx.currentTime + i * 0.1);
    osc.stop(ctx.currentTime + i * 0.1 + 0.3);
  });
}

export function playBreakStart() {
  const ctx = getCtx();
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.type = 'triangle';
  osc.frequency.setValueAtTime(440, ctx.currentTime); // A4
  osc.frequency.setValueAtTime(392, ctx.currentTime + 0.12); // G4
  gain.gain.setValueAtTime(0.12, ctx.currentTime);
  gain.gain.setTargetAtTime(0.001, ctx.currentTime + 0.2, 0.04);
  osc.start(ctx.currentTime);
  osc.stop(ctx.currentTime + 0.3);
}
