// HONE Adventure V2 — Cleanroom AI Data Center Sprite Atlas, Hardware Projectiles, Icons & Dual Renderer

import { SHEET_FRAMES } from "./atlas-coords.js?v=3.0.0";

const TAU = Math.PI * 2;

export function rgbaHex(hex, a) {
  const n = parseInt(hex.slice(1), 16);
  return `rgba(${n >> 16},${(n >> 8) & 255},${n & 255},${a})`;
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

// Fallback procedural hero/enemy shapes (overridden when sprites.webp loads)
const HERO_PAL = {
  hone: ["#2563eb", "#1d4ed8", "#0f172a"],
  gold: ["#f59e0b", "#d97706", "#0f172a"],
  rubin: ["#059669", "#047857", "#0f172a"],
  photon: ["#0891b2", "#0e7490", "#0f172a"],
  quant: ["#4f46e5", "#4338ca", "#0f172a"],
};

Object.keys(HERO_PAL).forEach((id) => {
  const [c1, c2, dark] = HERO_PAL[id];
  const draw = (ctx) => {
    ctx.fillStyle = linGrad(ctx, 0, -20, 0, 20, [0, c1, 1, c2]);
    ctx.strokeStyle = dark;
    ctx.lineWidth = 2.4;
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
  packet: [34, "#dc2626"],
  jitter: [38, "#d97706"],
  thermal: [50, "#ea580c"],
  crosstalk: [42, "#7c3aed"],
  surge: [44, "#2563eb"],
  leak: [44, "#059669"],
  short: [42, "#e11d48"],
  halluc: [42, "#6d28d9"],
  frag: [26, "#0d9488"],
  boss1: [128, "#1d4ed8"],
  boss2: [136, "#0891b2"],
  boss3: [132, "#b45309"],
  boss4: [142, "#047857"],
  boss5: [144, "#6d28d9"],
};

Object.entries(ENEMY_PAL).forEach(([id, [sz, col]]) => {
  defSpr("e_" + id, sz, sz, (ctx) => {
    const r = sz * 0.32;
    ctx.fillStyle = col;
    ctx.strokeStyle = "#0f172a";
    ctx.lineWidth = 2.4;
    poly(ctx, 0, 0, id.startsWith("boss") ? 8 : 6, r, 0.2);
    ctx.fill();
    ctx.stroke();
  });
});

// ============================================================================
// V2 High-Contrast Hardware Projectiles & Cleanroom Zones (Designed for Light Floor!)
// ============================================================================

// 1. HBM Data Bus Packet: Rectangular framed bus packet [||===▶]
defSpr("p_hbm", 44, 18, (ctx) => {
  roundRectPath(ctx, -18, -5.5, 30, 11, 2.5);
  ctx.fillStyle = linGrad(ctx, -18, 0, 14, 0, [0, "#1e3a8a", 0.5, "#2563eb", 1, "#38bdf8"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.8;
  ctx.stroke();
  // Leading arrowhead
  ctx.beginPath();
  ctx.moveTo(11, -6.5);
  ctx.lineTo(20, 0);
  ctx.lineTo(11, 6.5);
  ctx.closePath();
  ctx.fillStyle = "#f59e0b";
  ctx.fill();
  ctx.stroke();
  // Gold HBM clock pins inside packet
  ctx.fillStyle = "#fde68a";
  ctx.fillRect(-14, -3, 3, 6);
  ctx.fillRect(-8, -3, 3, 6);
  ctx.fillStyle = "#e0f2fe";
  ctx.fillRect(-2, -2, 10, 4);
});

// 2. Silicon Photonics Coherent Lance + 3 WDM Wavelength Needles
defSpr("p_prism", 46, 22, (ctx) => {
  ctx.beginPath();
  ctx.moveTo(21, 0);
  ctx.lineTo(2, -7.5);
  ctx.lineTo(-18, -3.5);
  ctx.lineTo(-14, 0);
  ctx.lineTo(-18, 3.5);
  ctx.lineTo(2, 7.5);
  ctx.closePath();
  ctx.fillStyle = linGrad(ctx, -18, 0, 21, 0, [0, "#0e7490", 0.5, "#06b6d4", 1, "#ecfeff"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.8;
  ctx.stroke();
  // Internal prism facet
  ctx.beginPath();
  ctx.moveTo(14, 0);
  ctx.lineTo(0, -4);
  ctx.lineTo(-8, 0);
  ctx.lineTo(0, 4);
  ctx.closePath();
  ctx.fillStyle = "#f472b6";
  ctx.fill();
});

const makeWdmNeedle = (c1, c2) => (ctx) => {
  ctx.beginPath();
  ctx.moveTo(17, 0);
  ctx.lineTo(-3, -4.5);
  ctx.lineTo(-16, 0);
  ctx.lineTo(-3, 4.5);
  ctx.closePath();
  ctx.fillStyle = linGrad(ctx, -16, 0, 17, 0, [0, c2, 0.65, c1, 1, "#ffffff"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.5;
  ctx.stroke();
};
defSpr("p_w0", 38, 14, makeWdmNeedle("#0284c7", "#1e3a8a")); // 850nm Cobalt
defSpr("p_w1", 38, 14, makeWdmNeedle("#059669", "#064e3b")); // 1310nm Emerald
defSpr("p_w2", 38, 14, makeWdmNeedle("#c026d3", "#581c87")); // 1550nm Magenta

// 3. Systolic 3x3 Tensor Matrix Stamp (Ultra-Sheer Cleanroom Wireframe Reticle — Zero Occlusion!)
defSpr("p_tensor", 128, 128, (ctx) => {
  // Crisp outer square frame with ultra-sheer 4% tint so monsters stay 100% legible
  roundRectPath(ctx, -54, -54, 108, 108, 8);
  ctx.fillStyle = "rgba(16, 185, 129, 0.045)";
  ctx.fill();
  ctx.strokeStyle = "rgba(5, 150, 105, 0.72)";
  ctx.lineWidth = 2.2;
  ctx.stroke();

  // 3x3 internal MAC compute wireframe cells
  const step = 31;
  for (let gy = -1; gy <= 1; gy++) {
    for (let gx = -1; gx <= 1; gx++) {
      const cx = gx * step;
      const cy = gy * step;
      roundRectPath(ctx, cx - 11.5, cy - 11.5, 23, 23, 3);
      if (gx === 0 && gy === 0) {
        ctx.fillStyle = "rgba(245, 158, 11, 0.14)";
        ctx.fill();
      }
      ctx.strokeStyle = gx === 0 && gy === 0 ? "rgba(217, 119, 6, 0.8)" : "rgba(16, 185, 129, 0.42)";
      ctx.lineWidth = 1.3;
      ctx.stroke();
    }
  }

  // Crisp dark-slate corner alignment brackets
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2.8;
  const B = 58, L = 14;
  for (const [sx, sy] of [[-1, -1], [1, -1], [1, 1], [-1, 1]]) {
    ctx.beginPath();
    ctx.moveTo(sx * B, sy * (B - L));
    ctx.lineTo(sx * B, sy * B);
    ctx.lineTo(sx * (B - L), sy * B);
    ctx.stroke();
  }
});

// 4. L3 Cache Silicon Wafer Die
defSpr("p_die", 36, 36, (ctx) => {
  poly(ctx, 0, 0, 8, 13.5, Math.PI / 8);
  ctx.fillStyle = linGrad(ctx, -12, -12, 12, 12, [0, "#6366f1", 0.5, "#4338ca", 1, "#1e1b4b"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
  // Gold silicon die core
  roundRectPath(ctx, -6, -6, 12, 12, 2);
  ctx.fillStyle = linGrad(ctx, -6, -6, 6, 6, [0, "#fde68a", 1, "#d97706"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.2;
  ctx.stroke();
});

// 5. PCB Bus Trace Segment & Solder Via Node
defSpr("p_bus", 64, 12, (ctx) => {
  ctx.fillStyle = "#0f172a";
  ctx.fillRect(-32, -4.2, 64, 8.4);
  ctx.fillStyle = linGrad(ctx, 0, -2.6, 0, 2.6, [0, "#fbbf24", 0.5, "#fef08a", 1, "#d97706"]);
  ctx.fillRect(-32, -2.6, 64, 5.2);
});

defSpr("p_bus_c", 64, 12, (ctx) => {
  ctx.fillStyle = "#0f172a";
  ctx.fillRect(-32, -4.2, 64, 8.4);
  ctx.fillStyle = linGrad(ctx, 0, -2.6, 0, 2.6, [0, "#38bdf8", 0.5, "#e0f2fe", 1, "#2563eb"]);
  ctx.fillRect(-32, -2.6, 64, 5.2);
});

defSpr("via", 20, 20, (ctx) => {
  circle(ctx, 0, 0, 6.5);
  ctx.fillStyle = "#f59e0b";
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
  circle(ctx, 0, 0, 2.6);
  ctx.fillStyle = "#ffffff";
  ctx.fill();
});

// 6. EUV Lithography Cleanroom Scanner Beam
defSpr("p_litho", 64, 30, (ctx) => {
  ctx.fillStyle = linGrad(ctx, 0, -13, 0, 13, [
    0, "rgba(124, 58, 237, 0)",
    0.15, "rgba(109, 40, 217, 0.85)",
    0.5, "rgba(192, 38, 211, 0.95)",
    0.85, "rgba(109, 40, 217, 0.85)",
    1, "rgba(124, 58, 237, 0)",
  ]);
  ctx.fillRect(-32, -13, 64, 26);
  // Crisp dark-violet outer rail lines + white laser core
  ctx.fillStyle = "#2e1065";
  ctx.fillRect(-32, -11, 64, 2);
  ctx.fillRect(-32, 9, 64, 2);
  ctx.fillStyle = "#fdf4ff";
  ctx.fillRect(-32, -2.2, 64, 4.4);
});

// 7. Autonomous ASIC Delta-Wing Drone
defSpr("p_drone", 42, 36, (ctx) => {
  ctx.beginPath();
  ctx.moveTo(17, 0);
  ctx.lineTo(-13, -13);
  ctx.lineTo(-7, 0);
  ctx.lineTo(-13, 13);
  ctx.closePath();
  ctx.fillStyle = linGrad(ctx, -13, -10, 17, 10, [0, "#1e3a8a", 0.55, "#0284c7", 1, "#38bdf8"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
  // Thruster & NPU core
  ctx.fillStyle = "#f59e0b";
  circle(ctx, -8, 0, 3.4);
  ctx.fill();
  ctx.fillStyle = "#e0f2fe";
  ctx.beginPath();
  ctx.moveTo(9, 0);
  ctx.lineTo(-2, -4);
  ctx.lineTo(-2, 4);
  ctx.closePath();
  ctx.fill();
});

// 8. Immersion Cryo-Coolant Hexagonal Plate
defSpr("p_cryo", 128, 128, (ctx) => {
  poly(ctx, 0, 0, 6, 56, 0);
  ctx.fillStyle = "rgba(13, 148, 136, 0.16)";
  ctx.fill();
  ctx.strokeStyle = "#0f766e";
  ctx.lineWidth = 3.2;
  ctx.stroke();

  poly(ctx, 0, 0, 6, 36, Math.PI / 6);
  ctx.fillStyle = "rgba(6, 182, 212, 0.16)";
  ctx.fill();
  ctx.strokeStyle = "#0891b2";
  ctx.lineWidth = 2;
  ctx.stroke();

  // Frost crystal spokes
  ctx.strokeStyle = "#134e4a";
  ctx.lineWidth = 2;
  for (let i = 0; i < 6; i++) {
    const a = (i / 6) * TAU;
    ctx.beginPath();
    ctx.moveTo(Math.cos(a) * 10, Math.sin(a) * 10);
    ctx.lineTo(Math.cos(a) * 50, Math.sin(a) * 50);
    ctx.stroke();
  }
  circle(ctx, 0, 0, 8);
  ctx.fillStyle = "#06b6d4";
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.stroke();
});

// 9. NVLink Laser Mesh Beam & Optical Node
defSpr("p_nvlink", 64, 12, (ctx) => {
  ctx.fillStyle = "#4c0519";
  ctx.fillRect(-32, -4.2, 64, 8.4);
  ctx.fillStyle = linGrad(ctx, 0, -2.8, 0, 2.8, [0, "#f472b6", 0.5, "#fff1f2", 1, "#db2777"]);
  ctx.fillRect(-32, -2.6, 64, 5.2);
});

defSpr("p_nvnode", 22, 22, (ctx) => {
  poly(ctx, 0, 0, 6, 8, 0);
  ctx.fillStyle = "#ec4899";
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
  circle(ctx, 0, 0, 3);
  ctx.fillStyle = "#ffffff";
  ctx.fill();
});

// 10. Liquid Nitrogen Cryo-Mortar Canister & Frost Cross Rail
defSpr("p_n2", 34, 24, (ctx) => {
  roundRectPath(ctx, -12, -8, 24, 16, 5);
  ctx.fillStyle = linGrad(ctx, -12, -8, 12, 8, [0, "#e0f2fe", 0.5, "#0284c7", 1, "#0c4a6e"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2.2;
  ctx.stroke();
  ctx.fillStyle = "#38bdf8";
  ctx.fillRect(-5, -4, 10, 8);
});

defSpr("p_frost_rail", 64, 16, (ctx) => {
  ctx.fillStyle = "#082f49";
  ctx.fillRect(-32, -5.2, 64, 10.4);
  ctx.fillStyle = linGrad(ctx, 0, -3.5, 0, 3.5, [0, "#7dd3fc", 0.5, "#ffffff", 1, "#0284c7"]);
  ctx.fillRect(-32, -3.5, 64, 7);
});

// Cleanroom Interactive Sector Node Ring & Energy Forcefield Shield
defSpr("p_node_ring", 128, 128, (ctx) => {
  circle(ctx, 0, 0, 52);
  ctx.fillStyle = "rgba(37, 99, 235, 0.06)";
  ctx.fill();
  ctx.strokeStyle = "#1d4ed8";
  ctx.lineWidth = 3;
  ctx.setLineDash([12, 6]);
  ctx.stroke();
  ctx.setLineDash([]);
  poly(ctx, 0, 0, 6, 38, 0);
  ctx.strokeStyle = "rgba(15, 23, 42, 0.45)";
  ctx.lineWidth = 1.6;
  ctx.stroke();
});

defSpr("p_shield", 84, 84, (ctx) => {
  poly(ctx, 0, 0, 6, 36, Math.PI / 6);
  ctx.fillStyle = "rgba(56, 189, 248, 0.11)";
  ctx.fill();
  ctx.strokeStyle = "#0284c7";
  ctx.lineWidth = 2.8;
  ctx.stroke();
  poly(ctx, 0, 0, 6, 32, Math.PI / 6);
  ctx.strokeStyle = "rgba(255, 255, 255, 0.8)";
  ctx.lineWidth = 1.4;
  ctx.stroke();
});

defSpr("p_skybeam", 44, 128, (ctx) => {
  ctx.fillStyle = linGrad(ctx, -18, 0, 18, 0, [
    0, "rgba(245, 158, 11, 0)",
    0.2, "rgba(217, 119, 6, 0.72)",
    0.5, "rgba(254, 243, 199, 0.96)",
    0.8, "rgba(217, 119, 6, 0.72)",
    1, "rgba(245, 158, 11, 0)",
  ]);
  ctx.fillRect(-18, -64, 36, 128);
});

// Enemy Hazard Projectiles & Cleanroom Warning Reticles
defSpr("ebullet", 24, 24, (ctx) => {
  circle(ctx, 0, 0, 7);
  ctx.fillStyle = radGrad(ctx, -1, -1, 0.5, 0, 0, 7, [0, "#fff1f2", 0.45, "#f43f5e", 1, "#9f1239"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
});

defSpr("eb_sq", 24, 24, (ctx) => {
  roundRectPath(ctx, -7, -7, 14, 14, 2.5);
  ctx.fillStyle = linGrad(ctx, -7, -7, 7, 7, [0, "#fb7185", 0.5, "#e11d48", 1, "#881337"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
  ctx.fillStyle = "#fef2f2";
  ctx.fillRect(-2.5, -2.5, 5, 5);
});

defSpr("p_hazard", 128, 128, (ctx) => {
  roundRectPath(ctx, -52, -52, 104, 104, 10);
  ctx.fillStyle = "rgba(225, 29, 72, 0.13)";
  ctx.fill();
  ctx.strokeStyle = "#be123c";
  ctx.lineWidth = 3.2;
  ctx.setLineDash([10, 6]);
  ctx.stroke();
  ctx.setLineDash([]);
  // Crosshair center
  ctx.strokeStyle = "#9f1239";
  ctx.lineWidth = 2.2;
  ctx.beginPath();
  ctx.moveTo(-16, 0);
  ctx.lineTo(16, 0);
  ctx.moveTo(0, -16);
  ctx.lineTo(0, 16);
  ctx.stroke();
});

defSpr("ebeam", 64, 26, (ctx) => {
  ctx.fillStyle = "rgba(225, 29, 72, 0.88)";
  ctx.fillRect(-32, -8, 64, 16);
  ctx.fillStyle = "#881337";
  ctx.fillRect(-32, -9.5, 64, 2);
  ctx.fillRect(-32, 7.5, 64, 2);
  ctx.fillStyle = "#ffe4e6";
  ctx.fillRect(-32, -2.5, 64, 5);
});

defSpr("crown", 24, 18, (ctx) => {
  ctx.fillStyle = linGrad(ctx, 0, -6, 0, 5, [0, "#fde68a", 1, "#d97706"]);
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.8;
  ctx.beginPath();
  ctx.moveTo(-8, 5);
  ctx.lineTo(-9.5, -4);
  ctx.lineTo(-4, 0);
  ctx.lineTo(0, -6.5);
  ctx.lineTo(4, 0);
  ctx.lineTo(9.5, -4);
  ctx.lineTo(8, 5);
  ctx.closePath();
  ctx.fill();
  ctx.stroke();
});

// Pre-tinted Crisp Hexagonal / Circular Shockwave Rings (Work identically in WebGL & 2D on Light Floor!)
const makeCleanRing = (strokeHex, innerHex, hexShape = false) => (ctx) => {
  ctx.strokeStyle = strokeHex;
  ctx.lineWidth = 4.2;
  if (hexShape) poly(ctx, 0, 0, 6, 54, 0);
  else circle(ctx, 0, 0, 54);
  ctx.stroke();
  ctx.strokeStyle = innerHex;
  ctx.lineWidth = 2;
  if (hexShape) poly(ctx, 0, 0, 6, 48, 0);
  else circle(ctx, 0, 0, 48);
  ctx.stroke();
};

defSpr("ring", 128, 128, makeCleanRing("#2563eb", "#0f172a", false));
defSpr("ring2", 128, 128, makeCleanRing("#059669", "#064e3b", true));
defSpr("ring_warn", 128, 128, makeCleanRing("#e11d48", "#881337", false));
defSpr("ring_gold", 128, 128, makeCleanRing("#d97706", "#78350f", true));

defSpr("disc", 64, 64, (ctx) => {
  circle(ctx, 0, 0, 30);
  ctx.fillStyle = "rgba(37, 99, 235, 0.16)";
  ctx.fill();
});

defSpr("glow", 64, 64, (ctx) => {
  ctx.fillStyle = radGrad(ctx, 0, 0, 0, 0, 0, 30, [
    0, "rgba(37, 99, 235, 0.22)",
    0.55, "rgba(37, 99, 235, 0.08)",
    1, "rgba(37, 99, 235, 0)",
  ]);
  ctx.fillRect(-32, -32, 64, 64);
});

defSpr("dotp", 16, 16, (ctx) => {
  roundRectPath(ctx, -4.5, -4.5, 9, 9, 2);
  ctx.fillStyle = "#2563eb";
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.2;
  ctx.stroke();
});

defSpr("shard", 18, 18, (ctx) => {
  ctx.beginPath();
  ctx.moveTo(0, -7);
  ctx.lineTo(6, 0);
  ctx.lineTo(0, 7);
  ctx.lineTo(-6, 0);
  ctx.closePath();
  ctx.fillStyle = "#0891b2";
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.4;
  ctx.stroke();
});

defSpr("spark", 26, 26, (ctx) => {
  starPoly(ctx, 0, 0, 4, 10.5, 3, 0);
  ctx.fillStyle = "#f59e0b";
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.4;
  ctx.stroke();
});

defSpr("px", 8, 8, (ctx) => {
  ctx.fillStyle = "#ffffff";
  ctx.fillRect(-3, -3, 6, 6);
});

// Subtle Cleanroom Edge Frame (NO dark deep-sea tunnel!)
defSpr("vig", 128, 128, (ctx) => {
  ctx.fillStyle = radGrad(ctx, 0, 0, 42, 0, 0, 92, [
    0, "rgba(15, 23, 42, 0)",
    0.75, "rgba(15, 23, 42, 0.03)",
    1, "rgba(15, 23, 42, 0.12)",
  ]);
  ctx.fillRect(-64, -64, 128, 128);
});

defSpr("vigr", 128, 128, (ctx) => {
  ctx.fillStyle = radGrad(ctx, 0, 0, 38, 0, 0, 90, [
    0, "rgba(225, 29, 72, 0)",
    0.65, "rgba(225, 29, 72, 0.16)",
    1, "rgba(225, 29, 72, 0.55)",
  ]);
  ctx.fillRect(-64, -64, 128, 128);
});

// High-Contrast Compute Data Tokens (XP Gems on Light Floor!)
const makeChipGem = (c1, c2, sz) => (ctx) => {
  ctx.beginPath();
  ctx.moveTo(0, -sz * 1.22);
  ctx.lineTo(sz * 0.98, 0);
  ctx.lineTo(0, sz * 1.22);
  ctx.lineTo(-sz * 0.98, 0);
  ctx.closePath();
  ctx.fillStyle = linGrad(ctx, -sz, -sz, sz, sz, [0, c1, 1, c2]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.7;
  ctx.stroke();
  ctx.fillStyle = "rgba(255,255,255,.85)";
  circle(ctx, -sz * 0.18, -sz * 0.25, sz * 0.22);
  ctx.fill();
};

defSpr("gem0", 22, 22, makeChipGem("#10b981", "#047857", 5.2));
defSpr("gem1", 26, 26, makeChipGem("#38bdf8", "#1d4ed8", 6.2));
defSpr("gem2", 32, 32, makeChipGem("#fbbf24", "#d97706", 7.6));

defSpr("heart", 34, 34, (ctx) => {
  roundRectPath(ctx, -9, -12, 18, 24, 5);
  ctx.fillStyle = linGrad(ctx, 0, -12, 0, 12, [0, "#34d399", 1, "#059669"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
});

defSpr("magnet", 36, 36, (ctx) => {
  ctx.lineWidth = 6;
  ctx.lineCap = "round";
  ctx.strokeStyle = "#2563eb";
  ctx.beginPath();
  ctx.arc(0, 1, 9, 0, Math.PI);
  ctx.moveTo(-9, 1);
  ctx.lineTo(-9, -8);
  ctx.moveTo(9, 1);
  ctx.lineTo(9, -8);
  ctx.stroke();
});

defSpr("orb", 40, 40, (ctx) => {
  circle(ctx, 0, 0, 12);
  ctx.fillStyle = radGrad(ctx, 0, 0, 1, 0, 0, 12, [0, "#ffffff", 0.5, "#a855f7", 1, "#4c1d95"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
});

defSpr("core", 30, 30, (ctx) => {
  roundRectPath(ctx, -8.5, -8.5, 17, 17, 3);
  ctx.fillStyle = linGrad(ctx, -8, -8, 8, 8, [0, "#fde68a", 0.5, "#f59e0b", 1, "#b45309"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
});

defSpr("chest", 48, 44, (ctx) => {
  roundRectPath(ctx, -14, -10, 28, 22, 4);
  ctx.fillStyle = linGrad(ctx, 0, -10, 0, 12, [0, "#3b82f6", 1, "#1d4ed8"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2.2;
  ctx.stroke();
});

defSpr("joy", 120, 120, (ctx) => {
  poly(ctx, 0, 0, 8, 50, Math.PI / 8);
  ctx.fillStyle = "rgba(15, 23, 42, 0.06)";
  ctx.fill();
  ctx.strokeStyle = "rgba(37, 99, 235, 0.45)";
  ctx.lineWidth = 2;
  ctx.stroke();
  circle(ctx, 0, 0, 30);
  ctx.strokeStyle = "rgba(15, 23, 42, 0.18)";
  ctx.lineWidth = 1.2;
  ctx.stroke();
});

defSpr("knob", 52, 52, (ctx) => {
  poly(ctx, 0, 0, 6, 16, -Math.PI / 2);
  ctx.fillStyle = linGrad(ctx, 0, -16, 0, 16, [0, "#3b82f6", 1, "#1d4ed8"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
});

// Cleanroom Floor Bay Pad & Props
defSpr("bay_pad", 160, 110, (ctx) => {
  roundRectPath(ctx, -72, -46, 144, 92, 8);
  ctx.fillStyle = "rgba(203, 213, 225, 0.34)";
  ctx.fill();
  ctx.strokeStyle = "rgba(100, 116, 139, 0.38)";
  ctx.lineWidth = 1.8;
  ctx.setLineDash([8, 5]);
  ctx.stroke();
  ctx.setLineDash([]);
});

defSpr("d_rack", 72, 76, (ctx) => {
  roundRectPath(ctx, -22, -30, 44, 60, 5);
  ctx.fillStyle = linGrad(ctx, 0, -30, 0, 30, [0, "#e2e8f0", 1, "#94a3b8"]);
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
});

defSpr("d_monolith", 68, 72, (ctx) => {
  poly(ctx, 0, 2, 6, 28, -Math.PI / 2);
  ctx.fillStyle = "#fde68a";
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
});

defSpr("d_crystal", 64, 64, (ctx) => {
  roundRectPath(ctx, -20, -14, 40, 28, 6);
  ctx.fillStyle = "#67e8f9";
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 2;
  ctx.stroke();
});

defSpr("streak", 64, 14, (ctx) => {
  ctx.fillStyle = linGrad(ctx, -30, 0, 30, 0, [
    0, "rgba(37, 99, 235, 0)",
    0.55, "rgba(37, 99, 235, 0.36)",
    1, "rgba(56, 189, 248, 0.72)",
  ]);
  roundRectPath(ctx, -30, -3.5, 60, 7, 3.5);
  ctx.fill();
});

defSpr("flare", 72, 72, (ctx) => {
  starPoly(ctx, 0, 0, 4, 24, 5, 0);
  ctx.fillStyle = "rgba(245, 158, 11, 0.85)";
  ctx.fill();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.4;
  ctx.stroke();
});

defSpr("rune", 128, 128, (ctx) => {
  ctx.strokeStyle = "#2563eb";
  ctx.lineWidth = 2.6;
  poly(ctx, 0, 0, 6, 54, 0);
  ctx.stroke();
  ctx.strokeStyle = "#0f172a";
  ctx.lineWidth = 1.4;
  circle(ctx, 0, 0, 42);
  ctx.stroke();
});

defSpr("shadow", 64, 32, (ctx) => {
  ctx.save();
  ctx.scale(1, 0.5);
  ctx.fillStyle = radGrad(ctx, 0, 0, 0, 0, 0, 28, [
    0, "rgba(15, 23, 42, 0.28)",
    0.65, "rgba(15, 23, 42, 0.12)",
    1, "rgba(15, 23, 42, 0)",
  ]);
  circle(ctx, 0, 0, 28);
  ctx.fill();
  ctx.restore();
});

// Crisp Dark-Slate & Crimson Outlined Damage Numerals for Light Cleanroom Floor
const NUM_FONT = '900 italic 20px "Avenir Next", "Arial Black", "Helvetica Neue", "PingFang SC", sans-serif';
for (let d = 0; d < 10; d++) {
  defSpr("n" + d, 18, 24, (ctx) => {
    ctx.font = NUM_FONT;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.lineJoin = "round";
    ctx.lineWidth = 4.2;
    ctx.strokeStyle = "#ffffff";
    ctx.strokeText(String(d), 0, 1);
    ctx.fillStyle = "#0f172a";
    ctx.fillText(String(d), 0, 1);
  });
  defSpr("c" + d, 20, 28, (ctx) => {
    ctx.font = NUM_FONT.replace("20px", "23px");
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.lineJoin = "round";
    ctx.lineWidth = 4.8;
    ctx.strokeStyle = "#0f172a";
    ctx.strokeText(String(d), 0, 1);
    ctx.fillStyle = linGrad(ctx, 0, -9, 0, 10, [0, "#fde047", 0.5, "#f59e0b", 1, "#dc2626"]);
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

// Light Cleanroom Raised Server Floor Fallback
export function makeFallbackFloor(size = 512) {
  const c = document.createElement("canvas");
  c.width = size;
  c.height = size;
  const ctx = c.getContext("2d");
  ctx.fillStyle = "#eef3f9";
  ctx.fillRect(0, 0, size, size);
  ctx.strokeStyle = "rgba(148, 163, 184, 0.32)";
  ctx.lineWidth = 1;
  for (let i = 0; i <= size; i += 64) {
    ctx.beginPath();
    ctx.moveTo(i, 0);
    ctx.lineTo(i, size);
    ctx.moveTo(0, i);
    ctx.lineTo(size, i);
    ctx.stroke();
  }
  ctx.fillStyle = "rgba(148, 163, 184, 0.35)";
  for (let y = 0; y < size; y += 64) {
    for (let x = 0; x < size; x += 64) {
      ctx.fillRect(x + 5, y + 5, 2, 2);
      ctx.fillRect(x + 57, y + 5, 2, 2);
      ctx.fillRect(x + 5, y + 57, 2, 2);
      ctx.fillRect(x + 57, y + 57, 2, 2);
    }
  }
  return c;
}

// Crisp Orthogonal Cleanroom Bus-Rail Overlay (Zero Underwater Caustics!)
export function makeQuantOverlay(size = 256) {
  const c = document.createElement("canvas");
  c.width = size;
  c.height = size;
  const ctx = c.getContext("2d");
  ctx.strokeStyle = "rgba(59, 130, 246, 0.11)";
  ctx.lineWidth = 1.5;
  for (let i = 0; i < 6; i++) {
    const y = 24 + i * 42;
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(96, y);
    ctx.lineTo(116, y + 20);
    ctx.lineTo(size, y + 20);
    ctx.stroke();
  }
  ctx.fillStyle = "rgba(37, 99, 235, 0.16)";
  for (let i = 0; i < 6; i++) {
    circle(ctx, 96, 24 + i * 42, 2.5);
    ctx.fill();
  }
  return c;
}

// ============================================================================
// Crisp Weapon, Passive & UI Icon Drawers (64x64 coordinate space, High Contrast)
// ============================================================================
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

const strokeStyle = (ctx, lw = 4.2, col = "#0f172a") => {
  ctx.strokeStyle = col;
  ctx.lineWidth = lw;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
};

export const ICON_DRAWERS = {
  hone_logo: (ctx) => {
    ctx.fillStyle = linGrad(ctx, -18, 8, 16, -22, [0, "#2563eb", 1, "#06b6d4"]);
    ctx.beginPath();
    ctx.moveTo(-16, 4);
    ctx.lineTo(4, -22);
    ctx.lineTo(18, -14);
    ctx.lineTo(-2, 12);
    ctx.closePath();
    ctx.fill();
    ctx.fillStyle = "#0f172a";
    ctx.beginPath();
    ctx.moveTo(-22, -6);
    ctx.lineTo(20, 12);
    ctx.lineTo(20, 21);
    ctx.lineTo(-22, 6);
    ctx.closePath();
    ctx.fill();
    ctx.fillStyle = "#f59e0b";
    ctx.beginPath();
    ctx.moveTo(-18, -10);
    ctx.lineTo(22, 8);
    ctx.lineTo(18, 11);
    ctx.lineTo(-22, -7);
    ctx.closePath();
    ctx.fill();
  },
  hbm: (ctx, sc) => {
    drawSprFit(ctx, sc, "p_hbm", 46, -2, -10, -0.35);
    drawSprFit(ctx, sc, "p_hbm", 46, 3, 2, -0.35);
    drawSprFit(ctx, sc, "p_hbm", 46, -2, 14, -0.35);
  },
  prism: (ctx, sc) => {
    drawSprFit(ctx, sc, "p_w0", 34, 8, -12, -0.55);
    drawSprFit(ctx, sc, "p_w1", 34, 11, 0, -0.15);
    drawSprFit(ctx, sc, "p_w2", 34, 8, 12, 0.25);
    drawSprFit(ctx, sc, "p_prism", 42, -8, 2, -0.15);
  },
  tensor: (ctx, sc) => drawSprFit(ctx, sc, "p_tensor", 54),
  cache: (ctx, sc) => {
    ctx.strokeStyle = "#4f46e5";
    ctx.lineWidth = 2.4;
    poly(ctx, 0, 0, 6, 19, 0);
    ctx.stroke();
    for (let i = 0; i < 3; i++) {
      const a = (i / 3) * TAU - 0.5;
      drawSprFit(ctx, sc, "p_die", 22, Math.cos(a) * 19, Math.sin(a) * 19);
    }
  },
  bus: (ctx) => {
    strokeStyle(ctx, 5.5, "#0f172a");
    ctx.beginPath();
    ctx.moveTo(-20, 16);
    ctx.lineTo(-4, 16);
    ctx.lineTo(8, -14);
    ctx.lineTo(22, -14);
    ctx.stroke();
    strokeStyle(ctx, 3, "#f59e0b");
    ctx.stroke();
    for (const [vx, vy] of [[-20, 16], [-4, 16], [8, -14], [22, -14]]) {
      ctx.fillStyle = "#fef08a";
      ctx.strokeStyle = "#0f172a";
      ctx.lineWidth = 2;
      circle(ctx, vx, vy, 4.5);
      ctx.fill();
      ctx.stroke();
    }
  },
  litho: (ctx) => {
    roundRectPath(ctx, -22, -18, 44, 36, 5);
    ctx.fillStyle = "rgba(124, 58, 237, 0.14)";
    ctx.fill();
    strokeStyle(ctx, 2.5, "#6d28d9");
    ctx.stroke();
    ctx.fillStyle = "#c026d3";
    ctx.fillRect(-24, -3, 48, 6);
    strokeStyle(ctx, 2, "#0f172a");
    ctx.strokeRect(-24, -3, 48, 6);
  },
  drone: (ctx, sc) => drawSprFit(ctx, sc, "p_drone", 54, 0, 0, -0.45),
  cryo: (ctx, sc) => drawSprFit(ctx, sc, "p_cryo", 56),
  nvlink: (ctx) => {
    const pts = [[0, -18], [18, 10], [-18, 10], [0, 3]];
    strokeStyle(ctx, 4.5, "#0f172a");
    ctx.beginPath();
    ctx.moveTo(0, -18); ctx.lineTo(18, 10); ctx.lineTo(-18, 10); ctx.closePath();
    ctx.moveTo(0, -18); ctx.lineTo(0, 3); ctx.lineTo(18, 10); ctx.moveTo(0, 3); ctx.lineTo(-18, 10);
    ctx.stroke();
    strokeStyle(ctx, 2.5, "#ec4899");
    ctx.stroke();
    for (const [nx, ny] of pts) {
      ctx.fillStyle = "#fdf2f8";
      ctx.strokeStyle = "#0f172a";
      ctx.lineWidth = 2;
      circle(ctx, nx, ny, 4.5);
      ctx.fill();
      ctx.stroke();
    }
  },
  liquid_n2: (ctx) => {
    strokeStyle(ctx, 4.5, "#0f172a");
    ctx.beginPath();
    ctx.moveTo(-22, 0); ctx.lineTo(22, 0); ctx.moveTo(0, -22); ctx.lineTo(0, 22);
    ctx.stroke();
    strokeStyle(ctx, 2.5, "#38bdf8");
    ctx.stroke();
    roundRectPath(ctx, -10, -12, 20, 24, 5);
    ctx.fillStyle = "#0284c7";
    ctx.fill();
    strokeStyle(ctx, 2.2, "#0f172a");
    ctx.stroke();
    ctx.fillStyle = "#e0f2fe";
    ctx.fillRect(-5, -5, 10, 10);
  },
  power_rail: (ctx) => {
    roundRectPath(ctx, -20, -13, 40, 9, 3);
    ctx.fillStyle = "#0284c7";
    ctx.fill();
    strokeStyle(ctx, 2.2, "#0f172a");
    ctx.stroke();
    roundRectPath(ctx, -20, 4, 40, 9, 3);
    ctx.fillStyle = "#f59e0b";
    ctx.fill();
    ctx.stroke();
  },
  ui_skill: (ctx) => {
    ctx.fillStyle = "#f59e0b";
    ctx.strokeStyle = "#0f172a";
    ctx.lineWidth = 2.4;
    ctx.beginPath();
    ctx.moveTo(4, -22);
    ctx.lineTo(-14, 2);
    ctx.lineTo(-1, 2);
    ctx.lineTo(-6, 22);
    ctx.lineTo(14, -3);
    ctx.lineTo(1, -3);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  },
  ui_jump: (ctx) => {
    strokeStyle(ctx, 3.8, "#2563eb");
    circle(ctx, 0, 0, 18);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(-7, 6);
    ctx.lineTo(6, -7);
    ctx.moveTo(-3, -7);
    ctx.lineTo(6, -7);
    ctx.lineTo(6, 2);
    ctx.stroke();
  },

  clock: (ctx) => {
    strokeStyle(ctx, 3.6, "#2563eb");
    circle(ctx, 0, 0, 19);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(0, -11);
    ctx.lineTo(0, 0);
    ctx.lineTo(9, 6);
    ctx.stroke();
  },
  waveguide: (ctx) => {
    strokeStyle(ctx, 3.2, "#0891b2");
    poly(ctx, 0, 0, 6, 21, 0);
    ctx.stroke();
    poly(ctx, 0, 0, 6, 11, 0);
    ctx.stroke();
  },
  ecc: (ctx) => {
    ctx.fillStyle = linGrad(ctx, 0, -22, 0, 22, [0, "#818cf8", 1, "#4338ca"]);
    ctx.strokeStyle = "#0f172a";
    ctx.lineWidth = 2.4;
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
  fp8: (ctx) => {
    strokeStyle(ctx, 3.2, "#059669");
    circle(ctx, 0, 0, 16);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(-22, 0);
    ctx.lineTo(-10, 0);
    ctx.moveTo(10, 0);
    ctx.lineTo(22, 0);
    ctx.moveTo(0, -22);
    ctx.lineTo(0, -10);
    ctx.moveTo(0, 10);
    ctx.lineTo(0, 22);
    ctx.stroke();
    ctx.fillStyle = "#f59e0b";
    circle(ctx, 0, 0, 4.5);
    ctx.fill();
  },
  reticle: (ctx, sc) => drawSprFit(ctx, sc, "magnet", 52),
  overvolt: (ctx) => {
    strokeStyle(ctx, 4.5, "#d97706");
    ctx.beginPath();
    ctx.moveTo(-20, 14);
    ctx.lineTo(-6, 2);
    ctx.lineTo(3, 9);
    ctx.lineTo(19, -14);
    ctx.moveTo(9, -14);
    ctx.lineTo(19, -14);
    ctx.lineTo(19, -4);
    ctx.stroke();
  },
  manifold: (ctx) => {
    strokeStyle(ctx, 3.6, "#0d9488");
    roundRectPath(ctx, -16, -16, 32, 32, 6);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(-8, -16);
    ctx.lineTo(-8, 16);
    ctx.moveTo(8, -16);
    ctx.lineTo(8, 16);
    ctx.stroke();
  },
  fabric: (ctx) => {
    ctx.fillStyle = linGrad(ctx, -18, 16, 18, -18, [0, "#0284c7", 1, "#38bdf8"]);
    ctx.strokeStyle = "#0f172a";
    ctx.lineWidth = 2.2;
    ctx.beginPath();
    ctx.moveTo(18, -18);
    ctx.lineTo(-18, -2);
    ctx.lineTo(-4, 4);
    ctx.lineTo(2, 18);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  },
  ups: (ctx) => {
    roundRectPath(ctx, -16, -12, 30, 24, 4);
    ctx.fillStyle = "#10b981";
    ctx.fill();
    strokeStyle(ctx, 2.4, "#0f172a");
    ctx.stroke();
    ctx.fillStyle = "#0f172a";
    ctx.fillRect(14, -5, 4, 10);
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(-3, -7, 4, 14);
    ctx.fillRect(-8, -2, 14, 4);
  },
  b_might: (ctx) => ICON_DRAWERS.overvolt(ctx),
  b_heal: (ctx) => ICON_DRAWERS.ups(ctx),
  b_core: (ctx, sc) => drawSprFit(ctx, sc, "core", 52),
  core: (ctx, sc) => drawSprFit(ctx, sc, "core", 54),
  chest: (ctx, sc) => drawSprFit(ctx, sc, "chest", 58),
  nuke: (ctx, sc) => drawSprFit(ctx, sc, "orb", 54),

  ui_up: (ctx) => {
    strokeStyle(ctx, 4.2, "#1e293b");
    ctx.beginPath();
    ctx.moveTo(0, 20);
    ctx.lineTo(0, -16);
    ctx.moveTo(-14, -3);
    ctx.lineTo(0, -18);
    ctx.lineTo(14, -3);
    ctx.moveTo(-18, 22);
    ctx.lineTo(18, 22);
    ctx.stroke();
  },
  ui_agent: (ctx, sc) => drawSprFit(ctx, sc, "h_hone", 58),
  ui_book: (ctx) => {
    strokeStyle(ctx, 4, "#1e293b");
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
    strokeStyle(ctx, 4, "#1e293b");
    circle(ctx, 0, 6, 14);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(-9, -6);
    ctx.lineTo(-14, -22);
    ctx.moveTo(9, -6);
    ctx.lineTo(14, -22);
    ctx.stroke();
    ctx.fillStyle = "#f59e0b";
    starPoly(ctx, 0, 6, 5, 7.5, 3.2, -Math.PI / 2);
    ctx.fill();
  },
  ui_rank: (ctx) => {
    strokeStyle(ctx, 4, "#1e293b");
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
  ui_star: (ctx) => {
    ctx.fillStyle = "#f59e0b";
    ctx.strokeStyle = "#0f172a";
    ctx.lineWidth = 2.4;
    starPoly(ctx, 0, 0, 5, 22, 9.5, -Math.PI / 2);
    ctx.fill();
    ctx.stroke();
  },
  ui_gear: (ctx) => {
    strokeStyle(ctx, 4, "#1e293b");
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
    ctx.fillStyle = "#0f172a";
    roundRectPath(ctx, -14, -17, 10, 34, 3);
    ctx.fill();
    roundRectPath(ctx, 4, -17, 10, 34, 3);
    ctx.fill();
  },
  ui_cal: (ctx) => {
    strokeStyle(ctx, 4, "#2563eb");
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
    ctx.fillStyle = "#f59e0b";
    starPoly(ctx, 0, 9, 5, 7.5, 3.2, -Math.PI / 2);
    ctx.fill();
  },
  ui_home: (ctx) => {
    strokeStyle(ctx, 4.2, "#0f172a");
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
    strokeStyle(ctx, 4.4, "#0f172a");
    ctx.beginPath();
    ctx.moveTo(16, 0);
    ctx.lineTo(-14, 0);
    ctx.moveTo(-3, -12);
    ctx.lineTo(-15, 0);
    ctx.lineTo(-3, 12);
    ctx.stroke();
  },
  ui_retry: (ctx) => {
    strokeStyle(ctx, 4, "#2563eb");
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
    strokeStyle(ctx, 4, "#0f172a");
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
    strokeStyle(ctx, 4, "#0f172a");
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
    strokeStyle(ctx, 4, "#0f172a");
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
    strokeStyle(ctx, 4, "#d97706");
    roundRectPath(ctx, -13, -3, 26, 20, 4);
    ctx.stroke();
    ctx.beginPath();
    ctx.arc(0, -4, 8, Math.PI, 0);
    ctx.stroke();
  },
  ui_x: (ctx) => {
    strokeStyle(ctx, 5, "#0f172a");
    ctx.beginPath();
    ctx.moveTo(-14, -14);
    ctx.lineTo(14, 14);
    ctx.moveTo(14, -14);
    ctx.lineTo(-14, 14);
    ctx.stroke();
  },
  ui_check: (ctx) => {
    strokeStyle(ctx, 6, "#059669");
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

// WebGL 4096-Quad Sprite Batcher + Canvas 2D Fallback (Light Cleanroom Calibrated)
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
      gl.clearColor(0.941, 0.961, 0.980, 1);
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
    spr(id, x, y, sx, sy, rot, alpha, col = 0xffffff, add = 0, flash = 0) {
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
    end() {
      flush();
    },
  };
}

function createCanvas2DRenderer(canvas, atlas, tiles) {
  const ctx = canvas.getContext("2d");
  const frames = atlas.frames;
  const img = atlas.canvas;
  const pxId = SPR_IDX.px;
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
      ctx.fillStyle = "#eff4fa";
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
    tile(idx, ox, oy, tileSz, alpha) {
      const pat = patterns[idx];
      if (!pat) return;
      ctx.save();
      ctx.globalAlpha = alpha;
      const hw = cw / 2 / viewSc + 60;
      const hh = ch / 2 / viewSc + 60;
      const s = tileSz / tiles[idx].width;
      ctx.translate(-ox, -oy);
      ctx.scale(s, s);
      ctx.fillStyle = pat;
      ctx.fillRect((viewCX - hw + ox) / s, (viewCY - hh + oy) / s, (hw * 2) / s, (hh * 2) / s);
      ctx.restore();
    },
    spr(id, x, y, sx, sy, rot, alpha, col = 0xffffff) {
      if (alpha <= 0.01) return;
      const f = frames[id];
      if (!f) return;
      ctx.save();
      ctx.globalAlpha = alpha;
      ctx.translate(x, y);
      if (rot) ctx.rotate(rot);
      if (id === pxId) {
        ctx.fillStyle = "#" + (col & 0xffffff).toString(16).padStart(6, "0");
        ctx.fillRect(-3 * sx, -3 * sy, 6 * sx, 6 * sy);
      } else {
        ctx.scale(sx, sy);
        ctx.drawImage(img, f.x, f.y, f.w, f.h, -f.lw / 2, -f.lh / 2, f.lw, f.lh);
      }
      ctx.restore();
    },
    end() {},
  };
}
