// Web Audio API synthesized alarm tones — no audio files needed
let audioCtx = null;

function getCtx() {
  if (!audioCtx) audioCtx = new (window.AudioContext || window.webkitAudioContext)();
  if (audioCtx.state === 'suspended') audioCtx.resume();
  return audioCtx;
}

function playNote(freq, duration, type = 'sine', gain = 0.3, delay = 0) {
  const ctx = getCtx();
  const t = ctx.currentTime + delay;
  const osc = ctx.createOscillator();
  const vol = ctx.createGain();
  osc.type = type;
  osc.frequency.value = freq;
  vol.gain.setValueAtTime(0.001, t);
  vol.gain.linearRampToValueAtTime(gain, t + 0.01);
  vol.gain.exponentialRampToValueAtTime(0.001, t + duration);
  osc.connect(vol);
  vol.connect(ctx.destination);
  osc.start(t);
  osc.stop(t + duration);
}

export const TONES = {
  chime: {
    label: 'Chime',
    description: 'Pleasant ascending notes',
    play() {
      playNote(523, 0.4, 'sine', 0.25, 0);
      playNote(659, 0.4, 'sine', 0.25, 0.15);
      playNote(784, 0.6, 'sine', 0.3, 0.3);
    },
  },
  gong: {
    label: 'Gong',
    description: 'Deep resonant strike',
    play() {
      playNote(260, 2.5, 'sine', 0.35, 0);
      playNote(520, 1.8, 'sine', 0.2, 0);
      playNote(780, 1.2, 'sine', 0.1, 0);
      playNote(390, 1.5, 'triangle', 0.12, 0);
    },
  },
  alarm: {
    label: 'Alarm Clock',
    description: 'Classic beep-beep pattern',
    play() {
      for (let i = 0; i < 4; i++) {
        playNote(880, 0.12, 'square', 0.15, i * 0.25);
        playNote(700, 0.12, 'square', 0.15, i * 0.25 + 0.12);
      }
    },
  },
  timer: {
    label: 'Timer Bell',
    description: 'Bright ding-ding',
    play() {
      playNote(1200, 0.3, 'sine', 0.2, 0);
      playNote(1200, 0.3, 'sine', 0.2, 0.4);
      playNote(1500, 0.5, 'sine', 0.25, 0.8);
    },
  },
  crystal: {
    label: 'Crystal',
    description: 'FF victory fanfare inspired',
    play() {
      const s = 0.12;
      playNote(466, s, 'sine', 0.2, 0);
      playNote(466, s, 'sine', 0.2, s);
      playNote(466, s, 'sine', 0.2, s * 2);
      playNote(466, 0.5, 'sine', 0.25, s * 3);
      playNote(415, 0.22, 'sine', 0.2, 0.9);
      playNote(466, 0.22, 'sine', 0.2, 1.12);
      playNote(523, 0.22, 'sine', 0.22, 1.4);
      playNote(554, 0.22, 'sine', 0.22, 1.62);
      playNote(622, 0.32, 'sine', 0.24, 1.84);
      playNote(698, 0.32, 'sine', 0.24, 2.1);
      playNote(932, 1.2, 'sine', 0.3, 2.4);
      playNote(622, 1.0, 'sine', 0.12, 2.4);
      playNote(466, 0.9, 'sine', 0.1, 2.4);
      playNote(1865, 0.6, 'sine', 0.06, 2.5);
      playNote(1397, 0.5, 'sine', 0.05, 2.6);
    },
  },
  encounter: {
    label: 'Encounter',
    description: 'Pokemon center heal jingle',
    play() {
      const n = 0.13;
      playNote(392, n, 'square', 0.12, 0);
      playNote(494, n, 'square', 0.12, n);
      playNote(587, n, 'square', 0.12, n * 2);
      playNote(784, n * 1.5, 'square', 0.14, n * 3.2);
      playNote(740, n, 'square', 0.12, n * 4.8);
      playNote(784, n * 1.5, 'square', 0.14, n * 5.8);
      playNote(988, n, 'square', 0.13, n * 8);
      playNote(880, n, 'square', 0.12, n * 9);
      playNote(784, n, 'square', 0.12, n * 10);
      playNote(880, n, 'square', 0.13, n * 11.5);
      playNote(988, n, 'square', 0.14, n * 12.5);
      playNote(1175, n * 1.3, 'square', 0.14, n * 13.5);
      playNote(1568, 0.8, 'square', 0.13, n * 15);
      playNote(1175, 0.7, 'square', 0.08, n * 15);
      playNote(784, 0.7, 'sine', 0.15, n * 15);
      playNote(988, 0.6, 'sine', 0.1, n * 15);
    },
  },
  quest: {
    label: 'Quest',
    description: 'Zelda item fanfare inspired',
    play() {
      const n = 0.15;
      playNote(330, n * 1.5, 'triangle', 0.22, 0);
      playNote(392, n * 1.5, 'triangle', 0.22, n * 1.8);
      playNote(440, n * 1.5, 'triangle', 0.22, n * 3.6);
      playNote(523, n, 'triangle', 0.2, n * 6);
      playNote(587, n, 'triangle', 0.2, n * 7);
      playNote(659, n, 'triangle', 0.22, n * 8);
      playNote(784, n, 'triangle', 0.22, n * 9);
      playNote(698, n * 1.2, 'triangle', 0.2, n * 10.5);
      playNote(784, n * 1.2, 'triangle', 0.22, n * 12);
      playNote(880, n * 1.2, 'triangle', 0.24, n * 13.5);
      playNote(1047, 1.2, 'triangle', 0.28, n * 15);
      playNote(784, 1.0, 'sine', 0.14, n * 15);
      playNote(523, 0.9, 'sine', 0.1, n * 15);
      playNote(2093, 0.4, 'sine', 0.04, n * 15.5);
      playNote(1568, 0.5, 'sine', 0.05, n * 16);
    },
  },
  none: {
    label: 'Silent',
    description: 'No sound',
    play() {},
  },
};

export const TONE_KEYS = Object.keys(TONES);

export function playTone(key) {
  const tone = TONES[key];
  if (tone) tone.play();
}
