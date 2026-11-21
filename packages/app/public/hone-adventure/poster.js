// HONE Adventure — 1080x1440 (3:4) Battle Report Poster Generator & Share Utilities

import { RULES, WEAPONS, WEAPON_IDX, fmtTime, getTitle } from "./data.js";
import { ICON_DRAWERS, drawSprite2D, makeFallbackFloor, makeQuantOverlay, roundRectPath } from "./art.js";

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

  // Obsidian-Jade Quant Matrix Background
  ctx.fillStyle = "#081113";
  ctx.fillRect(0, 0, 1080, 1440);
  ctx.save();
  ctx.scale(2, 2);
  ctx.fillStyle = ctx.createPattern(bgFloor, "repeat");
  ctx.fillRect(0, 0, 540, 720);
  ctx.restore();

  ctx.save();
  ctx.globalCompositeOperation = "lighter";
  ctx.globalAlpha = 0.48;
  ctx.scale(4.2, 4.2);
  ctx.fillStyle = ctx.createPattern(bgQuant, "repeat");
  ctx.fillRect(0, 0, 1080 / 4.2, 1440 / 4.2);
  ctx.restore();

  // Golden-Cyan Diagonal Light Shafts
  ctx.save();
  ctx.globalCompositeOperation = "lighter";
  for (let i = 0; i < 3; i++) {
    const g = ctx.createLinearGradient(0, 0, 0, 1150);
    g.addColorStop(0, i === 1 ? "rgba(255,173,54,.16)" : "rgba(92,225,230,.12)");
    g.addColorStop(1, "rgba(0,0,0,0)");
    ctx.fillStyle = g;
    ctx.save();
    ctx.translate(1080 * (0.2 + i * 0.3), -40);
    ctx.rotate(0.28);
    ctx.fillRect(-70 - i * 20, 0, 140 + i * 40, 1280);
    ctx.restore();
  }
  ctx.restore();

  const vig = ctx.createRadialGradient(540, 620, 260, 540, 720, 1080);
  vig.addColorStop(0, "rgba(4,9,11,0)");
  vig.addColorStop(1, "rgba(4,9,11,.86)");
  ctx.fillStyle = vig;
  ctx.fillRect(0, 0, 1080, 1440);

  ctx.textAlign = "center";
  ctx.textBaseline = "middle";

  // Surrounding Market Noise Ring
  const mobs = ["e_glitch", "e_fomo", "e_spam", "e_hallucination", "e_crash", "e_bubble_mob", "e_margin_bomb", "e_rumor"];
  mobs.forEach((m, i) => {
    const a = (i / mobs.length) * Math.PI * 2 + 0.35;
    const rx = 380 + (i % 2) * 44;
    ctx.globalAlpha = 0.92;
    drawSprite2D(ctx, m, 540 + Math.cos(a) * rx, 505 + Math.sin(a) * rx * 0.58, 3.0 + (i % 3) * 0.32, 0, Math.cos(a) > 0);
  });
  ctx.globalAlpha = 1;

  // Cyber Resonance Rings + Hero Spotlight
  ctx.save();
  ctx.globalCompositeOperation = "lighter";
  for (let i = 0; i < 3; i++) {
    ctx.strokeStyle = i === 0 ? "rgba(255,173,54,.55)" : `rgba(92,225,230,${0.36 - i * 0.1})`;
    ctx.lineWidth = 9 - i * 2.5;
    ctx.beginPath();
    ctx.ellipse(540, 515, 175 + i * 64, (175 + i * 64) * 0.6, 0, 0, Math.PI * 2);
    ctx.stroke();
  }
  const glow = ctx.createRadialGradient(540, 500, 0, 540, 500, 310);
  glow.addColorStop(0, "rgba(255,173,54,.45)");
  glow.addColorStop(0.55, "rgba(46,229,157,.16)");
  glow.addColorStop(1, "rgba(0,0,0,0)");
  ctx.fillStyle = glow;
  ctx.fillRect(0, 180, 1080, 640);
  ctx.restore();

  const heroKey = sum.char === "hone" && save.skin === "gold" && save.skins.gold ? "gold" : sum.char;
  ctx.imageSmoothingQuality = "high";
  drawSprite2D(ctx, "shadow", 540, 705, 8.2, 0, false);
  drawSprite2D(ctx, "P_" + heroKey, 540, 500, 8.8, 0, false);

  // Header Brand & Title
  ctx.font = "900 64px " + FONT;
  ctx.lineJoin = "round";
  ctx.lineWidth = 14;
  ctx.strokeStyle = "rgba(5,12,14,.92)";
  ctx.strokeText("HONE 历险记", 540, 108);
  const tg = ctx.createLinearGradient(0, 74, 0, 142);
  tg.addColorStop(0, "#fffdf8");
  tg.addColorStop(0.6, "#ffd166");
  tg.addColorStop(1, "#ffad36");
  ctx.fillStyle = tg;
  ctx.fillText("HONE 历险记", 540, 108);

  const rankName = getTitle(sum.time).name;
  ctx.font = "800 33px " + FONT;
  const pillW = ctx.measureText(rankName).width + 68;
  roundRectPath(ctx, 540 - pillW / 2, 164, pillW, 56, 28);
  ctx.fillStyle = "rgba(255,173,54,.16)";
  ctx.fill();
  ctx.strokeStyle = "rgba(255,209,102,.82)";
  ctx.lineWidth = 2.5;
  ctx.stroke();
  ctx.fillStyle = "#ffe8a3";
  ctx.fillText(rankName, 540, 193);

  // Survival Duration Hero Stat
  const ruleObj = RULES.find((r) => r.id === sum.rule);
  ctx.font = "600 35px " + FONT;
  ctx.fillStyle = "#a7c4c2";
  ctx.fillText(
    sum.daily ? `每日行情「${ruleObj ? ruleObj.name : ""}」· 坚守时长` : "本局算力坚守时长",
    540,
    836,
  );

  ctx.font = '900 italic 186px "Avenir Next", "Arial Black", ' + FONT;
  ctx.lineWidth = 22;
  ctx.strokeStyle = "rgba(5,12,14,.94)";
  ctx.strokeText(fmtTime(sum.time), 540, 958);
  const sg = ctx.createLinearGradient(0, 878, 0, 1038);
  sg.addColorStop(0, "#fffbeb");
  sg.addColorStop(0.52, "#ffad36");
  sg.addColorStop(1, "#f06449");
  ctx.fillStyle = sg;
  ctx.fillText(fmtTime(sum.time), 540, 958);

  // Three Core Stat Cards
  [
    ["净除噪音", String(sum.kills)],
    ["算力等级", "Lv." + sum.level],
    ["最高连击", "×" + sum.maxCombo],
  ].forEach(([label, val], idx) => {
    const cx = 180 + idx * 360;
    roundRectPath(ctx, cx - 160, 1074, 320, 124, 24);
    ctx.fillStyle = "rgba(11,25,28,.82)";
    ctx.fill();
    ctx.strokeStyle = "rgba(92,225,230,.28)";
    ctx.lineWidth = 2;
    ctx.stroke();
    ctx.font = "900 54px " + FONT;
    ctx.fillStyle = "#ffffff";
    ctx.fillText(val, cx, 1120);
    ctx.font = "600 27px " + FONT;
    ctx.fillStyle = "#94b4b2";
    ctx.fillText(label, cx, 1170);
  });

  // Top Build Slots
  const sortedW = sum.weapons.slice().sort((a, b) => b.dmg - a.dmg);
  const totalSlots = sortedW.length + sum.passives.length;
  const box = 98, gap = 16;
  const totalW = totalSlots * box + Math.max(0, totalSlots - 1) * gap + (sum.passives.length ? 20 : 0);
  let sx = 540 - totalW / 2;

  const mainW = sortedW[0]
    ? sortedW[0].evo
      ? WEAPONS[WEAPON_IDX[sortedW[0].id]].evoName
      : WEAPONS[WEAPON_IDX[sortedW[0].id]].name
    : "研报飞刃";
  ctx.font = "700 30px " + FONT;
  ctx.fillStyle = "#bfe3e0";
  ctx.fillText(`核心构筑 · 主力「${mainW}」`, 540, 1244);

  sortedW.forEach((w) => {
    roundRectPath(ctx, sx, 1278, box, box, 22);
    ctx.fillStyle = w.evo ? "rgba(255,173,54,.22)" : "rgba(11,25,28,.86)";
    ctx.fill();
    ctx.strokeStyle = w.evo ? "#ffd166" : "rgba(92,225,230,.36)";
    ctx.lineWidth = w.evo ? 4.5 : 2;
    ctx.stroke();
    drawIcon(ctx, w.id, sx + box / 2, 1278 + box / 2, 78);
    ctx.font = "800 22px " + FONT;
    ctx.textAlign = "right";
    ctx.strokeStyle = "rgba(5,12,14,.92)";
    ctx.lineWidth = 5;
    const tag = w.evo ? "超武" : "Lv" + w.lv;
    ctx.strokeText(tag, sx + box - 7, 1278 + box - 14);
    ctx.fillStyle = w.evo ? "#ffe8a3" : "#f3f7f6";
    ctx.fillText(tag, sx + box - 7, 1278 + box - 14);
    ctx.textAlign = "center";
    sx += box + gap;
  });

  if (sum.passives.length) sx += 20;
  sum.passives.forEach((p) => {
    ctx.beginPath();
    ctx.arc(sx + box / 2, 1278 + box / 2, box / 2 - 4, 0, Math.PI * 2);
    ctx.fillStyle = "rgba(11,25,28,.86)";
    ctx.fill();
    ctx.strokeStyle = "rgba(92,225,230,.36)";
    ctx.lineWidth = 2;
    ctx.stroke();
    drawIcon(ctx, p.id, sx + box / 2, 1278 + box / 2, 58);
    sx += box + gap;
  });

  const now = new Date();
  ctx.font = "600 25px " + FONT;
  ctx.fillStyle = "rgba(167,196,194,.78)";
  ctx.textAlign = "left";
  ctx.fillText("HONE · 去伪存真的投研智能体 | hone-claw.com", 48, 1410);
  ctx.textAlign = "right";
  ctx.fillText(
    `${now.getFullYear()}.${now.getMonth() + 1}.${now.getDate()}${sum.bosses ? ` · 击破巨兽 ×${sum.bosses}` : ""}`,
    1032,
    1410,
  );
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
