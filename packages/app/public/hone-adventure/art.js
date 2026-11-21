// HONE Adventure — Custom Sprite Atlas, Procedural Cyber-Vector Art, Icons & WebGL/2D Renderer

import { SHEET_FRAMES } from "./atlas-coords.js";

const TAU = Math.PI * 2;

export function rgbaHex(hex, a) {
  const n = parseInt(hex.slice(1), 16);
  return `rgba(${n >> 16},${(n >> 8) & 255},${n & 255},${a})`;
}

export function softGlow(ctx, x, y, r, hex, alpha) {
  const g = ctx.createRadialGradient(x, y, 0, x, y, r);
  g.addColorStop(0, rgbaHex(hex, alpha));
  g.addColorStop(0.45, rgbaHex(hex, alpha * 0.4));
  g.addColorStop(1, rgbaHex(hex, 0));
  ctx.fillStyle = g;
  ctx.beginPath();
  ctx.arc(x, y, r, 0, TAU);
  ctx.fill();
}

export function radGrad(ctx, x0, y0, r0, x1, y1, r1, stops) {
  const g = ctx.createRadialGradient(x0, y0, r0, x1, y1, r1);
  for (let i = 0; i < stops.length; i += 2) g.addColorStop(stops[i], stops[i + 1]);
  return g;
}

export function linGrad(ctx, x0, y0, x1, y1, stops) {
  const g = ctx.createLinearGradient(x0, y0, x1, y1);
  for (let i = 0; i < stops.length; i += 2) g.addColorStop(stops[i], stops[i + 1]);
  return g;
}

function circle(ctx, x, y, r) {
  ctx.beginPath();
  ctx.arc(x, y, r, 0, TAU);
}

function poly(ctx, x, y, sides, r, rot = 0) {
  ctx.beginPath();
  for (let i = 0; i < sides; i++) {
    const a = (i / sides) * TAU + rot;
    const px = x + Math.cos(a) * r;
    const py = y + Math.sin(a) * r;
    if (i === 0) ctx.moveTo(px, py);
    else ctx.lineTo(px, py);
  }
  ctx.closePath();
}

function starPoly(ctx, x, y, pts, rOuter, rInner, rot = 0) {
  ctx.beginPath();
  for (let i = 0; i < pts * 2; i++) {
    const a = (i / (pts * 2)) * TAU + rot;
    const r = i % 2 ? rInner : rOuter;
    const px = x + Math.cos(a) * r;
    const py = y + Math.sin(a) * r;
    if (i === 0) ctx.moveTo(px, py);
    else ctx.lineTo(px, py);
  }
  ctx.closePath();
}

export function roundRectPath(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

export const SPRITE_DEFS = [];
export const SPR_IDX = {};

function defSpr(name, w, h, fn) {
  SPR_IDX[name] = SPRITE_DEFS.length;
  SPRITE_DEFS.push({ n: name, w, h, f: fn });
}

// Fallback procedural hero/enemy shapes (overridden automatically when sprites.webp loads)
const HERO_PAL = {
  hone: ["#ffad36", "#f06449", "#17201f"],
  gold: ["#ffe27a", "#ff9f1c", "#1a1408"],
  rubin: ["#2ee59d", "#187a6c", "#102422"],
  photon: ["#7fe8ff", "#38bdf8", "#0f2438"],
  quant: ["#6fffe9", "#2dd4bf", "#0d2824"],
};

Object.keys(HERO_PAL).forEach((id) => {
  const [c1, c2, dark] = HERO_PAL[id];
  const draw = (ctx, sc) => {
    softGlow(ctx, 0, 0, 32, c1, 0.38);
    ctx.fillStyle = linGrad(ctx, 0, -20, 0, 20, [0, c1, 1, c2]);
    ctx.strokeStyle = dark;
    ctx.lineWidth = 2;
    poly(ctx, 0, 2, 6, 18, -Math.PI / 2);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = "#ffffff";
    circle(ctx, 5, -2, 3.5);
    ctx.fill();
  };
  defSpr("h_" + id, 68, 68, draw);
  defSpr("h_" + id + "_joy", 68, 68, draw);
  defSpr("h_" + id + "_hurt", 68, 68, draw);
  defSpr("P_" + id, 84, 84, draw);
});

const ENEMY_PAL = {
  glitch: [34, "#c084fc"],
  fomo: [40, "#f43f5e"],
  spam: [52, "#6366f1"],
  rumor: [42, "#fb923c"],
  crash: [44, "#ef4444"],
  bubble_mob: [46, "#d8b4fe"],
  margin_bomb: [42, "#f97316"],
  hallucination: [42, "#67e8f9"],
  mini: [24, "#a855f7"],
  boss1: [128, "#e879f9"],
  boss2: [148, "#fb7185"],
  boss3: [144, "#c084fc"],
  boss4: [160, "#f43f5e"],
};

Object.entries(ENEMY_PAL).forEach(([id, [sz, col]]) => {
  defSpr("e_" + id, sz, sz, (ctx) => {
    const r = sz * 0.32;
    softGlow(ctx, 0, 0, r * 1.6, col, 0.35);
    ctx.fillStyle = "#221638";
    ctx.strokeStyle = col;
    ctx.lineWidth = 2;
    poly(ctx, 0, 0, id.startsWith("boss") ? 8 : 6, r, 0.2);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = col;
    circle(ctx, -r * 0.28, -r * 0.1, r * 0.16);
    ctx.fill();
    circle(ctx, r * 0.28, -r * 0.1, r * 0.16);
    ctx.fill();
  });
});

// Projectiles, Shields, Zones & FX Sprites
defSpr("blade", 36, 22, (ctx, sc) => {
  softGlow(ctx, 0, 0, 16, "#ffad36", 0.5);
  ctx.save();
  ctx.shadowColor = "#ffad36";
  ctx.shadowBlur = 4 * sc;
  ctx.beginPath();
  ctx.moveTo(15, 0);
  ctx.lineTo(-4, -6.5);
  ctx.lineTo(-12, 0);
  ctx.lineTo(-4, 6.5);
  ctx.closePath();
  ctx.fillStyle = linGrad(ctx, -12, 0, 15, 0, [0, "#f15b08", 0.5, "#ffad36", 1, "#fff8dc"]);
  ctx.fill();
  ctx.strokeStyle = "#241608";
  ctx.lineWidth = 1.2;
  ctx.stroke();
  ctx.restore();
});

defSpr("wafer", 40, 40, (ctx, sc) => {
  softGlow(ctx, 0, 0, 20, "#5ce1e6", 0.48);
  ctx.save();
  ctx.shadowColor = "#5ce1e6";
  ctx.shadowBlur = 5 * sc;
  starPoly(ctx, 0, 0, 5, 15, 7, 0);
  ctx.fillStyle = radGrad(ctx, 0, 0, 1, 0, 0, 15, [0, "#ffffff", 0.45, "#5ce1e6", 1, "#0284c7"]);
  ctx.fill();
  ctx.strokeStyle = "#e0faff";
  ctx.lineWidth = 1.1;
  ctx.stroke();
  ctx.restore();
  ctx.fillStyle = "#ffad36";
  circle(ctx, 0, 0, 3.2);
  ctx.fill();
});

defSpr("moat", 30, 30, (ctx, sc) => {
  softGlow(ctx, 0, 0, 15, "#ffd97d", 0.58);
  ctx.save();
  ctx.shadowColor = "#ffad36";
  ctx.shadowBlur = 4 * sc;
  poly(ctx, 0, 0, 6, 8.8, -Math.PI / 2);
  ctx.fillStyle = linGrad(ctx, 0, -9, 0, 9, [0, "#fffbeb", 0.45, "#ffd166", 1, "#d97706"]);
  ctx.fill();
  ctx.strokeStyle = "#ffffff";
  ctx.lineWidth = 1.1;
  ctx.stroke();
  ctx.restore();
  poly(ctx, 0, 0, 6, 4.2, -Math.PI / 2);
  ctx.strokeStyle = "rgba(120,53,15,.65)";
  ctx.lineWidth = 0.9;
  ctx.stroke();
});

defSpr("mine", 34, 34, (ctx, sc) => {
  softGlow(ctx, 0, 0, 17, "#ff6b6b", 0.48);
  starPoly(ctx, 0, 0, 8, 13, 7.5, 0);
  ctx.fillStyle = radGrad(ctx, 0, 0, 1, 0, 0, 13, [0, "#ffd166", 0.45, "#ef4444", 1, "#7f1d1d"]);
  ctx.fill();
  ctx.strokeStyle = "#fecaca";
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.fillStyle = "#fff";
  circle(ctx, 0, 0, 2.6);
  ctx.fill();
});

defSpr("swirl", 128, 128, (ctx) => {
  softGlow(ctx, 0, 0, 62, "#a78bfa", 0.34);
  for (let i = 0; i < 4; i++) {
    ctx.beginPath();
    for (let s = 0; s <= 44; s++) {
      const k = s / 44;
      const a = (i / 4) * TAU + k * 4.4;
      const r = 4 + k * 54;
      const px = Math.cos(a) * r;
      const py = Math.sin(a) * r;
      if (s === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.strokeStyle = i % 2 ? "rgba(92,225,230,.58)" : "rgba(196,181,253,.85)";
    ctx.lineWidth = i % 2 ? 4.8 : 2.5;
    ctx.lineCap = "round";
    ctx.stroke();
  }
  softGlow(ctx, 0, 0, 18, "#090618", 0.95);
});

defSpr("beam", 64, 28, (ctx) => {
  ctx.fillStyle = linGrad(ctx, 0, -14, 0, 14, [
    0, "rgba(255,255,255,0)",
    0.28, "rgba(255,255,255,.42)",
    0.46, "rgba(255,255,255,.96)",
    0.54, "rgba(255,255,255,.96)",
    0.72, "rgba(255,255,255,.42)",
    1, "rgba(255,255,255,0)",
  ]);
  ctx.fillRect(-32, -14, 64, 28);
  ctx.globalCompositeOperation = "destination-in";
  ctx.fillStyle = linGrad(ctx, -32, 0, 32, 0, [
    0, "rgba(0,0,0,.2)",
    0.06, "rgba(0,0,0,1)",
    0.82, "rgba(0,0,0,1)",
    1, "rgba(0,0,0,0)",
  ]);
  ctx.fillRect(-32, -14, 64, 28);
  ctx.globalCompositeOperation = "source-over";
});

defSpr("bolt", 64, 14, (ctx) => {
  ctx.fillStyle = linGrad(ctx, 0, -7, 0, 7, [
    0, "rgba(111,255,233,0)",
    0.3, "rgba(111,255,233,.58)",
    0.47, "#ffffff",
    0.53, "#ffffff",
    0.7, "rgba(111,255,233,.58)",
    1, "rgba(111,255,233,0)",
  ]);
  ctx.fillRect(-32, -7, 64, 14);
});

defSpr("ebullet", 22, 22, (ctx) => {
  softGlow(ctx, 0, 0, 11, "#f43f5e", 0.68);
  circle(ctx, 0, 0, 6);
  ctx.fillStyle = radGrad(ctx, 0, 0, 0.5, 0, 0, 6, [0, "#ffffff", 0.4, "#fecdd3", 0.75, "#f43f5e", 1, "#9f1239"]);
  ctx.fill();
  ctx.strokeStyle = "#3b0819";
  ctx.lineWidth = 0.9;
  ctx.stroke();
});

defSpr("crown", 22, 16, (ctx, sc) => {
  ctx.save();
  ctx.shadowColor = "#ffd166";
  ctx.shadowBlur = 4 * sc;
  ctx.fillStyle = linGrad(ctx, 0, -6, 0, 5, [0, "#fffbeb", 1, "#f59e0b"]);
  ctx.beginPath();
  ctx.moveTo(-7, 5);
  ctx.lineTo(-8.5, -4);
  ctx.lineTo(-4, 0);
  ctx.lineTo(0, -6);
  ctx.lineTo(4, 0);
  ctx.lineTo(8.5, -4);
  ctx.lineTo(7, 5);
  ctx.closePath();
  ctx.fill();
  ctx.restore();
});

defSpr("ring", 128, 128, (ctx, sc) => {
  ctx.save();
  ctx.shadowColor = "#fff";
  ctx.shadowBlur = 6 * sc;
  ctx.strokeStyle = "#fff";
  ctx.lineWidth = 3;
  circle(ctx, 0, 0, 56);
  ctx.stroke();
  ctx.restore();
});

defSpr("ring2", 128, 128, (ctx) => {
  ctx.strokeStyle = radGrad(ctx, 0, 0, 40, 0, 0, 62, [
    0, "rgba(255,255,255,0)",
    0.75, "rgba(255,255,255,.9)",
    1, "rgba(255,255,255,0)",
  ]);
  ctx.lineWidth = 22;
  circle(ctx, 0, 0, 51);
  ctx.stroke();
});

defSpr("disc", 64, 64, (ctx) => {
  circle(ctx, 0, 0, 30);
  ctx.fillStyle = radGrad(ctx, 0, 0, 0, 0, 0, 30, [
    0, "rgba(255,255,255,.55)",
    0.8, "rgba(255,255,255,.75)",
    0.94, "rgba(255,255,255,1)",
    1, "rgba(255,255,255,0)",
  ]);
  ctx.fill();
});

defSpr("glow", 64, 64, (ctx) => {
  ctx.fillStyle = radGrad(ctx, 0, 0, 0, 0, 0, 32, [
    0, "rgba(255,255,255,1)",
    0.25, "rgba(255,255,255,.55)",
    0.6, "rgba(255,255,255,.14)",
    1, "rgba(255,255,255,0)",
  ]);
  ctx.fillRect(-32, -32, 64, 64);
});

defSpr("dotp", 16, 16, (ctx) => {
  ctx.fillStyle = radGrad(ctx, 0, 0, 0, 0, 0, 7.5, [0, "#fff", 0.6, "#fff", 1, "rgba(255,255,255,0)"]);
  ctx.fillRect(-8, -8, 16, 16);
});

defSpr("shard", 16, 16, (ctx) => {
  ctx.fillStyle = "#fff";
  ctx.beginPath();
  ctx.moveTo(0, -6.5);
  ctx.lineTo(5.5, 0);
  ctx.lineTo(0, 6.5);
  ctx.lineTo(-5.5, 0);
  ctx.closePath();
  ctx.fill();
});

defSpr("spark", 28, 28, (ctx, sc) => {
  ctx.save();
  ctx.shadowColor = "#fff";
  ctx.shadowBlur = 3 * sc;
  ctx.fillStyle = "#fff";
  starPoly(ctx, 0, 0, 4, 12, 2.2, 0);
  ctx.fill();
  ctx.restore();
});

defSpr("bub", 20, 20, (ctx) => {
  ctx.strokeStyle = "rgba(255,255,255,.88)";
  ctx.lineWidth = 1.3;
  poly(ctx, 0, 0, 6, 6.5, 0);
  ctx.stroke();
  ctx.fillStyle = "rgba(255,255,255,.18)";
  ctx.fill();
});

defSpr("px", 8, 8, (ctx) => {
  ctx.fillStyle = "#fff";
  ctx.fillRect(-3, -3, 6, 6);
});

defSpr("shaft", 64, 256, (ctx) => {
  const g = ctx.createLinearGradient(-32, 0, 32, 0);
  g.addColorStop(0, "rgba(255,255,255,0)");
  g.addColorStop(0.5, "rgba(255,255,255,1)");
  g.addColorStop(1, "rgba(255,255,255,0)");
  ctx.fillStyle = g;
  ctx.fillRect(-32, -128, 64, 256);
  ctx.globalCompositeOperation = "destination-in";
  ctx.fillStyle = linGrad(ctx, 0, -128, 0, 128, [
    0, "rgba(0,0,0,.9)",
    0.6, "rgba(0,0,0,.35)",
    1, "rgba(0,0,0,0)",
  ]);
  ctx.fillRect(-32, -128, 64, 256);
  ctx.globalCompositeOperation = "source-over";
});

defSpr("vig", 128, 128, (ctx) => {
  ctx.fillStyle = radGrad(ctx, 0, 0, 30, 0, 0, 90, [
    0, "rgba(4,8,10,0)",
    0.55, "rgba(4,8,10,.3)",
    1, "rgba(4,8,10,.92)",
  ]);
  ctx.fillRect(-64, -64, 128, 128);
});

defSpr("vigr", 128, 128, (ctx) => {
  ctx.fillStyle = radGrad(ctx, 0, 0, 34, 0, 0, 90, [
    0, "rgba(240,70,70,0)",
    0.6, "rgba(240,70,70,.3)",
    1, "rgba(220,38,38,.94)",
  ]);
  ctx.fillRect(-64, -64, 128, 128);
});

const makeShard = (c1, c2, sz) => (ctx, sc) => {
  softGlow(ctx, 0, 0, sz * 2.1, c1, 0.62);
  ctx.save();
  ctx.shadowColor = c1;
  ctx.shadowBlur = 4 * sc;
  ctx.beginPath();
  ctx.moveTo(0, -sz * 1.2);
  ctx.lineTo(sz * 0.92, 0);
  ctx.lineTo(0, sz * 1.2);
  ctx.lineTo(-sz * 0.92, 0);
  ctx.closePath();
  ctx.fillStyle = linGrad(ctx, -sz, -sz, sz, sz, [0, "#ffffff", 0.45, c1, 1, c2]);
  ctx.fill();
  ctx.restore();
  ctx.fillStyle = "rgba(255,255,255,.9)";
  circle(ctx, -sz * 0.18, -sz * 0.28, sz * 0.2);
  ctx.fill();
};

defSpr("gem0", 20, 20, makeShard("#2ee59d", "#059669", 4.6));
defSpr("gem1", 24, 24, makeShard("#5ce1e6", "#0284c7", 5.6));
defSpr("gem2", 30, 30, makeShard("#ffad36", "#ea580c", 7.2));

defSpr("heart", 34, 34, (ctx, sc) => {
  softGlow(ctx, 0, 0, 17, "#fb7185", 0.55);
  circle(ctx, 0, 0, 12);
  ctx.strokeStyle = "#ffd166";
  ctx.lineWidth = 1.6;
  ctx.stroke();
  ctx.save();
  ctx.shadowColor = "#fb7185";
  ctx.shadowBlur = 5 * sc;
  ctx.fillStyle = linGrad(ctx, 0, -6, 0, 8, [0, "#fda4af", 1, "#e11d48"]);
  ctx.beginPath();
  ctx.moveTo(0, 7);
  ctx.bezierCurveTo(-11, -1, -6, -9, 0, -3.5);
  ctx.bezierCurveTo(6, -9, 11, -1, 0, 7);
  ctx.fill();
  ctx.restore();
});

defSpr("magnet", 36, 36, (ctx, sc) => {
  softGlow(ctx, 0, 0, 18, "#2ee59d", 0.55);
  ctx.save();
  ctx.shadowColor = "#2ee59d";
  ctx.shadowBlur = 4 * sc;
  ctx.lineWidth = 5;
  ctx.lineCap = "round";
  ctx.strokeStyle = "#2ee59d";
  ctx.beginPath();
  ctx.arc(0, 1, 9, 0, Math.PI);
  ctx.moveTo(-9, 1);
  ctx.lineTo(-9, -8);
  ctx.moveTo(9, 1);
  ctx.lineTo(9, -8);
  ctx.stroke();
  ctx.restore();
});

defSpr("orb", 40, 40, (ctx, sc) => {
  softGlow(ctx, 0, 0, 20, "#5ce1e6", 0.75);
  poly(ctx, 0, 0, 8, 12, 0.2);
  ctx.fillStyle = radGrad(ctx, 0, 0, 0.5, 0, 0, 12, [0, "#ffffff", 0.5, "#ffd166", 1, "#06b6d4"]);
  ctx.fill();
  ctx.strokeStyle = "#fff";
  ctx.lineWidth = 1.3;
  ctx.stroke();
  ctx.save();
  ctx.shadowColor = "#fff";
  ctx.shadowBlur = 4 * sc;
  ctx.fillStyle = "#fff";
  starPoly(ctx, 0, 0, 4, 7, 2, 0);
  ctx.fill();
  ctx.restore();
});

defSpr("core", 28, 28, (ctx, sc) => {
  softGlow(ctx, 0, 0, 14, "#ffad36", 0.58);
  ctx.save();
  ctx.shadowColor = "#ffad36";
  ctx.shadowBlur = 4 * sc;
  poly(ctx, 0, 0, 6, 9, -Math.PI / 2);
  ctx.fillStyle = linGrad(ctx, 0, -9, 0, 9, [0, "#fff7ed", 0.45, "#ffad36", 1, "#ea580c"]);
  ctx.fill();
  ctx.strokeStyle = "#fde68a";
  ctx.lineWidth = 1.1;
  ctx.stroke();
  ctx.restore();
});

defSpr("chest", 48, 44, (ctx, sc) => {
  softGlow(ctx, 0, 0, 24, "#ffd166", 0.6);
  ctx.save();
  ctx.shadowColor = "#ffd166";
  ctx.shadowBlur = 6 * sc;
  roundRectPath(ctx, -14, -10, 28, 22, 4);
  ctx.fillStyle = linGrad(ctx, 0, -10, 0, 12, [0, "#2ee59d", 1, "#0f766e"]);
  ctx.fill();
  ctx.strokeStyle = "#ffd166";
  ctx.lineWidth = 2;
  ctx.stroke();
  ctx.fillStyle = "#ffd166";
  ctx.fillRect(-14, -2, 28, 3);
  roundRectPath(ctx, -3.5, -5, 7, 8, 2);
  ctx.fill();
  ctx.restore();
});

defSpr("joy", 120, 120, (ctx) => {
  poly(ctx, 0, 0, 8, 52, Math.PI / 8);
  ctx.fillStyle = "rgba(255,173,54,.06)";
  ctx.fill();
  ctx.strokeStyle = "rgba(255,209,102,.38)";
  ctx.lineWidth = 1.6;
  ctx.stroke();
  circle(ctx, 0, 0, 32);
  ctx.strokeStyle = "rgba(92,225,230,.2)";
  ctx.lineWidth = 1;
  ctx.stroke();
});

defSpr("knob", 52, 52, (ctx) => {
  softGlow(ctx, 0, 0, 26, "#ffad36", 0.45);
  poly(ctx, 0, 0, 6, 16, -Math.PI / 2);
  ctx.fillStyle = radGrad(ctx, -3, -4, 1, 0, 0, 16, [0, "rgba(255,250,235,.95)", 1, "rgba(255,173,54,.65)"]);
  ctx.fill();
  ctx.strokeStyle = "rgba(255,255,255,.88)";
  ctx.lineWidth = 1.3;
  ctx.stroke();
});

// Arena Props
defSpr("d_rack", 88, 96, (ctx) => {
  roundRectPath(ctx, -26, -38, 52, 76, 6);
  ctx.fillStyle = linGrad(ctx, 0, -38, 0, 38, [0, "#1e3438", 1, "#0d1a1d"]);
  ctx.fill();
  ctx.strokeStyle = "rgba(92,225,230,.25)";
  ctx.lineWidth = 1.5;
  ctx.stroke();
  ctx.fillStyle = "rgba(46,229,157,.55)";
  for (let i = 0; i < 4; i++) ctx.fillRect(-16, -24 + i * 13, 32, 3);
});

defSpr("d_monolith", 84, 92, (ctx) => {
  poly(ctx, 0, 4, 6, 34, -Math.PI / 2);
  ctx.fillStyle = linGrad(ctx, 0, -32, 0, 36, [0, "#243034", 1, "#0f171a"]);
  ctx.fill();
  ctx.strokeStyle = "rgba(255,173,54,.35)";
  ctx.lineWidth = 1.6;
  ctx.stroke();
});

defSpr("d_crystal", 76, 76, (ctx) => {
  for (const [ox, oy, h, rot] of [[-10, 8, 26, -0.25], [10, 10, 22, 0.3], [0, 4, 34, 0]]) {
    ctx.save();
    ctx.translate(ox, oy);
    ctx.rotate(rot);
    ctx.beginPath();
    ctx.moveTo(0, -h);
    ctx.lineTo(8, 0);
    ctx.lineTo(5, 12);
    ctx.lineTo(-5, 12);
    ctx.lineTo(-8, 0);
    ctx.closePath();
    ctx.fillStyle = linGrad(ctx, 0, -h, 0, 12, [0, "#6fffe9", 1, "#0f766e"]);
    ctx.fill();
    ctx.restore();
  }
});

defSpr("d_node", 64, 64, (ctx) => {
  poly(ctx, 0, 0, 6, 22, 0);
  ctx.fillStyle = "#13262a";
  ctx.fill();
  ctx.strokeStyle = "rgba(92,225,230,.26)";
  ctx.lineWidth = 1.4;
  ctx.stroke();
  circle(ctx, 0, 0, 6);
  ctx.fillStyle = "rgba(255,173,54,.35)";
  ctx.fill();
});

defSpr("streak", 64, 16, (ctx) => {
  ctx.fillStyle = radGrad(ctx, 0, 0, 0, 0, 0, 30, [
    0, "rgba(255,255,255,1)",
    0.4, "rgba(255,255,255,.6)",
    1, "rgba(255,255,255,0)",
  ]);
  ctx.save();
  ctx.scale(1, 0.2);
  circle(ctx, 0, 0, 30);
  ctx.fill();
  ctx.restore();
});

defSpr("flare", 96, 96, (ctx) => {
  ctx.fillStyle = radGrad(ctx, 0, 0, 0, 0, 0, 22, [
    0, "rgba(255,255,255,1)",
    0.35, "rgba(255,255,255,.5)",
    1, "rgba(255,255,255,0)",
  ]);
  circle(ctx, 0, 0, 22);
  ctx.fill();
  for (let i = 0; i < 4; i++) {
    ctx.save();
    ctx.rotate((i * Math.PI) / 2);
    const g = ctx.createLinearGradient(0, 0, 46, 0);
    g.addColorStop(0, "rgba(255,255,255,.95)");
    g.addColorStop(1, "rgba(255,255,255,0)");
    ctx.fillStyle = g;
    ctx.beginPath();
    ctx.moveTo(0, -3.2);
    ctx.lineTo(46, 0);
    ctx.lineTo(0, 3.2);
    ctx.closePath();
    ctx.fill();
    ctx.restore();
  }
});

defSpr("pillar", 64, 256, (ctx) => {
  const g = ctx.createLinearGradient(-32, 0, 32, 0);
  g.addColorStop(0, "rgba(255,255,255,0)");
  g.addColorStop(0.3, "rgba(255,255,255,.35)");
  g.addColorStop(0.5, "rgba(255,255,255,1)");
  g.addColorStop(0.7, "rgba(255,255,255,.35)");
  g.addColorStop(1, "rgba(255,255,255,0)");
  ctx.fillStyle = g;
  ctx.fillRect(-32, -128, 64, 256);
  ctx.globalCompositeOperation = "destination-in";
  ctx.fillStyle = linGrad(ctx, 0, -128, 0, 128, [
    0, "rgba(0,0,0,0)",
    0.35, "rgba(0,0,0,.55)",
    0.9, "rgba(0,0,0,1)",
    1, "rgba(0,0,0,.6)",
  ]);
  ctx.fillRect(-32, -128, 64, 256);
  ctx.globalCompositeOperation = "source-over";
});

defSpr("ray", 32, 128, (ctx) => {
  const g = ctx.createLinearGradient(0, 64, 0, -64);
  g.addColorStop(0, "rgba(255,255,255,.9)");
  g.addColorStop(1, "rgba(255,255,255,0)");
  ctx.fillStyle = g;
  ctx.beginPath();
  ctx.moveTo(-2, 64);
  ctx.lineTo(-13, -64);
  ctx.lineTo(13, -64);
  ctx.lineTo(2, 64);
  ctx.closePath();
  ctx.fill();
});

defSpr("ink", 64, 64, (ctx) => {
  for (const [ox, oy, r] of [[0, 0, 20], [-10, -7, 12], [11, -5, 13], [8, 10, 11], [-9, 9, 11]]) {
    ctx.fillStyle = radGrad(ctx, ox, oy, 0, ox, oy, r, [
      0, "rgba(255,255,255,.95)",
      0.6, "rgba(255,255,255,.65)",
      1, "rgba(255,255,255,0)",
    ]);
    poly(ctx, ox, oy, 6, r, 0.3);
    ctx.fill();
  }
});

defSpr("rune", 128, 128, (ctx, sc) => {
  ctx.save();
  ctx.shadowColor = "#fff";
  ctx.shadowBlur = 4 * sc;
  ctx.strokeStyle = "#fff";
  ctx.lineWidth = 2.2;
  poly(ctx, 0, 0, 6, 56, 0);
  ctx.stroke();
  ctx.lineWidth = 1.2;
  circle(ctx, 0, 0, 44);
  ctx.stroke();
  poly(ctx, 0, 0, 6, 28, Math.PI / 6);
  ctx.stroke();
  for (let i = 0; i < 12; i++) {
    const a = (i / 12) * TAU;
    ctx.beginPath();
    ctx.moveTo(Math.cos(a) * 45, Math.sin(a) * 45);
    ctx.lineTo(Math.cos(a) * 52, Math.sin(a) * 52);
    ctx.stroke();
  }
  ctx.restore();
});

defSpr("shadow", 64, 32, (ctx) => {
  ctx.save();
  ctx.scale(1, 0.5);
  ctx.fillStyle = radGrad(ctx, 0, 0, 0, 0, 0, 30, [
    0, "rgba(2,8,10,.65)",
    0.6, "rgba(2,8,10,.36)",
    1, "rgba(2,8,10,0)",
  ]);
  circle(ctx, 0, 0, 30);
  ctx.fill();
  ctx.restore();
});

const NUM_FONT = '900 italic 20px "Avenir Next", "Arial Black", "Helvetica Neue", "PingFang SC", sans-serif';
for (let d = 0; d < 10; d++) {
  defSpr("n" + d, 16, 24, (ctx) => {
    ctx.font = NUM_FONT;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.lineJoin = "round";
    ctx.lineWidth = 4;
    ctx.strokeStyle = "#081214";
    ctx.strokeText(String(d), 0, 1);
    ctx.fillStyle = "#ffffff";
    ctx.fillText(String(d), 0, 1);
  });
  defSpr("c" + d, 18, 26, (ctx) => {
    ctx.font = NUM_FONT.replace("20px", "22px");
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.lineJoin = "round";
    ctx.lineWidth = 4.6;
    ctx.strokeStyle = "#3d1206";
    ctx.strokeText(String(d), 0, 1);
    ctx.fillStyle = linGrad(ctx, 0, -9, 0, 10, [0, "#fffbeb", 0.52, "#ffad36", 1, "#f06449"]);
    ctx.fillText(String(d), 0, 1);
  });
}

export const ART_STATE = { sheet: false };

export function bindSpriteSheet(img) {
  const aliases = {
    h_hone: ["h_hone", "h_hone_joy", "h_hone_hurt"],
    h_rubin: ["h_rubin", "h_rubin_joy", "h_rubin_hurt"],
    h_photon: ["h_photon", "h_photon_joy", "h_photon_hurt"],
    h_quant: ["h_quant", "h_quant_joy", "h_quant_hurt"],
    h_gold: ["h_gold", "h_gold_joy", "h_gold_hurt"],
  };
  for (const [key, rect] of Object.entries(SHEET_FRAMES)) {
    const [sx, sy, sw, sh, lw, lh] = rect;
    const drawFn = (ctx) => {
      ctx.drawImage(img, sx, sy, sw, sh, -lw / 2, -lh / 2, lw, lh);
    };
    const targets = aliases[key] || [key];
    for (const name of targets) {
      const idx = SPR_IDX[name];
      if (idx != null) {
        const def = SPRITE_DEFS[idx];
        def.w = Math.ceil(lw) + 2;
        def.h = Math.ceil(lh) + 2;
        def.f = drawFn;
        def.img = 1;
        def.lw = lw;
        def.lh = lh;
      }
    }
  }
  ART_STATE.sheet = true;
}

export function loadImage(url, timeoutMs = 6000) {
  return new Promise((resolve) => {
    const img = new Image();
    let done = false;
    const finish = (v) => {
      if (!done) {
        done = true;
        resolve(v);
      }
    };
    img.onload = () => finish(img.naturalWidth > 1 ? img : null);
    img.onerror = () => finish(null);
    setTimeout(() => finish(null), timeoutMs);
    img.src = url;
  });
}

export function buildAtlas(scale, size = 2048) {
  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext("2d");
  const frames = new Array(SPRITE_DEFS.length);
  const order = SPRITE_DEFS.map((_, i) => i).sort((a, b) => SPRITE_DEFS[b].h - SPRITE_DEFS[a].h);
  let x = 2, y = 2, rowH = 0;

  for (const idx of order) {
    const def = SPRITE_DEFS[idx];
    if (def.n.startsWith("P_")) {
      frames[idx] = { x: 0, y: 0, w: 1, h: 1, lw: def.lw || def.w, lh: def.lh || def.h, u0: 0, v0: 0, u1: 0, v1: 0 };
      continue;
    }
    const pw = Math.ceil(def.w * scale);
    const ph = Math.ceil(def.h * scale);
    if (x + pw + 2 > size) {
      x = 2;
      y += rowH + 2;
      rowH = 0;
    }
    if (ph > rowH) rowH = ph;
    ctx.save();
    ctx.beginPath();
    ctx.rect(x, y, pw, ph);
    ctx.clip();
    ctx.translate(x + pw / 2, y + ph / 2);
    ctx.scale(scale, scale);
    def.f(ctx, scale);
    ctx.restore();
    frames[idx] = {
      x, y, w: pw, h: ph,
      lw: def.lw || def.w,
      lh: def.lh || def.h,
      u0: (x + 0.5) / size,
      v0: (y + 0.5) / size,
      u1: (x + pw - 0.5) / size,
      v1: (y + ph - 0.5) / size,
    };
    x += pw + 2;
  }
  return { canvas, frames, used: y + rowH, size };
}

export function drawSprite2D(ctx, name, x, y, scale, rot = 0, flipX = false) {
  const def = SPRITE_DEFS[SPR_IDX[name]];
  if (!def) return;
  ctx.save();
  ctx.translate(x, y);
  if (rot) ctx.rotate(rot);
  ctx.scale(scale * (flipX ? -1 : 1), scale);
  def.f(ctx, scale);
  ctx.restore();
}

export function makeFallbackFloor(size = 512) {
  const c = document.createElement("canvas");
  c.width = size;
  c.height = size;
  const ctx = c.getContext("2d");
  ctx.fillStyle = "#0b1416";
  ctx.fillRect(0, 0, size, size);
  ctx.strokeStyle = "rgba(92,225,230,.055)";
  ctx.lineWidth = 1;
  for (let i = 0; i <= size; i += 64) {
    ctx.beginPath();
    ctx.moveTo(i, 0);
    ctx.lineTo(i, size);
    ctx.moveTo(0, i);
    ctx.lineTo(size, i);
    ctx.stroke();
  }
  ctx.strokeStyle = "rgba(255,173,54,.045)";
  ctx.lineWidth = 1.5;
  for (let i = 0; i < 12; i++) {
    const x = ((i * 97) % 8) * 64 + 32;
    const y = ((i * 53) % 8) * 64 + 32;
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(x + 48, y);
    ctx.lineTo(x + 64, y + 16);
    ctx.stroke();
  }
  return c;
}

export function makeQuantOverlay(size = 256) {
  const c = document.createElement("canvas");
  c.width = size;
  c.height = size;
  const ctx = c.getContext("2d");
  const imgData = ctx.createImageData(size, size);
  const d = imgData.data;
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const fx = (x / size) * TAU;
      const fy = (y / size) * TAU;
      const w1 = Math.sin(fx * 2 + 1.1 * Math.sin(fy * 2 + 0.4));
      const w2 = Math.sin(fy * 3 + 1.1 * Math.sin(fx * 2 + 1.5));
      const grid = Math.max(Math.exp(-Math.abs(Math.sin(fx * 4)) * 18), Math.exp(-Math.abs(Math.sin(fy * 4)) * 18));
      const v = Math.min(1, (Math.exp(-Math.abs(w1) * 10) * 0.55 + Math.exp(-Math.abs(w2) * 10) * 0.45 + grid * 0.35) * 0.58);
      const idx = (y * size + x) * 4;
      d[idx] = 85 + 140 * v;
      d[idx + 1] = 215 + 35 * v;
      d[idx + 2] = 210 + 40 * v;
      d[idx + 3] = Math.round(255 * Math.min(1, v * v * 0.48));
    }
  }
  ctx.putImageData(imgData, 0, 0);
  return c;
}

// Crisp Weapon, Passive & UI Icon Drawers (64x64 coordinate space)
const drawSprFit = (ctx, sc, name, targetSz, ox = 0, oy = 0, rot = 0) => {
  const def = SPRITE_DEFS[SPR_IDX[name]];
  if (!def) return;
  const k = targetSz / Math.max(def.lw || def.w, def.lh || def.h);
  ctx.save();
  ctx.translate(ox, oy);
  if (rot) ctx.rotate(rot);
  ctx.scale(k, k);
  def.f(ctx, sc * k);
  ctx.restore();
};

const strokeStyle = (ctx, lw = 4.2, col = "#f3f7f6") => {
  ctx.strokeStyle = col;
  ctx.lineWidth = lw;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
};

export const ICON_DRAWERS = {
  hone_logo: (ctx) => {
    // Iconic HONE Whetstone + Blade mark
    ctx.fillStyle = linGrad(ctx, -18, 8, 16, -22, [0, "#f15b08", 1, "#ffad36"]);
    ctx.beginPath();
    ctx.moveTo(-16, 4);
    ctx.lineTo(4, -22);
    ctx.lineTo(18, -14);
    ctx.lineTo(-2, 12);
    ctx.closePath();
    ctx.fill();
    ctx.fillStyle = linGrad(ctx, -22, -8, 22, 18, [0, "#94a3b8", 0.5, "#334155", 1, "#0f172a"]);
    ctx.beginPath();
    ctx.moveTo(-22, -6);
    ctx.lineTo(20, 12);
    ctx.lineTo(20, 21);
    ctx.lineTo(-22, 6);
    ctx.closePath();
    ctx.fill();
    ctx.fillStyle = "#ffd166";
    ctx.beginPath();
    ctx.moveTo(-18, -10);
    ctx.lineTo(22, 8);
    ctx.lineTo(18, 11);
    ctx.lineTo(-22, -7);
    ctx.closePath();
    ctx.fill();
  },
  blade: (ctx, sc) => {
    softGlow(ctx, 0, 0, 28, "#ffad36", 0.42);
    drawSprFit(ctx, sc, "blade", 52, 0, 0, -0.65);
  },
  wafer: (ctx, sc) => {
    softGlow(ctx, 0, 0, 28, "#5ce1e6", 0.42);
    drawSprFit(ctx, sc, "wafer", 54, 0, 0, -0.2);
  },
  moat: (ctx, sc) => {
    ctx.strokeStyle = "rgba(255,217,125,.5)";
    ctx.lineWidth = 1.8;
    circle(ctx, 0, 0, 19);
    ctx.stroke();
    for (let i = 0; i < 3; i++) {
      const a = (i / 3) * TAU - 0.5;
      drawSprFit(ctx, sc, "moat", 22, Math.cos(a) * 19, Math.sin(a) * 19);
    }
  },
  arc: (ctx, sc) => {
    softGlow(ctx, 0, 0, 28, "#6fffe9", 0.45);
    ctx.save();
    ctx.shadowColor = "#6fffe9";
    ctx.shadowBlur = 6 * sc;
    ctx.fillStyle = linGrad(ctx, 0, -24, 0, 24, [0, "#ffffff", 0.5, "#6fffe9", 1, "#06b6d4"]);
    ctx.beginPath();
    ctx.moveTo(6, -25);
    ctx.lineTo(-14, 2);
    ctx.lineTo(-2, 2);
    ctx.lineTo(-8, 25);
    ctx.lineTo(15, -5);
    ctx.lineTo(2, -5);
    ctx.closePath();
    ctx.fill();
    ctx.restore();
  },
  pulse: (ctx) => {
    softGlow(ctx, 0, 0, 28, "#2ee59d", 0.36);
    ctx.strokeStyle = "#2ee59d";
    for (let i = 0; i < 3; i++) {
      ctx.globalAlpha = 1 - i * 0.28;
      ctx.lineWidth = 3.4 - i * 0.6;
      poly(ctx, 0, 0, 6, 8 + i * 8, 0);
      ctx.stroke();
    }
    ctx.globalAlpha = 1;
    ctx.fillStyle = "#d1fae5";
    circle(ctx, 0, 0, 3.8);
    ctx.fill();
  },
  mine: (ctx, sc) => drawSprFit(ctx, sc, "mine", 54),
  laser: (ctx, sc) => {
    softGlow(ctx, -6, 6, 26, "#ffd166", 0.55);
    ctx.save();
    ctx.rotate(-0.6);
    ctx.fillStyle = linGrad(ctx, 0, -8, 0, 8, [0, "rgba(255,209,102,0)", 0.5, "#fffbeb", 1, "rgba(255,209,102,0)"]);
    ctx.fillRect(-4, -8, 36, 16);
    ctx.restore();
    poly(ctx, -10, 9, 4, 9, 0);
    ctx.fillStyle = linGrad(ctx, -18, 0, -2, 18, [0, "#6fffe9", 1, "#ffad36"]);
    ctx.fill();
    ctx.strokeStyle = "#fff";
    ctx.lineWidth = 1.5;
    ctx.stroke();
  },
  vortex: (ctx, sc) => {
    softGlow(ctx, 0, 0, 28, "#a78bfa", 0.42);
    drawSprFit(ctx, sc, "swirl", 56);
  },
  haste: (ctx) => {
    strokeStyle(ctx, 4.2, "#ffad36");
    for (let i = 0; i < 3; i++) {
      ctx.globalAlpha = 0.45 + i * 0.27;
      ctx.beginPath();
      ctx.moveTo(-18 + i * 11, -13);
      ctx.lineTo(-6 + i * 11, 0);
      ctx.lineTo(-18 + i * 11, 13);
      ctx.stroke();
    }
    ctx.globalAlpha = 1;
  },
  area: (ctx) => {
    strokeStyle(ctx, 3.2, "#5ce1e6");
    poly(ctx, 0, 0, 6, 21, 0);
    ctx.stroke();
    ctx.globalAlpha = 0.55;
    poly(ctx, 0, 0, 6, 12, 0);
    ctx.stroke();
    ctx.globalAlpha = 1;
  },
  margin: (ctx) => {
    ctx.fillStyle = linGrad(ctx, 0, -22, 0, 22, [0, "#fef3c7", 1, "#d97706"]);
    ctx.strokeStyle = "#fff";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(0, -22);
    ctx.lineTo(18, -14);
    ctx.lineTo(16, 6);
    ctx.quadraticCurveTo(0, 22, 0, 24);
    ctx.quadraticCurveTo(0, 22, -16, 6);
    ctx.lineTo(-18, -14);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  },
  sharpe: (ctx) => {
    strokeStyle(ctx, 3, "#6fffe9");
    circle(ctx, 0, 0, 17);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(-23, 0);
    ctx.lineTo(-11, 0);
    ctx.moveTo(11, 0);
    ctx.lineTo(23, 0);
    ctx.moveTo(0, -23);
    ctx.lineTo(0, -11);
    ctx.moveTo(0, 11);
    ctx.lineTo(0, 23);
    ctx.stroke();
    ctx.fillStyle = "#ffad36";
    circle(ctx, 0, 0, 4.5);
    ctx.fill();
  },
  siphon: (ctx, sc) => drawSprFit(ctx, sc, "magnet", 52),
  leverage: (ctx, sc) => {
    ctx.save();
    ctx.shadowColor = "#ff6b6b";
    ctx.shadowBlur = 5 * sc;
    strokeStyle(ctx, 4.5, "#ff6b6b");
    ctx.beginPath();
    ctx.moveTo(-20, 14);
    ctx.lineTo(-6, 2);
    ctx.lineTo(3, 9);
    ctx.lineTo(19, -14);
    ctx.moveTo(9, -14);
    ctx.lineTo(19, -14);
    ctx.lineTo(19, -4);
    ctx.stroke();
    ctx.restore();
  },
  compound: (ctx) => {
    strokeStyle(ctx, 3.2, "#ffd166");
    ctx.beginPath();
    ctx.moveTo(-12, -20);
    ctx.lineTo(12, -20);
    ctx.lineTo(-12, 20);
    ctx.lineTo(12, 20);
    ctx.closePath();
    ctx.stroke();
    ctx.fillStyle = "#fde68a";
    ctx.beginPath();
    ctx.moveTo(-6, 17);
    ctx.lineTo(6, 17);
    ctx.lineTo(0, 7);
    ctx.fill();
  },
  momentum: (ctx, sc) => {
    ctx.save();
    ctx.shadowColor = "#a78bfa";
    ctx.shadowBlur = 5 * sc;
    ctx.fillStyle = linGrad(ctx, -18, 16, 18, -18, [0, "#7c3aed", 1, "#ddd6fe"]);
    ctx.beginPath();
    ctx.moveTo(18, -18);
    ctx.lineTo(-18, -2);
    ctx.lineTo(-4, 4);
    ctx.lineTo(2, 18);
    ctx.closePath();
    ctx.fill();
    ctx.restore();
  },
  dividend: (ctx, sc) => {
    ctx.save();
    ctx.shadowColor = "#fb7185";
    ctx.shadowBlur = 5 * sc;
    ctx.fillStyle = linGrad(ctx, 0, -14, 0, 18, [0, "#fda4af", 1, "#e11d48"]);
    ctx.beginPath();
    ctx.moveTo(0, 20);
    ctx.bezierCurveTo(-30, -2, -15, -24, 0, -9);
    ctx.bezierCurveTo(15, -24, 30, -2, 0, 20);
    ctx.fill();
    ctx.restore();
    ctx.fillStyle = "#fff";
    ctx.fillRect(-2, -6, 4, 14);
    ctx.fillRect(-7, -1, 14, 4);
  },
  b_might: (ctx, sc) => {
    ICON_DRAWERS.leverage(ctx, sc);
    ctx.fillStyle = "#ffd166";
    starPoly(ctx, -12, -12, 4, 7, 2, 0);
    ctx.fill();
  },
  b_heal: (ctx, sc) => ICON_DRAWERS.dividend(ctx, sc),
  b_core: (ctx, sc) => drawSprFit(ctx, sc, "core", 52),
  core: (ctx, sc) => drawSprFit(ctx, sc, "core", 54),
  chest: (ctx, sc) => drawSprFit(ctx, sc, "chest", 58),
  nuke: (ctx, sc) => drawSprFit(ctx, sc, "orb", 54),
  ui_up: (ctx) => {
    strokeStyle(ctx);
    ctx.beginPath();
    ctx.moveTo(0, 20);
    ctx.lineTo(0, -16);
    ctx.moveTo(-14, -3);
    ctx.lineTo(0, -18);
    ctx.lineTo(14, -3);
    ctx.stroke();
    strokeStyle(ctx, 4.2, "rgba(243,247,246,.45)");
    ctx.beginPath();
    ctx.moveTo(-18, 22);
    ctx.lineTo(18, 22);
    ctx.stroke();
  },
  ui_agent: (ctx, sc) => drawSprFit(ctx, sc, "h_hone", 58),
  ui_book: (ctx) => {
    strokeStyle(ctx);
    ctx.beginPath();
    ctx.moveTo(0, -14);
    ctx.quadraticCurveTo(-11, -20, -22, -15);
    ctx.lineTo(-22, 16);
    ctx.quadraticCurveTo(-11, 11, 0, 17);
    ctx.quadraticCurveTo(11, 11, 22, 16);
    ctx.lineTo(22, -15);
    ctx.quadraticCurveTo(11, -20, 0, -14);
    ctx.lineTo(0, 17);
    ctx.stroke();
  },
  ui_medal: (ctx) => {
    strokeStyle(ctx);
    circle(ctx, 0, 6, 14);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(-9, -6);
    ctx.lineTo(-14, -22);
    ctx.moveTo(9, -6);
    ctx.lineTo(14, -22);
    ctx.stroke();
    ctx.fillStyle = "#ffd166";
    starPoly(ctx, 0, 6, 5, 7.5, 3.2, -Math.PI / 2);
    ctx.fill();
  },
  ui_rank: (ctx) => {
    strokeStyle(ctx);
    ctx.beginPath();
    ctx.moveTo(-22, 20);
    ctx.lineTo(-22, 2);
    ctx.lineTo(-8, 2);
    ctx.lineTo(-8, -14);
    ctx.lineTo(8, -14);
    ctx.lineTo(8, 8);
    ctx.lineTo(22, 8);
    ctx.lineTo(22, 20);
    ctx.closePath();
    ctx.stroke();
  },
  ui_gear: (ctx) => {
    strokeStyle(ctx, 4);
    circle(ctx, 0, 0, 8);
    ctx.stroke();
    for (let i = 0; i < 8; i++) {
      const a = (i / 8) * TAU;
      ctx.beginPath();
      ctx.moveTo(Math.cos(a) * 15, Math.sin(a) * 15);
      ctx.lineTo(Math.cos(a) * 21, Math.sin(a) * 21);
      ctx.stroke();
    }
    circle(ctx, 0, 0, 15);
    ctx.stroke();
  },
  ui_pause: (ctx) => {
    ctx.fillStyle = "#f3f7f6";
    roundRectPath(ctx, -14, -17, 10, 34, 3);
    ctx.fill();
    roundRectPath(ctx, 4, -17, 10, 34, 3);
    ctx.fill();
  },
  ui_cal: (ctx) => {
    strokeStyle(ctx);
    roundRectPath(ctx, -19, -15, 38, 36, 6);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(-19, -4);
    ctx.lineTo(19, -4);
    ctx.moveTo(-9, -22);
    ctx.lineTo(-9, -12);
    ctx.moveTo(9, -22);
    ctx.lineTo(9, -12);
    ctx.stroke();
    ctx.fillStyle = "#ffad36";
    starPoly(ctx, 0, 9, 5, 7.5, 3.2, -Math.PI / 2);
    ctx.fill();
  },
  ui_home: (ctx) => {
    strokeStyle(ctx);
    ctx.beginPath();
    ctx.moveTo(-21, -1);
    ctx.lineTo(0, -20);
    ctx.lineTo(21, -1);
    ctx.moveTo(-14, -6);
    ctx.lineTo(-14, 19);
    ctx.lineTo(14, 19);
    ctx.lineTo(14, -6);
    ctx.stroke();
  },
  ui_back: (ctx) => {
    strokeStyle(ctx, 4.4);
    ctx.beginPath();
    ctx.moveTo(16, 0);
    ctx.lineTo(-14, 0);
    ctx.moveTo(-3, -12);
    ctx.lineTo(-15, 0);
    ctx.lineTo(-3, 12);
    ctx.stroke();
  },
  ui_retry: (ctx) => {
    strokeStyle(ctx);
    ctx.beginPath();
    ctx.arc(0, 1, 17, -0.4, TAU - 1.5);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(4, -21);
    ctx.lineTo(16, -15);
    ctx.lineTo(8, -4);
    ctx.stroke();
  },
  ui_save: (ctx) => {
    strokeStyle(ctx);
    ctx.beginPath();
    ctx.moveTo(0, -20);
    ctx.lineTo(0, 8);
    ctx.moveTo(-11, -2);
    ctx.lineTo(0, 9);
    ctx.lineTo(11, -2);
    ctx.moveTo(-19, 19);
    ctx.lineTo(19, 19);
    ctx.stroke();
  },
  ui_copy: (ctx) => {
    strokeStyle(ctx, 4);
    roundRectPath(ctx, -8, -14, 24, 28, 4);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(-12, 8);
    ctx.lineTo(-16, 8);
    ctx.arcTo(-20, 8, -20, 4, 4);
    ctx.lineTo(-20, -18);
    ctx.arcTo(-20, -22, -16, -22, 4);
    ctx.lineTo(4, -22);
    ctx.stroke();
  },
  ui_share: (ctx) => {
    strokeStyle(ctx);
    ctx.beginPath();
    ctx.moveTo(-18, 2);
    ctx.lineTo(20, -18);
    ctx.lineTo(9, 20);
    ctx.lineTo(1, 6);
    ctx.closePath();
    ctx.moveTo(1, 6);
    ctx.lineTo(20, -18);
    ctx.stroke();
  },
  ui_pin: (ctx) => {
    strokeStyle(ctx, 4, "#ffd166");
    roundRectPath(ctx, -13, -3, 26, 20, 4);
    ctx.stroke();
    ctx.beginPath();
    ctx.arc(0, -4, 8, Math.PI, 0);
    ctx.stroke();
  },
  ui_x: (ctx) => {
    strokeStyle(ctx, 5);
    ctx.beginPath();
    ctx.moveTo(-14, -14);
    ctx.lineTo(14, 14);
    ctx.moveTo(14, -14);
    ctx.lineTo(-14, 14);
    ctx.stroke();
  },
  ui_check: (ctx) => {
    strokeStyle(ctx, 6, "#2ee59d");
    ctx.beginPath();
    ctx.moveTo(-16, 1);
    ctx.lineTo(-4, 13);
    ctx.lineTo(17, -12);
    ctx.stroke();
  },
};

export function renderIconCanvas(id, size = 96) {
  const c = document.createElement("canvas");
  c.width = size;
  c.height = size;
  const ctx = c.getContext("2d");
  ctx.translate(size / 2, size / 2);
  ctx.scale(size / 64, size / 64);
  if (ICON_DRAWERS[id]) {
    ICON_DRAWERS[id](ctx, size / 64);
  } else if (SPR_IDX[id] != null) {
    const def = SPRITE_DEFS[SPR_IDX[id]];
    const k = 56 / Math.max(def.lw || def.w, def.lh || def.h);
    ctx.scale(k, k);
    def.f(ctx, (size / 64) * k);
  }
  return c;
}

// WebGL 4096-Quad Sprite Batcher + Canvas 2D Fallback
const VERT_SRC =
  "attribute vec2 aPos;attribute vec2 aUV;attribute vec4 aCol;attribute vec2 aFx;uniform vec4 uView;varying vec2 vUV;varying vec4 vCol;varying vec2 vFx;void main(){vUV=aUV;vCol=aCol;vFx=aFx;gl_Position=vec4(aPos*uView.xy+uView.zw,0.,1.);}";
const FRAG_SRC =
  "precision mediump float;uniform sampler2D uTex;varying vec2 vUV;varying vec4 vCol;varying vec2 vFx;void main(){vec4 t=texture2D(uTex,vUV);vec3 rgb=mix(t.rgb*vCol.rgb,vec3(t.a),vFx.y);gl_FragColor=vec4(rgb*vCol.a,t.a*vCol.a*(1.-vFx.x));}";

export function createRenderer(canvas, atlas, tiles, force2D = false) {
  if (!force2D) {
    try {
      const glR = createWebGLRenderer(canvas, atlas, tiles);
      if (glR) return glR;
    } catch {
      // Fall back to Canvas 2D cleanly
    }
  }
  return createCanvas2DRenderer(canvas, atlas, tiles);
}

function createWebGLRenderer(canvas, atlas, tiles) {
  const opts = { alpha: false, antialias: false, depth: false, stencil: false, premultipliedAlpha: true, preserveDrawingBuffer: false };
  const gl = canvas.getContext("webgl", opts) || canvas.getContext("experimental-webgl", opts);
  if (!gl) return null;

  const frames = atlas.frames;
  const MAX_QUADS = 4096;
  const vbufData = new Float32Array(MAX_QUADS * 4 * 10);
  const ibufData = new Uint16Array(MAX_QUADS * 6);
  for (let i = 0; i < MAX_QUADS; i++) {
    const ii = i * 6;
    const vi = i * 4;
    ibufData[ii] = vi;
    ibufData[ii + 1] = vi + 1;
    ibufData[ii + 2] = vi + 2;
    ibufData[ii + 3] = vi;
    ibufData[ii + 4] = vi + 2;
    ibufData[ii + 5] = vi + 3;
  }

  const compile = (type, src) => {
    const sh = gl.createShader(type);
    gl.shaderSource(sh, src);
    gl.compileShader(sh);
    if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw new Error("shader");
    return sh;
  };

  const makeTex = (source, repeat) => {
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, true);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, source);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, repeat ? gl.REPEAT : gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, repeat ? gl.REPEAT : gl.CLAMP_TO_EDGE);
    return tex;
  };

  const prog = gl.createProgram();
  gl.attachShader(prog, compile(gl.VERTEX_SHADER, VERT_SRC));
  gl.attachShader(prog, compile(gl.FRAGMENT_SHADER, FRAG_SRC));
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) throw new Error("link");
  gl.useProgram(prog);

  const vbo = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
  gl.bufferData(gl.ARRAY_BUFFER, vbufData.byteLength, gl.DYNAMIC_DRAW);

  const ibo = gl.createBuffer();
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, ibo);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, ibufData, gl.STATIC_DRAW);

  const stride = 40;
  const locPos = gl.getAttribLocation(prog, "aPos");
  const locUV = gl.getAttribLocation(prog, "aUV");
  const locCol = gl.getAttribLocation(prog, "aCol");
  const locFx = gl.getAttribLocation(prog, "aFx");
  gl.enableVertexAttribArray(locPos);
  gl.vertexAttribPointer(locPos, 2, gl.FLOAT, false, stride, 0);
  gl.enableVertexAttribArray(locUV);
  gl.vertexAttribPointer(locUV, 2, gl.FLOAT, false, stride, 8);
  gl.enableVertexAttribArray(locCol);
  gl.vertexAttribPointer(locCol, 4, gl.FLOAT, false, stride, 16);
  gl.enableVertexAttribArray(locFx);
  gl.vertexAttribPointer(locFx, 2, gl.FLOAT, false, stride, 32);

  const uView = gl.getUniformLocation(prog, "uView");
  const uTex = gl.getUniformLocation(prog, "uTex");
  gl.uniform1i(uTex, 0);

  const atlasTex = makeTex(atlas.canvas, false);
  const tileTexs = tiles.map((t) => makeTex(t, true));

  gl.enable(gl.BLEND);
  gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);

  let count = 0;
  let curTex = null;
  let cw = 390, ch = 844;

  function flush() {
    if (!count) return;
    gl.bindTexture(gl.TEXTURE_2D, curTex);
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, vbufData.subarray(0, count * 40));
    gl.drawElements(gl.TRIANGLES, count * 6, gl.UNSIGNED_SHORT, 0);
    count = 0;
  }

  function pushQuad(tex, x0, y0, x1, y1, x2, y2, x3, y3, u0, v0, u1, v1, col, alpha, add, flash) {
    if (tex !== curTex || count >= MAX_QUADS) {
      flush();
      curTex = tex;
    }
    const r = ((col >> 16) & 255) / 255;
    const g = ((col >> 8) & 255) / 255;
    const b = (col & 255) / 255;
    let o = count * 40;
    const writeV = (px, py, u, v) => {
      vbufData[o] = px;
      vbufData[o + 1] = py;
      vbufData[o + 2] = u;
      vbufData[o + 3] = v;
      vbufData[o + 4] = r;
      vbufData[o + 5] = g;
      vbufData[o + 6] = b;
      vbufData[o + 7] = alpha;
      vbufData[o + 8] = add ? 1 : 0;
      vbufData[o + 9] = flash || 0;
      o += 10;
    };
    writeV(x0, y0, u0, v0);
    writeV(x1, y1, u1, v0);
    writeV(x2, y2, u1, v1);
    writeV(x3, y3, u0, v1);
    count++;
  }

  let viewCX = 0, viewCY = 0, viewSc = 1;

  return {
    kind: "webgl",
    resize(w, h, dpr) {
      cw = w;
      ch = h;
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
      canvas.style.width = w + "px";
      canvas.style.height = h + "px";
      gl.viewport(0, 0, canvas.width, canvas.height);
    },
    begin() {
      count = 0;
      curTex = atlasTex;
      gl.clearColor(0.043, 0.075, 0.082, 1);
      gl.clear(gl.COLOR_BUFFER_BIT);
    },
    world(cx, cy, sc) {
      flush();
      viewCX = cx;
      viewCY = cy;
      viewSc = sc;
      const sx = (2 * sc) / cw;
      const sy = (-2 * sc) / ch;
      gl.uniform4f(uView, sx, sy, -cx * sx, -cy * sy);
    },
    screen() {
      flush();
      gl.uniform4f(uView, 2 / cw, -2 / ch, -1, 1);
    },
    tile(idx, ox, oy, tileSz, alpha, add, col = 0xffffff) {
      const hw = cw / 2 / viewSc + 80;
      const hh = ch / 2 / viewSc + 80;
      const x0 = viewCX - hw;
      const x1 = viewCX + hw;
      const y0 = viewCY - hh;
      const y1 = viewCY + hh;
      const u0 = (x0 + ox) / tileSz;
      const u1 = (x1 + ox) / tileSz;
      const v0 = (y0 + oy) / tileSz;
      const v1 = (y1 + oy) / tileSz;
      pushQuad(tileTexs[idx], x0, y0, x1, y0, x1, y1, x0, y1, u0, v0, u1, v1, col, alpha, add, 0);
    },
    spr(id, x, y, sx, sy, rot, alpha, col, add, flash) {
      if (alpha <= 0.004) return;
      const f = frames[id];
      if (!f) return;
      const hw = f.lw * 0.5 * sx;
      const hh = f.lh * 0.5 * sy;
      if (rot) {
        const c = Math.cos(rot);
        const s = Math.sin(rot);
        const wx = hw * c, wy = hw * s;
        const hx = -hh * s, hy = hh * c;
        pushQuad(
          atlasTex,
          x - wx - hx, y - wy - hy,
          x + wx - hx, y + wy - hy,
          x + wx + hx, y + wy + hy,
          x - wx + hx, y - wy + hy,
          f.u0, f.v0, f.u1, f.v1, col, alpha, add, flash,
        );
      } else {
        pushQuad(
          atlasTex,
          x - hw, y - hh,
          x + hw, y - hh,
          x + hw, y + hh,
          x - hw, y + hh,
          f.u0, f.v0, f.u1, f.v1, col, alpha, add, flash,
        );
      }
    },
    wave(id, x, y, sx, sy, rot, alpha, col, add, flash, phase, amp) {
      const f = frames[id];
      if (!f || alpha <= 0.004) return;
      const SLICES = 5;
      const hw = f.lw * 0.5 * sx;
      const hh = f.lh * 0.5 * sy;
      const c = Math.cos(rot);
      const s = Math.sin(rot);
      for (let i = 0; i < SLICES; i++) {
        const t0 = i / SLICES;
        const t1 = (i + 1) / SLICES;
        const lx0 = -hw + (t0 * 2 * hw);
        const lx1 = -hw + (t1 * 2 * hw);
        const w0 = Math.sin(phase + t0 * 4.2) * amp * (1 - t0 * 0.7);
        const w1 = Math.sin(phase + t1 * 4.2) * amp * (1 - t1 * 0.7);
        const u0 = f.u0 + (f.u1 - f.u0) * t0;
        const u1 = f.u0 + (f.u1 - f.u0) * t1;
        const p0x = x + lx0 * c - (-hh + w0) * s;
        const p0y = y + lx0 * s + (-hh + w0) * c;
        const p1x = x + lx1 * c - (-hh + w1) * s;
        const p1y = y + lx1 * s + (-hh + w1) * c;
        const p2x = x + lx1 * c - (hh + w1) * s;
        const p2y = y + lx1 * s + (hh + w1) * c;
        const p3x = x + lx0 * c - (hh + w0) * s;
        const p3y = y + lx0 * s + (hh + w0) * c;
        pushQuad(atlasTex, p0x, p0y, p1x, p1y, p2x, p2y, p3x, p3y, u0, f.v0, u1, f.v1, col, alpha, add, flash);
      }
    },
    end() {
      flush();
    },
  };
}

function createCanvas2DRenderer(canvas, atlas, tiles) {
  const ctx = canvas.getContext("2d");
  const frames = atlas.frames;
  const img = atlas.canvas;
  let cw = 390, ch = 844, dpr = 1, viewCX = 0, viewCY = 0, viewSc = 1;
  const patterns = tiles.map((t) => ctx.createPattern(t, "repeat"));

  return {
    kind: "2d",
    resize(w, h, r) {
      cw = w;
      ch = h;
      dpr = r;
      canvas.width = Math.round(w * r);
      canvas.height = Math.round(h * r);
      canvas.style.width = w + "px";
      canvas.style.height = h + "px";
    },
    begin() {
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.fillStyle = "#0b1315";
      ctx.fillRect(0, 0, cw, ch);
    },
    world(cx, cy, sc) {
      viewCX = cx;
      viewCY = cy;
      viewSc = sc;
      ctx.setTransform(dpr * sc, 0, 0, dpr * sc, dpr * (cw / 2 - cx * sc), dpr * (ch / 2 - cy * sc));
    },
    screen() {
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    },
    tile(idx, ox, oy, tileSz, alpha, add) {
      const pat = patterns[idx];
      if (!pat) return;
      ctx.save();
      ctx.globalAlpha = alpha;
      ctx.globalCompositeOperation = add ? "lighter" : "source-over";
      const hw = cw / 2 / viewSc + 60;
      const hh = ch / 2 / viewSc + 60;
      const s = tileSz / tiles[idx].width;
      ctx.translate(-ox, -oy);
      ctx.scale(s, s);
      ctx.fillStyle = pat;
      ctx.fillRect((viewCX - hw + ox) / s, (viewCY - hh + oy) / s, (hw * 2) / s, (hh * 2) / s);
      ctx.restore();
    },
    spr(id, x, y, sx, sy, rot, alpha, _col, add) {
      if (alpha <= 0.01) return;
      const f = frames[id];
      if (!f) return;
      ctx.save();
      ctx.globalAlpha = alpha;
      if (add) ctx.globalCompositeOperation = "lighter";
      ctx.translate(x, y);
      if (rot) ctx.rotate(rot);
      ctx.scale(sx, sy);
      ctx.drawImage(img, f.x, f.y, f.w, f.h, -f.lw / 2, -f.lh / 2, f.lw, f.lh);
      ctx.restore();
    },
    wave(id, x, y, sx, sy, rot, alpha, col, add, flash) {
      this.spr(id, x, y, sx, sy, rot, alpha, col, add, flash);
    },
    end() {},
  };
}
