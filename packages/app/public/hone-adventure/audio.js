// HONE Adventure — Pure WebAudio DSP Synthesizer & 130 BPM 4-Stage Cyber-Quant Soundtrack

const SR = 44100;
const TAU = Math.PI * 2;

let ctx = null;
let masterGain = null;
let sfxGain = null;
let bgmGain = null;
let comp = null;
let sfxOn = true;
let musicOn = true;

const rawBuffers = {};
const audioBuffers = {};
const lastPlayed = {};
let activeVoices = 0;

let seed = 24681357;
const rnd = () => ((seed = (seed * 1103515245 + 12345) & 0x7fffffff), seed / 0x7fffffff);

const panLR = (pan = 0) => {
  const a = ((pan + 1) * Math.PI) / 4;
  return [Math.cos(a), Math.sin(a)];
};

const alloc = (dur) => {
  const len = Math.max(1, Math.floor(dur * SR));
  return [new Float32Array(len), new Float32Array(len)];
};

function tone(buf, start, dur, f0, f1, amp, decay, pan = 0, detune = 0, mode = 0) {
  const [gl, gr] = panLR(pan);
  const s0 = Math.floor(start * SR);
  const len = Math.min(buf[0].length - s0, Math.floor(dur * SR));
  if (len <= 0) return;
  let phL = 0, phR = 0.28;
  const k = f1 && f1 !== f0 ? Math.log(f1 / f0) / len : 0;
  const det = 1 + detune;
  for (let i = 0; i < len; i++) {
    const t = i / SR;
    const f = k ? f0 * Math.exp(k * i) : f0;
    phL += (TAU * f) / SR;
    phR += (TAU * f * det) / SR;
    const env = Math.min(1, t / 0.002) * Math.exp(-t / decay) * Math.min(1, (len - i) / (0.004 * SR));
    let wl = Math.sin(phL);
    let wr = Math.sin(phR);
    if (mode === 1) {
      wl += 0.35 * Math.sin(phL * 3) + 0.2 * Math.sin(phL * 5);
      wr += 0.35 * Math.sin(phR * 3) + 0.2 * Math.sin(phR * 5);
    } else if (mode === 2) {
      wl += 0.5 * Math.sin(phL * 2) + 0.25 * Math.sin(phL * 3);
      wr += 0.5 * Math.sin(phR * 2) + 0.25 * Math.sin(phR * 3);
    }
    buf[0][s0 + i] += wl * env * amp * gl;
    buf[1][s0 + i] += wr * env * amp * gr;
  }
}

function noise(buf, start, dur, amp, decay, fc0, fc1 = 0, q = 1, type = 0, pan = 0, atk = 0.001) {
  const [gl, gr] = panLR(pan);
  const s0 = Math.floor(start * SR);
  const len = Math.min(buf[0].length - s0, Math.floor(dur * SR));
  if (len <= 0) return;
  let b0 = 0, b1 = 0, b2 = 0, a1 = 0, a2 = 0;
  const mem = [0, 0, 0, 0, 0, 0, 0, 0];
  const k = fc1 && fc1 !== fc0 ? Math.log(fc1 / fc0) / len : 0;
  for (let i = 0; i < len; i++) {
    if (!(i & 31)) {
      const fc = Math.min(SR * 0.45, Math.max(30, k ? fc0 * Math.exp(k * i) : fc0));
      const w0 = (TAU * fc) / SR;
      const cos = Math.cos(w0);
      const alpha = Math.sin(w0) / (2 * q);
      const a0 = 1 + alpha;
      if (type === 1) {
        b0 = (1 - cos) / 2 / a0;
        b1 = (1 - cos) / a0;
        b2 = b0;
      } else if (type === 2) {
        b0 = (1 + cos) / 2 / a0;
        b1 = -(1 + cos) / a0;
        b2 = b0;
      } else {
        b0 = alpha / a0;
        b1 = 0;
        b2 = -alpha / a0;
      }
      a1 = (-2 * cos) / a0;
      a2 = (1 - alpha) / a0;
    }
    const t = i / SR;
    const env = Math.min(1, t / atk) * Math.exp(-t / decay) * Math.min(1, (len - i) / (0.004 * SR));
    for (let ch = 0; ch < 2; ch++) {
      const x = rnd() * 2 - 1;
      const m = ch * 4;
      const y = b0 * x + b1 * mem[m] + b2 * mem[m + 1] - a1 * mem[m + 2] - a2 * mem[m + 3];
      mem[m + 1] = mem[m];
      mem[m] = x;
      mem[m + 3] = mem[m + 2];
      mem[m + 2] = y;
      buf[ch][s0 + i] += y * env * amp * (ch ? gr : gl);
    }
  }
}

const bell = (b, st, f, a, dec, pan = 0) => {
  tone(b, st, dec * 4, f, 0, a, dec, pan, 0.002);
  tone(b, st, dec * 3, f * 2.76, 0, a * 0.35, dec * 0.6, -pan, 0.003);
  tone(b, st, dec * 2, f * 5.4, 0, a * 0.15, dec * 0.3, pan, 0);
};

const chime = (b, st, f, a, dec, pan = 0) => {
  tone(b, st, dec * 4, f, 0, a, dec, pan, 0.003);
  tone(b, st, dec * 3, f * 2, 0, a * 0.4, dec * 0.5, -pan * 0.5, 0.002);
  noise(b, st, 0.012, a * 0.45, 0.004, f * 4, 0, 1.2, 0, pan);
};

function commit(name, buf, peakTarget = 0.8) {
  let max = 0;
  const n = buf[0].length;
  const fade = Math.min(Math.floor(0.003 * SR), n >> 1);
  for (let ch = 0; ch < 2; ch++) {
    for (let i = 0; i < n; i++) {
      let v = buf[ch][i];
      if (!isFinite(v)) v = 0;
      if (i < fade) v *= i / fade;
      if (n - 1 - i < fade) v *= (n - 1 - i) / fade;
      buf[ch][i] = v;
      const av = Math.abs(v);
      if (av > max) max = av;
    }
  }
  const scale = max > 0 ? peakTarget / max : 0;
  for (let ch = 0; ch < 2; ch++) {
    for (let i = 0; i < n; i++) buf[ch][i] *= scale;
  }
  rawBuffers[name] = buf;
}

const RECIPES = [
  ["hit", () => { const b = alloc(0.09); noise(b, 0, 0.03, 1, 0.005, 3200, 1800, 1.5); tone(b, 0, 0.08, 620, 380, 0.65, 0.02, 0, 0.004); commit("hit", b, 0.7); }],
  ["hit2", () => { const b = alloc(0.09); noise(b, 0, 0.03, 1, 0.005, 3800, 2100, 1.6, 0, 0.1); tone(b, 0, 0.08, 740, 440, 0.6, 0.02, 0, 0.004); commit("hit2", b, 0.7); }],
  ["crit", () => { const b = alloc(0.24); noise(b, 0, 0.02, 0.9, 0.004, 5800, 3200, 1.3); bell(b, 0, 1760, 0.7, 0.05, 0.2); bell(b, 0.012, 2349, 0.5, 0.045, -0.2); commit("crit", b, 0.76); }],
  ["pop", () => { const b = alloc(0.12); noise(b, 0, 0.015, 0.85, 0.004, 3400, 0, 1); tone(b, 0, 0.05, 380, 1020, 0.9, 0.024, 0, 0.005); commit("pop", b, 0.72); }],
  ["pop2", () => { const b = alloc(0.12); noise(b, 0, 0.015, 0.85, 0.004, 4100, 0, 1); tone(b, 0, 0.05, 480, 1260, 0.9, 0.022, 0, 0.005); commit("pop2", b, 0.72); }],
  ["pop3", () => { const b = alloc(0.15); noise(b, 0, 0.018, 0.85, 0.005, 2600, 0, 1); tone(b, 0, 0.07, 260, 780, 0.95, 0.03, 0, 0.005); commit("pop3", b, 0.74); }],
  ["shatter", () => { const b = alloc(0.4); for (let i = 0; i < 8; i++) { const st = i ? rnd() * 0.18 : 0; bell(b, st, 1900 + rnd() * 2600, 0.24, 0.04, rnd() * 1.4 - 0.7); } tone(b, 0, 0.2, 190, 68, 0.9, 0.06); commit("shatter", b, 0.8); }],
  ["gem", () => { const b = alloc(0.18); chime(b, 0, 987.77, 0.9, 0.042, 0); commit("gem", b, 0.6); }],
  ["shoot", () => { const b = alloc(0.1); noise(b, 0, 0.06, 0.8, 0.015, 4800, 1600, 1.4); tone(b, 0, 0.08, 920, 440, 0.7, 0.025, 0, 0.003); commit("shoot", b, 0.5); }],
  ["star", () => { const b = alloc(0.24); noise(b, 0, 0.22, 1, 0.08, 700, 2800, 2.2, 0, -0.3, 0.02); tone(b, 0, 0.12, 520, 780, 0.3, 0.05); commit("star", b, 0.5); }],
  ["zap", () => { const b = alloc(0.18); noise(b, 0, 0.15, 1, 0.032, 4600, 1700, 0.9, 2); tone(b, 0, 0.11, 2800, 520, 0.55, 0.03, 0, 0.01); commit("zap", b, 0.6); }],
  ["sonar", () => { const b = alloc(0.46); tone(b, 0, 0.3, 740, 710, 0.85, 0.09, -0.2, 0.002); tone(b, 0.13, 0.28, 740, 710, 0.3, 0.08, 0.4, 0.002); tone(b, 0, 0.12, 130, 82, 0.5, 0.04); commit("sonar", b, 0.6); }],
  ["drop", () => { const b = alloc(0.1); tone(b, 0, 0.09, 260, 160, 1, 0.03); commit("drop", b, 0.5); }],
  ["boom", () => { const b = alloc(0.4); tone(b, 0, 0.34, 155, 42, 1, 0.11); noise(b, 0, 0.28, 0.85, 0.07, 980, 180, 0.7, 1); commit("boom", b, 0.8); }],
  ["beam", () => { const b = alloc(0.42); tone(b, 0, 0.4, 261.6, 277.2, 0.5, 0.18, -0.3, 0.004, 2); tone(b, 0, 0.4, 392, 415.3, 0.35, 0.16, 0.3, 0.004, 2); commit("beam", b, 0.5); }],
  ["whirl", () => { const b = alloc(0.5); noise(b, 0, 0.48, 1, 0.2, 320, 1600, 2.5, 0, -0.3, 0.07); tone(b, 0, 0.45, 115, 72, 0.5, 0.18); commit("whirl", b, 0.5); }],
  ["level", () => { const b = alloc(1.05); [587.33, 739.99, 880, 1174.66, 1479.98].forEach((f, i) => chime(b, i * 0.07, f, 0.8, 0.15, i % 2 ? 0.4 : -0.4)); commit("level", b, 0.8); }],
  ["select", () => { const b = alloc(0.28); chime(b, 0, 1046.5, 0.8, 0.06, -0.2); chime(b, 0.065, 1396.9, 0.8, 0.08, 0.2); commit("select", b, 0.6); }],
  ["evo", () => { const b = alloc(1.4); [440, 554.37, 659.25, 880, 1108.73, 1318.5, 1760].forEach((f, i) => chime(b, i * 0.068, f, 0.82, 0.2, i % 2 ? 0.5 : -0.5)); tone(b, 0.48, 0.4, 140, 52, 0.9, 0.12); commit("evo", b, 0.85); }],
  ["chest", () => { const b = alloc(0.85); tone(b, 0, 0.28, 170, 75, 0.9, 0.09); for (let i = 0; i < 12; i++) bell(b, 0.08 + i * 0.045, 1250 + i * 155, 0.3, 0.06, rnd() * 1.6 - 0.8); commit("chest", b, 0.76); }],
  ["hurt", () => { const b = alloc(0.26); tone(b, 0, 0.24, 125, 58, 1, 0.07); noise(b, 0, 0.06, 0.65, 0.015, 950, 380, 0.8, 1); commit("hurt", b, 0.76); }],
  ["dead", () => { const b = alloc(1.2); [440, 370, 311, 220].forEach((f, i) => tone(b, i * 0.15, 0.55, f, f * 0.9, 0.6, 0.2, i % 2 ? 0.4 : -0.4, 0.005, 2)); commit("dead", b, 0.8); }],
  ["warn", () => { const b = alloc(0.95); for (let i = 0; i < 2; i++) { tone(b, i * 0.45, 0.38, 220, 196, 0.75, 0.16, i ? 0.4 : -0.4, 0.004, 2); tone(b, i * 0.45, 0.38, 110, 98, 0.8, 0.18); } commit("warn", b, 0.72); }],
  ["bossdead", () => { const b = alloc(1.8); tone(b, 0, 0.85, 180, 36, 1, 0.28); noise(b, 0, 0.75, 0.9, 0.2, 1300, 130, 0.7, 1); [587.33, 739.99, 880, 1174.66, 1479.98, 1760].forEach((f, i) => bell(b, 0.48 + i * 0.085, f, 0.42, 0.14, i % 2 ? 0.6 : -0.6)); commit("bossdead", b, 0.88); }],
  ["nuke", () => { const b = alloc(1.3); noise(b, 0, 0.24, 0.5, 0.4, 320, 5200, 2, 0, 0, 0.18); tone(b, 0.22, 0.78, 195, 36, 1, 0.24); noise(b, 0.22, 0.68, 0.9, 0.18, 2100, 150, 0.7, 1); commit("nuke", b, 0.9); }],
  ["magnet", () => { const b = alloc(0.5); tone(b, 0, 0.46, 300, 1350, 0.6, 0.28, -0.3, 0.006); tone(b, 0.03, 0.46, 450, 1980, 0.32, 0.28, 0.3, 0.006); commit("magnet", b, 0.6); }],
  ["heart", () => { const b = alloc(0.4); chime(b, 0, 880, 0.8, 0.1, -0.3); chime(b, 0.09, 1174.66, 0.8, 0.12, 0.3); commit("heart", b, 0.6); }],
  ["core", () => { const b = alloc(0.16); bell(b, 0, 2217.5, 0.7, 0.03, 0.2); bell(b, 0.03, 2793.8, 0.5, 0.03, -0.2); commit("core", b, 0.52); }],
  ["tap", () => { const b = alloc(0.09); tone(b, 0, 0.08, 960, 820, 0.8, 0.018); commit("tap", b, 0.45); }],
  ["buy", () => { const b = alloc(0.46); [659.25, 830.6, 987.77, 1318.5].forEach((f, i) => chime(b, i * 0.058, f, 0.8, 0.1, i % 2 ? 0.4 : -0.4)); commit("buy", b, 0.66); }],
  ["err", () => { const b = alloc(0.2); tone(b, 0, 0.08, 196, 180, 0.8, 0.04, 0, 0.004, 1); tone(b, 0.1, 0.09, 165, 150, 0.8, 0.04, 0, 0.004, 1); commit("err", b, 0.45); }],
  ["combo", () => { const b = alloc(0.48); [880, 1108.73, 1318.5, 1760].forEach((f, i) => chime(b, i * 0.048, f, 0.8, 0.09, i % 2 ? 0.5 : -0.5)); commit("combo", b, 0.7); }],
  ["eshot", () => { const b = alloc(0.14); tone(b, 0, 0.13, 350, 175, 0.8, 0.04, 0, 0.004, 1); commit("eshot", b, 0.4); }],
  ["dash", () => { const b = alloc(0.22); noise(b, 0, 0.2, 1, 0.065, 960, 300, 1.6, 0, 0, 0.02); commit("dash", b, 0.5); }],
  ["wave", () => { const b = alloc(0.38); tone(b, 0, 0.34, 110, 65, 1, 0.12); chime(b, 0.05, 659.25, 0.32, 0.1, 0.3); commit("wave", b, 0.6); }],
  ["elite", () => { const b = alloc(0.56); tone(b, 0, 0.5, 92, 123, 0.9, 0.2, 0, 0.006, 2); bell(b, 0.2, 659.25, 0.4, 0.12, -0.3); commit("elite", b, 0.65); }],
  ["revive", () => { const b = alloc(0.96); [440, 554.37, 659.25, 880, 1108.73, 1318.5].forEach((f, i) => chime(b, i * 0.075, f, 0.8, 0.18, i % 2 ? 0.5 : -0.5)); commit("revive", b, 0.8); }],
  // Cyber-Synthwave BGM Instruments (D minor / A minor pentatonic pulse)
  ["m_kick", () => { const b = alloc(0.2); tone(b, 0, 0.19, 130, 42, 1, 0.07); noise(b, 0, 0.012, 0.32, 0.003, 2100); commit("m_kick", b, 0.82); }],
  ["m_hat", () => { const b = alloc(0.085); noise(b, 0, 0.075, 1, 0.013, 8600, 0, 1.2, 2); commit("m_hat", b, 0.4); }],
  ["m_bass", () => { const b = alloc(0.3); tone(b, 0, 0.28, 110, 0, 1, 0.11, 0, 0.003, 2); tone(b, 0, 0.28, 55, 0, 0.82, 0.14); commit("m_bass", b, 0.8); }],
  ["m_arp", () => { const b = alloc(0.28); tone(b, 0, 0.26, 440, 0, 0.88, 0.068, 0, 0.004, 2); tone(b, 0, 0.18, 880, 0, 0.3, 0.04, 0.3, 0.004); commit("m_arp", b, 0.58); }],
  ["m_pad", () => { const b = alloc(1.85); [220, 261.63, 329.63].forEach((f, i) => tone(b, 0, 1.85, f, 0, 0.38, 2.8, i - 1, 0.004, 2)); commit("m_pad", b, 0.48); }],
];

let recipeCursor = 0;
export function warmAudioStep(all = false) {
  const start = Date.now();
  while (recipeCursor < RECIPES.length) {
    RECIPES[recipeCursor][1]();
    recipeCursor++;
    if (!all && Date.now() - start > 10) break;
  }
  if (recipeCursor >= RECIPES.length) {
    uploadBuffers();
    return true;
  }
  return false;
}

function uploadBuffers() {
  if (!ctx) return;
  for (const k in rawBuffers) {
    if (audioBuffers[k]) continue;
    const raw = rawBuffers[k];
    const ab = ctx.createBuffer(2, raw[0].length, SR);
    ab.getChannelData(0).set(raw[0]);
    ab.getChannelData(1).set(raw[1]);
    audioBuffers[k] = ab;
  }
}

function ensureAudio() {
  if (!ctx) {
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return;
    try {
      ctx = new AC();
    } catch {
      return;
    }
    comp = ctx.createDynamicsCompressor();
    masterGain = ctx.createGain();
    masterGain.gain.value = 0.9;
    sfxGain = ctx.createGain();
    sfxGain.gain.value = sfxOn ? 0.85 : 0;
    bgmGain = ctx.createGain();
    bgmGain.gain.value = 0;
    sfxGain.connect(comp);
    bgmGain.connect(comp);
    comp.connect(masterGain);
    masterGain.connect(ctx.destination);
    warmAudioStep(true);
    uploadBuffers();
  }
  if (ctx && ctx.state !== "running" && ctx.state !== "closed" && !document.hidden) {
    ctx.resume().catch(() => {});
  }
}

if (typeof window !== "undefined") {
  ["pointerup", "touchend", "click", "keydown"].forEach((ev) =>
    window.addEventListener(ev, ensureAudio, { capture: true, passive: true }),
  );
  document.addEventListener("visibilitychange", () => {
    if (!ctx) return;
    if (document.hidden) ctx.suspend && ctx.suspend().catch(() => {});
    else ctx.resume && ctx.resume().catch(() => {});
  });
}

const COOLDOWNS = {
  hit: 0.045, hit2: 0.045, crit: 0.07, pop: 0.035, pop2: 0.035, pop3: 0.05,
  gem: 0.036, shoot: 0.085, boom: 0.08, zap: 0.09, eshot: 0.12, core: 0.06,
  shatter: 0.12, star: 0.15, sonar: 0.2, drop: 0.1, beam: 0.3, whirl: 0.4,
};

const GAINS = {
  hit: 0.34, hit2: 0.34, crit: 0.5, pop: 0.48, pop2: 0.48, pop3: 0.54,
  gem: 0.42, shoot: 0.24, star: 0.3, zap: 0.4, sonar: 0.42, drop: 0.35,
  boom: 0.6, beam: 0.3, whirl: 0.34, level: 0.8, select: 0.6, evo: 0.9,
  chest: 0.75, hurt: 0.8, dead: 0.85, warn: 0.75, bossdead: 1, nuke: 1,
  magnet: 0.6, heart: 0.6, core: 0.44, tap: 0.48, buy: 0.7, err: 0.5,
  combo: 0.62, eshot: 0.25, dash: 0.4, wave: 0.5, elite: 0.6, revive: 0.8, shatter: 0.7,
};

function spawnSource(buf, when, gainVal, pan, rate, dest) {
  const src = ctx.createBufferSource();
  src.buffer = buf;
  src.playbackRate.value = rate;
  const g = ctx.createGain();
  g.gain.value = gainVal;
  src.connect(g);
  if (pan && ctx.createStereoPanner) {
    const p = ctx.createStereoPanner();
    p.pan.value = Math.max(-1, Math.min(1, pan));
    g.connect(p);
    p.connect(dest);
  } else {
    g.connect(dest);
  }
  activeVoices++;
  src.onended = () => { activeVoices--; };
  src.start(when);
}

export function playSfx(name, pan = 0, rate = 1, volMul = 1) {
  if (!ctx || !sfxOn || ctx.state !== "running") return;
  const buf = audioBuffers[name];
  if (!buf) return;
  const now = ctx.currentTime;
  const cd = COOLDOWNS[name] || 0.02;
  if (lastPlayed[name] && now - lastPlayed[name] < cd) return;
  if (activeVoices >= 30 && !(GAINS[name] >= 0.6)) return;
  lastPlayed[name] = now;
  spawnSource(buf, now, (GAINS[name] || 0.5) * volMul, pan, rate, sfxGain);
}

const PENTA = [0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24, 26, 28];
let gemStep = 0;
let lastGemT = 0;

export function playGemChime(pan, tier) {
  if (!ctx) return;
  const now = ctx.currentTime;
  if (now - lastGemT > 0.42) gemStep = 0;
  if (now - lastGemT < 0.035) return;
  lastGemT = now;
  const semi = PENTA[Math.min(PENTA.length - 1, gemStep)] + (tier === 2 ? 12 : tier === 1 ? 7 : 0);
  gemStep = gemStep < PENTA.length - 1 ? gemStep + 1 : PENTA.length - 3;
  playSfx("gem", pan, Math.pow(2, (semi - 5) / 12), tier ? 1.15 : 1);
}

let popIdx = 0;
export function playKillPop(pan, big, combo) {
  const rate = 0.92 + (popIdx++ % 5) * 0.045 + Math.min(0.3, (combo || 0) / 1500);
  playSfx(big ? "pop3" : popIdx % 2 ? "pop" : "pop2", pan, rate);
}

export function setAudioPrefs(sfx, music) {
  sfxOn = !!sfx;
  musicOn = !!music;
  if (ctx) {
    sfxGain.gain.value = sfxOn ? 0.85 : 0;
    if (!musicOn) {
      bgmGain.gain.cancelScheduledValues(ctx.currentTime);
      bgmGain.gain.value = 0;
      bgmPlaying = false;
    }
  }
}

const BPM = 130;
const STEP_DUR = 60 / BPM / 4;
const ROOTS = [0, -4, 3, -2];
const CHORDS = [[0, 3, 7, 12], [-4, 0, 3, 8], [3, 7, 10, 15], [-2, 2, 5, 10]];
let bgmPlaying = false;
let nextStepT = 0;
let stepIdx = 0;
let bgmIntensity = 0;

export function startBgm() {
  if (!ctx || !musicOn || bgmPlaying) return;
  bgmPlaying = true;
  stepIdx = 0;
  nextStepT = ctx.currentTime + 0.08;
  bgmGain.gain.cancelScheduledValues(ctx.currentTime);
  bgmGain.gain.setValueAtTime(bgmGain.gain.value, ctx.currentTime);
  bgmGain.gain.linearRampToValueAtTime(0.4, ctx.currentTime + 0.6);
}

export function stopBgm(fade = 0.25) {
  if (!ctx || !bgmPlaying) return;
  bgmPlaying = false;
  const now = ctx.currentTime;
  bgmGain.gain.cancelScheduledValues(now);
  bgmGain.gain.setValueAtTime(bgmGain.gain.value, now);
  bgmGain.gain.linearRampToValueAtTime(0, now + fade);
}

export function setBgmIntensity(lv) {
  bgmIntensity = lv;
}

export function tickBgm() {
  if (!ctx || !bgmPlaying || ctx.state !== "running") return;
  const now = ctx.currentTime;
  if (nextStepT < now - 0.3) nextStepT = now + 0.05;
  while (nextStepT < now + 0.22) {
    const s = stepIdx % 16;
    const bar = Math.floor(stepIdx / 16) % 4;
    const root = ROOTS[bar];
    const chord = CHORDS[bar];
    const lv = bgmIntensity;
    const semi = (n) => Math.pow(2, n / 12);

    if ((s === 0 || s === 8 || (lv >= 1 && (s === 4 || s === 12))) && audioBuffers.m_kick) {
      spawnSource(audioBuffers.m_kick, nextStepT, s % 8 === 0 ? 0.75 : 0.55, 0, 1, bgmGain);
    }
    if ((s === 0 || s === 6 || s === 10 || (lv >= 2 && (s === 2 || s === 14))) && audioBuffers.m_bass) {
      spawnSource(audioBuffers.m_bass, nextStepT, 0.62, 0, semi(root + (s === 10 ? 12 : 0)), bgmGain);
    }
    if (lv >= 1 && s % 4 === 2 && audioBuffers.m_hat) {
      spawnSource(audioBuffers.m_hat, nextStepT, 0.26, s % 8 === 2 ? 0.3 : -0.3, 1, bgmGain);
    }
    if (lv >= 2 && s % 2 === 1 && audioBuffers.m_hat) {
      spawnSource(audioBuffers.m_hat, nextStepT, 0.11, s % 4 === 1 ? -0.4 : 0.4, 1.22, bgmGain);
    }
    if (lv >= 1 && s === 0 && audioBuffers.m_pad) {
      spawnSource(audioBuffers.m_pad, nextStepT, 0.48, 0, semi(root), bgmGain);
    }
    if (lv >= 2 && (s % 2 === 0 || lv >= 3) && audioBuffers.m_arp) {
      const note = chord[(s * 3 + bar) % 4] + (lv >= 3 && s % 4 === 3 ? 12 : 0);
      spawnSource(audioBuffers.m_arp, nextStepT, lv >= 3 ? 0.3 : 0.22, (s % 4 - 1.5) * 0.3, semi(note), bgmGain);
    }
    nextStepT += STEP_DUR;
    stepIdx++;
  }
}

export function vibrate(kind = "light") {
  try {
    if (navigator.vibrate) {
      navigator.vibrate(kind === "heavy" ? 28 : kind === "medium" ? 16 : 8);
    }
  } catch {
    // Ignore unsupported devices
  }
}
