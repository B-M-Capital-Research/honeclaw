// HONE Adventure V2 — 1080x1440 (3:4) Light Cleanroom AI Data Center Battle Report Poster

import { HEROES, HERO_IDX, HONECLAW_REPO_LABEL, RULES, STAGES, STAGE_IDX, WEAPONS, WEAPON_IDX, fmtTime, getTitle } from "./data.js?v=3.0.0";
import { ICON_DRAWERS, drawSprite2D, makeFallbackFloor, makeQuantOverlay, roundRectPath } from "./art.js?v=3.0.0";

const FONT = '"Avenir Next", -apple-system, "PingFang SC", "HarmonyOS Sans SC", "Microsoft YaHei", sans-serif';
let bgFloor = null;
let bgQuant = null;

function drawIcon(ctx, id, x, y, sz) {
  ctx.save();
  ctx.translate(x, y);
  ctx.scale(sz / 64, sz / 64);
  if (ICON_DRAWERS[id]) ICON_DRAWERS[id](ctx, sz / 64);
  ctx.restore();
}

export function renderBattlePoster(sum, save, floorImg) {
  const c = document.createElement("canvas");
  c.width = 1080;
  c.height = 1440;
  const ctx = c.getContext("2d");

  bgFloor = floorImg || bgFloor || makeFallbackFloor(512);
  bgQuant = bgQuant || makeQuantOverlay(256);

  // 1. Bright Cleanroom AI Data Center Floor Canvas
  ctx.fillStyle = "#eef4fb";
  ctx.fillRect(0, 0, 1080, 1440);
  ctx.save();
  ctx.scale(2, 2);
  ctx.fillStyle = ctx.createPattern(bgFloor, "repeat");
  ctx.fillRect(0, 0, 540, 720);
  ctx.restore();

  ctx.save();
  ctx.globalAlpha = 0.72;
  ctx.scale(3.6, 3.6);
  ctx.fillStyle = ctx.createPattern(bgQuant, "repeat");
  ctx.fillRect(0, 0, 1080 / 3.6, 1440 / 3.6);
  ctx.restore();

  // Crisp Architectural Frame Border
  roundRectPath(ctx, 28, 28, 1024, 1384, 24);
  ctx.strokeStyle = "rgba(37, 99, 235, 0.28)";
  ctx.lineWidth = 4;
  ctx.stroke();

  ctx.textAlign = "center";
  ctx.textBaseline = "middle";

  // 2. Surrounding Hardware Bottleneck Ring
  const mobs = ["e_packet", "e_jitter", "e_thermal", "e_crosstalk", "e_surge", "e_leak", "e_short", "e_halluc"];
  mobs.forEach((m, i) => {
    const a = (i / mobs.length) * Math.PI * 2 + 0.35;
    const rx = 365 + (i % 2) * 38;
    ctx.globalAlpha = 0.92;
    drawSprite2D(ctx, m, 540 + Math.cos(a) * rx, 515 + Math.sin(a) * rx * 0.54, 2.8 + (i % 3) * 0.25, 0, Math.cos(a) > 0);
  });
  ctx.globalAlpha = 1;

  // 3. Cleanroom Circuit Rings + Hero Spotlight
  for (let i = 0; i < 3; i++) {
    ctx.strokeStyle = i === 0 ? "rgba(37, 99, 235, 0.38)" : `rgba(15, 23, 42, ${0.14 - i * 0.03})`;
    ctx.lineWidth = 6 - i * 1.5;
    ctx.beginPath();
    ctx.ellipse(540, 525, 175 + i * 62, (175 + i * 62) * 0.56, 0, 0, Math.PI * 2);
    ctx.stroke();
  }

  const heroDef = HEROES[HERO_IDX[sum.char]] || HEROES[0];
  const heroKey = sum.char === "hone" && save.skin === "gold" && save.skins?.gold ? "gold" : heroDef.sprite || sum.char;
  ctx.imageSmoothingQuality = "high";
  drawSprite2D(ctx, "shadow", 540, 700, 8.0, 0, false);
  drawSprite2D(ctx, "P_" + heroKey, 540, 505, 8.5, 0, false);

  // 4. Header Title & Promotion Badge
  const stg = STAGES[STAGE_IDX[sum.endStage || sum.stage || "storage"]] || STAGES[0];
  ctx.font = "800 28px " + FONT;
  ctx.fillStyle = "#2563eb";
  ctx.fillText(`HONE AI DATA CENTER · ${stg.name}`, 540, 78);

  ctx.font = "900 66px " + FONT;
  ctx.fillStyle = "#0f172a";
  ctx.fillText("HONE 历险记", 540, 138);

  const rankName = getTitle(Math.max(save.best.time || 0, sum.time)).name;
  ctx.font = "800 31px " + FONT;
  const pillW = ctx.measureText(rankName).width + 64;
  roundRectPath(ctx, 540 - pillW / 2, 186, pillW, 52, 12);
  ctx.fillStyle = "#dbeafe";
  ctx.fill();
  ctx.strokeStyle = "#2563eb";
  ctx.lineWidth = 2.5;
  ctx.stroke();
  ctx.fillStyle = "#1e3a8a";
  ctx.fillText(rankName, 540, 212);

  // 5. Survival Time Hero Metric
  const ruleObj = RULES.find((r) => r.id === sum.rule);
  ctx.font = "700 32px " + FONT;
  ctx.fillStyle = "#475569";
  ctx.fillText(
    sum.daily ? `每日集群挑战「${ruleObj ? ruleObj.name : ""}」· 稳定运行时长` : "本局数据中心巡检时长",
    540,
    820,
  );

  ctx.font = '900 italic 176px "Avenir Next", "Arial Black", ' + FONT;
  ctx.lineWidth = 14;
  ctx.strokeStyle = "#ffffff";
  ctx.strokeText(fmtTime(sum.time), 540, 938);
  const sg = ctx.createLinearGradient(0, 860, 0, 1010);
  sg.addColorStop(0, "#1e3a8a");
  sg.addColorStop(0.6, "#2563eb");
  sg.addColorStop(1, "#0891b2");
  ctx.fillStyle = sg;
  ctx.fillText(fmtTime(sum.time), 540, 938);

  // 6. Three Crisp White Cleanroom Stat Cards
  [
    ["清除异常", String(sum.kills)],
    ["集群等级", "Lv." + sum.level],
    ["峰值连击", "×" + sum.maxCombo],
  ].forEach(([label, val], idx) => {
    const cx = 188 + idx * 352;
    roundRectPath(ctx, cx - 156, 1046, 312, 118, 16);
    ctx.fillStyle = "rgba(255, 255, 255, 0.94)";
    ctx.fill();
    ctx.strokeStyle = "#cbd5e1";
    ctx.lineWidth = 2.5;
    ctx.stroke();
    ctx.font = "900 50px " + FONT;
    ctx.fillStyle = "#0f172a";
    ctx.fillText(val, cx, 1090);
    ctx.font = "700 25px " + FONT;
    ctx.fillStyle = "#64748b";
    ctx.fillText(label, cx, 1138);
  });

  // 7. Hardware Build Slots
  const sortedW = sum.weapons.slice().sort((a, b) => b.dmg - a.dmg);
  const totalSlots = sortedW.length + sum.passives.length;
  const box = 92, gap = 14;
  const totalW = totalSlots * box + Math.max(0, totalSlots - 1) * gap + (sum.passives.length ? 18 : 0);
  let sx = 540 - totalW / 2;

  const mainW = sortedW[0]
    ? sortedW[0].evo
      ? WEAPONS[WEAPON_IDX[sortedW[0].id]].evoName
      : WEAPONS[WEAPON_IDX[sortedW[0].id]].name
    : "HBM 带宽飞梭";
  ctx.font = "800 28px " + FONT;
  ctx.fillStyle = "#334155";
  ctx.fillText(`主力硬件构筑 · 「${mainW}」`, 540, 1208);

  sortedW.forEach((w) => {
    roundRectPath(ctx, sx, 1238, box, box, 14);
    ctx.fillStyle = w.evo ? "#fef3c7" : "#ffffff";
    ctx.fill();
    ctx.strokeStyle = w.evo ? "#d97706" : "#94a3b8";
    ctx.lineWidth = w.evo ? 4 : 2.2;
    ctx.stroke();
    drawIcon(ctx, w.id, sx + box / 2, 1238 + box / 2, 70);
    ctx.font = "800 20px " + FONT;
    ctx.textAlign = "right";
    ctx.fillStyle = w.evo ? "#b45309" : "#0f172a";
    ctx.fillText(w.evo ? "超武" : "Lv" + w.lv, sx + box - 7, 1238 + box - 13);
    ctx.textAlign = "center";
    sx += box + gap;
  });

  if (sum.passives.length) sx += 18;
  sum.passives.forEach((p) => {
    roundRectPath(ctx, sx + 4, 1242, box - 8, box - 8, 14);
    ctx.fillStyle = "#f8fafc";
    ctx.fill();
    ctx.strokeStyle = "#cbd5e1";
    ctx.lineWidth = 2;
    ctx.stroke();
    drawIcon(ctx, p.id, sx + box / 2, 1238 + box / 2, 54);
    sx += box + gap;
  });

  // 8. Footer with honeclaw GitHub Repo & Star CTA
  roundRectPath(ctx, 54, 1352, 972, 48, 10);
  ctx.fillStyle = "#0f172a";
  ctx.fill();
  ctx.font = "700 22px " + FONT;
  ctx.fillStyle = "#fbbf24";
  ctx.textAlign = "left";
  ctx.fillText(`★ Star on GitHub: ${HONECLAW_REPO_LABEL}`, 76, 1376);
  ctx.fillStyle = "#e2e8f0";
  ctx.textAlign = "right";
  ctx.fillText(`hone-claw.com${sum.bosses ? ` · 击破巨兽 ×${sum.bosses}` : ""}`, 1004, 1376);

  return c;
}

export function downloadPoster(canvas, filename = "hone-adventure-report.png") {
  try {
    const url = canvas.toDataURL("image/png");
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    return true;
  } catch {
    return false;
  }
}

export async function copyPosterToClipboard(canvas, textFallback) {
  try {
    if (navigator.clipboard && window.ClipboardItem) {
      const blob = await new Promise((res) => canvas.toBlob(res, "image/png"));
      if (blob) {
        await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
        return "image";
      }
    }
    if (navigator.clipboard && navigator.clipboard.writeText) {
      await navigator.clipboard.writeText(textFallback);
      return "text";
    }
  } catch {
    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(textFallback);
        return "text";
      }
    } catch {
      // ignore
    }
  }
  return null;
}
