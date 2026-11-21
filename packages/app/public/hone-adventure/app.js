// HONE Adventure V2 — Cleanroom AI Data Center Shell, 6-Stage Switcher, Promotion Roadmap & GitHub Star CTA

import {
  ACHIEVEMENTS,
  BOOSTS,
  BOSS_IDS,
  CODEX_ENEMY_IDS,
  COMBO_CALLOUTS,
  ENEMIES,
  ENEMY_IDX,
  HEROES,
  HERO_IDX,
  HONECLAW_GITHUB_URL,
  HONECLAW_REPO_LABEL,
  PASSIVES,
  PASSIVE_IDX,
  RULES,
  STAGES,
  STAGE_IDX,
  TALENTS,
  TALENT_BRANCHES,
  TITLES,
  VERSION,
  WEAPONS,
  WEAPON_IDX,
  branchPoints,
  dateKey,
  fmtTime,
  getDailyChallenge,
  getNextTitle,
  getTitle,
} from "./data.js";
import { GameWorld, createAutoBot } from "./world.js";
import {
  ART_STATE,
  SPRITE_DEFS,
  SPR_IDX,
  bindSpriteSheet,
  buildAtlas,
  createRenderer,
  drawSprite2D,
  loadImage,
  makeFallbackFloor,
  makeQuantOverlay,
  renderIconCanvas,
} from "./art.js";
import {
  playSfx,
  setAudioPrefs,
  setBgmIntensity,
  startBgm,
  stopBgm,
  tickBgm,
  vibrate,
  warmAudioStep,
} from "./audio.js";
import { ZONES, cloneZone, createFxSystem, lerpZone, pickZoneKey, renderWorld } from "./fx.js";
import { copyPosterToClipboard, downloadPoster, renderBattlePoster } from "./poster.js";

const SAVE_KEY = "hone_adventure_save_v2";
const GITHUB_API_URL = "https://api.github.com/repos/B-M-Capital-Research/honeclaw";
const GITHUB_STARS_CACHE_KEY = "hone.github.stars.v2";

let ghStarsLabel = "742";
try {
  const cached = JSON.parse(localStorage.getItem(GITHUB_STARS_CACHE_KEY) || "null");
  if (cached && typeof cached.value === "string" && cached.value) ghStarsLabel = cached.value;
} catch {
  // ignore
}

function refreshGithubStars() {
  fetch(GITHUB_API_URL, { headers: { Accept: "application/vnd.github+json" } })
    .then((r) => (r.ok ? r.json() : null))
    .then((data) => {
      if (data && typeof data.stargazers_count === "number") {
        const n = data.stargazers_count;
        ghStarsLabel = n >= 1000 ? (n / 1000).toFixed(1) + "k" : String(n);
        try {
          localStorage.setItem(GITHUB_STARS_CACHE_KEY, JSON.stringify({ value: ghStarsLabel, cachedAt: Date.now() }));
        } catch {
          // ignore
        }
        const badge = document.querySelector(".gh-stars-cnt");
        if (badge) badge.textContent = ghStarsLabel;
      }
    })
    .catch(() => {});
}

function defaultSave() {
  return {
    v: 2,
    t: 0,
    cores: 0,
    talents: {},
    chars: { hone: 1 },
    sel: "hone",
    stage: "storage",
    stageClear: {},
    promoClaimed: 1,
    skins: {},
    skin: "",
    ach: {},
    codex: { w: {}, evo: {}, e: {} },
    best: { time: 0, kills: 0, level: 0, combo: 0 },
    stats: { runs: 0, kills: 0, time: 0, bosses: 0, evos: 0, dailyDone: 0, bestTime: 0 },
    daily: {},
    streak: { n: 0, last: "" },
    hof: [],
    set: { sfx: 1, music: 1, vib: 1, shake: 1 },
    gift: 0,
    starred: 0,
  };
}

function loadSave() {
  const base = defaultSave();
  try {
    const raw = localStorage.getItem(SAVE_KEY);
    if (!raw) return base;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return base;
    const out = Object.assign(base, parsed);
    ["talents", "chars", "skins", "ach", "best", "stats", "streak", "set", "daily", "stageClear"].forEach((k) => {
      out[k] = Object.assign(defaultSave()[k], parsed[k] && typeof parsed[k] === "object" ? parsed[k] : {});
    });
    const cleanW = {}, cleanEvo = {}, cleanE = {};
    if (parsed.codex && parsed.codex.w) {
      for (const k of Object.keys(parsed.codex.w)) if (WEAPON_IDX[k] != null) cleanW[k] = 1;
    }
    if (parsed.codex && parsed.codex.evo) {
      for (const k of Object.keys(parsed.codex.evo)) if (WEAPON_IDX[k] != null) cleanEvo[k] = 1;
    }
    if (parsed.codex && parsed.codex.e) {
      for (const k of Object.keys(parsed.codex.e)) if (ENEMY_IDX[k] != null) cleanE[k] = parsed.codex.e[k];
    }
    out.codex = { w: cleanW, evo: cleanEvo, e: cleanE };
    out.hof = Array.isArray(parsed.hof) ? parsed.hof.slice(0, 10) : [];
    out.chars.hone = 1;
    if (!out.chars[out.sel]) out.sel = "hone";
    if (STAGE_IDX[out.stage] == null) out.stage = "storage";
    if (out.skin && !out.skins[out.skin]) out.skin = "";
    out.cores = Math.max(0, Math.floor(+out.cores || 0));
    out.promoClaimed = Math.max(1, Math.floor(+out.promoClaimed || 1));
    return out;
  } catch {
    return base;
  }
}

function persistSave(save) {
  save.t = Date.now();
  try {
    localStorage.setItem(SAVE_KEY, JSON.stringify(save));
  } catch {
    // ignore storage quota errors
  }
}

// DOM & Icon Cache Helpers
const $ = (sel, root = document) => root.querySelector(sel);
function el(tag, cls, html) {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (html != null) node.innerHTML = html;
  return node;
}

const iconCache = {};
function iconUrl(id, sz = 96) {
  const key = id + "@" + sz;
  if (!iconCache[key]) iconCache[key] = renderIconCanvas(id, sz).toDataURL("image/png");
  return iconCache[key];
}

function iconImg(id, cls = "", sz = 96) {
  const img = el("img", ("ic " + cls).trim());
  img.src = iconUrl(id, sz);
  img.alt = "";
  img.draggable = false;
  return img;
}

function heroUrl(id, sz = 160, locked = false, suffix = "", portrait = false) {
  const sprKey = (portrait ? "P_" : "h_") + id + (portrait ? "" : suffix);
  const cacheKey = `hero:${sprKey}@${sz}${locked ? ":lk" : ""}`;
  if (!iconCache[cacheKey]) {
    const def = SPRITE_DEFS[SPR_IDX[sprKey] ?? SPR_IDX["h_" + id]];
    const c = document.createElement("canvas");
    c.width = sz;
    c.height = sz;
    const ctx = c.getContext("2d");
    const sc = (sz * 0.92) / Math.max(def.lw || def.w, def.lh || def.h);
    ctx.imageSmoothingQuality = "high";
    ctx.translate(sz / 2, sz / 2);
    ctx.scale(sc, sc);
    def.f(ctx, sc);
    if (locked) {
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.globalCompositeOperation = "source-atop";
      ctx.fillStyle = "rgba(100,116,139,.82)";
      ctx.fillRect(0, 0, sz, sz);
    }
    iconCache[cacheKey] = c.toDataURL("image/png");
  }
  return iconCache[cacheKey];
}

function spriteUrl(sprName, sz = 112, locked = false) {
  const cacheKey = `spr:${sprName}@${sz}${locked ? ":lk" : ""}`;
  if (!iconCache[cacheKey]) {
    const def = SPRITE_DEFS[SPR_IDX[sprName]];
    const c = document.createElement("canvas");
    c.width = sz;
    c.height = sz;
    const ctx = c.getContext("2d");
    const sc = (sz * 0.9) / Math.max(def.lw || def.w, def.lh || def.h);
    ctx.imageSmoothingQuality = "high";
    drawSprite2D(ctx, sprName, sz / 2, sz / 2, sc, 0, false);
    if (locked) {
      ctx.globalCompositeOperation = "source-atop";
      ctx.fillStyle = "rgba(100,116,139,.85)";
      ctx.fillRect(0, 0, sz, sz);
    }
    iconCache[cacheKey] = c.toDataURL("image/png");
  }
  return iconCache[cacheKey];
}

let toastEl = null;
let toastTimer = 0;
function showToast(msg, dur = 1900) {
  if (!toastEl) {
    toastEl = el("div", "toast");
    document.body.appendChild(toastEl);
  }
  toastEl.textContent = msg;
  toastEl.classList.add("on");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toastEl.classList.remove("on"), dur);
}

function bindTap(node, fn, opts = {}) {
  let down = null;
  let lastFire = 0;
  node.addEventListener("pointerdown", (e) => {
    if (e.button > 0) return;
    down = { x: e.clientX, y: e.clientY };
    node.classList.add("press");
  });
  const clear = () => {
    down = null;
    node.classList.remove("press");
  };
  node.addEventListener("pointercancel", clear);
  node.addEventListener("pointerleave", () => node.classList.remove("press"));
  node.addEventListener("pointerup", (e) => {
    if (!down) return;
    const moved = Math.abs(e.clientX - down.x) + Math.abs(e.clientY - down.y);
    clear();
    if (moved < 18) {
      lastFire = Date.now();
      if (!opts.quiet) playSfx("tap");
      fn && fn(e);
    }
  });
  node.addEventListener("click", (e) => {
    if (Date.now() - lastFire < 550) return;
    lastFire = Date.now();
    if (!opts.quiet) playSfx("tap");
    fn && fn(e);
  });
  return node;
}

function makeBtn(cls, html, fn, opts) {
  const b = el("div", ("btn " + (cls || "")).trim(), html);
  return bindTap(b, fn, opts);
}

function makeHomepageLink() {
  const a = el("a", "home-pill");
  a.href = "/";
  a.title = "回到 HONE 首页";
  a.appendChild(iconImg("ui_home", "", 48));
  a.appendChild(el("span", "", "回到首页"));
  a.addEventListener("click", (e) => {
    if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    window.location.href = "/";
  });
  return a;
}

const openModals = [];
function createModal(opts = {}) {
  const mask = el("div", "modal " + (opts.cls || ""));
  const box = el("div", "box");
  mask.appendChild(box);
  let closed = false;
  const close = () => {
    if (closed) return;
    closed = true;
    mask.classList.remove("on");
    const idx = openModals.indexOf(api);
    if (idx >= 0) openModals.splice(idx, 1);
    setTimeout(() => mask.parentNode && mask.parentNode.removeChild(mask), 200);
    opts.onClose && opts.onClose();
  };
  if (opts.title) {
    const head = el("div", "box-h");
    head.appendChild(el("div", "box-t", opts.title));
    if (!opts.noClose) {
      const xBtn = el("div", "box-x");
      xBtn.appendChild(iconImg("ui_x", "", 48));
      bindTap(xBtn, close);
      head.appendChild(xBtn);
    }
    box.appendChild(head);
  }
  const body = el("div", "box-b");
  box.appendChild(body);
  if (opts.dismiss) {
    bindTap(mask, (e) => {
      if (e.target === mask) close();
    });
  }
  document.body.appendChild(mask);
  setTimeout(() => mask.classList.add("on"), 16);
  const api = { el: mask, box, body, close, isOpen: () => !closed };
  openModals.push(api);
  return api;
}

function closeAllModals() {
  openModals.slice().forEach((m) => m.close());
}

// Progression & Settlement Helpers
function achProgress(save, ach) {
  if (ach.stat) return [save.stats[ach.stat] || 0, ach.n];
  if (ach.run) {
    const val =
      ach.run === "time" ? Math.floor(save.best.time) :
      ach.run === "kills" ? save.best.kills :
      ach.run === "level" ? save.best.level : save.best.combo;
    return [val, ach.n];
  }
  if (ach.codex) return [save.codex.e[ach.codex] ? 1 : 0, 1];
  if (ach.evoAll) return [Object.keys(save.codex.evo).length, ach.n];
  return [0, ach.n];
}

function heroProgress(save, hero) {
  if (!hero.unlock) return [1, 1];
  const cur = hero.unlock.stat === "bestTime" ? Math.floor(save.best.time) : save.stats[hero.unlock.stat] || 0;
  return [cur, hero.unlock.n];
}

function talentNextCost(save, tal) {
  const lv = save.talents[tal.id] || 0;
  return lv >= tal.max ? -1 : tal.cost[lv];
}

function nextGoalInfo(save) {
  const nt = getNextTitle(save.best.time);
  if (nt) {
    return {
      text: `晋升「${nt.name}」(${nt.perk})`,
      cur: Math.floor(save.best.time),
      n: nt.t,
      ratio: Math.min(1, save.best.time / nt.t),
      go: "promo",
    };
  }
  for (const tal of TALENTS) {
    const cost = talentNextCost(save, tal);
    if (cost > 0 && save.cores >= cost) {
      return { text: `晶核充足，升级「${tal.name}」`, cur: 1, n: 1, ratio: 1, go: "talent" };
    }
  }
  return { text: "已达首席算力官！冲击荣耀殿堂新纪录", cur: 1, n: 1, ratio: 1, go: "rank" };
}

function settleRun(save, sum) {
  const res = {
    base: sum.cores,
    bonus: [],
    total: 0,
    newAch: [],
    unlocked: [],
    bestTime: false,
    bestKills: false,
    titleUp: null,
  };
  const prevTitle = getTitle(save.best.time);

  save.stats.runs++;
  save.stats.kills += sum.kills;
  save.stats.time += sum.time;
  save.stats.bosses += sum.bosses;
  save.stats.evos += sum.evos.length;

  if (sum.time >= 60 && sum.stage) {
    save.stageClear[sum.stage] = Math.max(save.stageClear[sum.stage] || 0, Math.floor(sum.time));
  }

  if (sum.time > save.best.time) {
    res.bestTime = save.best.time > 0;
    save.best.time = sum.time;
  }
  if (sum.kills > save.best.kills) {
    res.bestKills = save.best.kills > 0;
    save.best.kills = sum.kills;
  }
  if (sum.level > save.best.level) save.best.level = sum.level;
  if (sum.maxCombo > save.best.combo) save.best.combo = sum.maxCombo;
  save.stats.bestTime = save.best.time;

  sum.weapons.forEach((w) => {
    if (WEAPON_IDX[w.id] != null) {
      save.codex.w[w.id] = 1;
      if (w.evo) save.codex.evo[w.id] = 1;
    }
  });
  sum.evos.forEach((id) => {
    if (WEAPON_IDX[id] != null) save.codex.evo[id] = 1;
  });
  sum.killsBy.forEach((cnt, idx) => {
    if (cnt > 0 && ENEMIES[idx] && !ENEMIES[idx].hide) {
      const id = ENEMIES[idx].id;
      save.codex.e[id] = (save.codex.e[id] || 0) + cnt;
    }
  });

  if (sum.bosses > 0) res.bonus.push({ label: `击破产线巨兽 ×${sum.bosses}`, n: 40 * sum.bosses });

  if (sum.daily) {
    const dk = dateKey();
    const rec = save.daily[dk] || (save.daily[dk] = { time: 0, kills: 0, rewarded: 0 });
    if (sum.time > rec.time) rec.time = sum.time;
    if (sum.kills > rec.kills) rec.kills = sum.kills;
    if (!rec.rewarded && sum.time >= 120) {
      rec.rewarded = 1;
      save.stats.dailyDone++;
      const yest = dateKey(new Date(Date.now() - 86400000));
      save.streak.n = save.streak.last === yest ? save.streak.n + 1 : 1;
      save.streak.last = dk;
      res.bonus.push({ label: "每日集群达标", n: 120 });
      if (save.streak.n > 1) res.bonus.push({ label: `连续值班 ${save.streak.n} 天`, n: Math.min(120, save.streak.n * 15) });
    }
  }

  // Promotion Rank Milestone Rewards
  const nextTitle = getTitle(save.best.time);
  if (nextTitle.rank > prevTitle.rank) {
    res.titleUp = nextTitle.name;
    let promoCores = 0;
    for (const item of TITLES) {
      if (item.rank > (save.promoClaimed || 1) && item.rank <= nextTitle.rank && item.reward) {
        promoCores += item.reward;
      }
    }
    save.promoClaimed = Math.max(save.promoClaimed || 1, nextTitle.rank);
    if (promoCores > 0) res.bonus.push({ label: `职级晋升奖励`, n: promoCores });
  }

  for (const a of ACHIEVEMENTS) {
    if (save.ach[a.id]) continue;
    const [cur, need] = achProgress(save, a);
    if (cur >= need) {
      save.ach[a.id] = 1;
      res.newAch.push(a);
      res.bonus.push({ label: `勋章「${a.name}」`, n: a.cores });
    }
  }
  for (const h of HEROES) {
    if (save.chars[h.id] || !h.unlock) continue;
    const [cur, need] = heroProgress(save, h);
    if (cur >= need) {
      save.chars[h.id] = 1;
      res.unlocked.push(h);
    }
  }

  const topWeapon = sum.weapons.slice().sort((a, b) => b.dmg - a.dmg)[0];
  const stg = STAGES[STAGE_IDX[sum.stage || "storage"]] || STAGES[0];
  save.hof.push({
    time: Math.floor(sum.time),
    kills: sum.kills,
    level: sum.level,
    char: sum.char,
    stage: stg.short,
    top: topWeapon && WEAPONS[WEAPON_IDX[topWeapon.id]]
      ? (topWeapon.evo ? WEAPONS[WEAPON_IDX[topWeapon.id]].evoName : WEAPONS[WEAPON_IDX[topWeapon.id]].name)
      : "HBM 带宽飞梭",
    daily: sum.daily,
    date: dateKey(),
  });
  save.hof.sort((a, b) => b.time - a.time || b.kills - a.kills);
  if (save.hof.length > 10) save.hof.length = 10;

  res.total = res.base + res.bonus.reduce((acc, b) => acc + b.n, 0);
  save.cores += res.total;
  return res;
}

// Global Engine State
const G = {
  mode: "boot",
  S: null,
  world: null,
  fx: null,
  R: null,
  atlas: null,
  floorImg: null,
  bot: null,
  cw: 390,
  ch: 844,
  sc: 1,
  dpr: 1.5,
  dprMax: 2,
  fixedQ: false,
  time: 0,
  last: 0,
  camX: 0,
  camY: 0,
  zoom: 0,
  joy: { on: false, bx: 0, by: 0, kx: 0, ky: 0, mx: 0, my: 0 },
  keys: {},
  paused: false,
  hitstop: 0,
  slowT: 0,
  slowS: 1,
  lvRamp: 0,
  lvArmed: false,
  deadT: 0,
  resumeT: 0,
  hurtT: 0,
  joyT: 0,
  joyK: 0,
  zone: cloneZone("storage"),
  zoneId: "storage",
  hero: "hone",
  run: null,
  perf: { ms: 0, fps: 60, n: 0, acc: 0, accMs: 0, low: 0, high: 0 },
};

function activeHeroSkin(charId) {
  return charId === "hone" && G.S.skin === "gold" && G.S.skins.gold ? "gold" : charId;
}

let homeEl, resultEl, hudEl, stageEl;
let homeRefs = {};
let hudRefs = {};
let levelModal = null;
let chestModal = null;
let pauseModal = null;

function selectStage(stageId) {
  if (STAGE_IDX[stageId] == null) return false;
  G.S.stage = stageId;
  persistSave(G.S);
  if (G.world && G.mode === "home") {
    G.world.startStageIdx = STAGE_IDX[stageId];
    G.world.setStage(stageId);
  }
  refreshHomeUI();
  return true;
}

function buildHomeUI() {
  homeEl = el("div", "home hide");
  homeEl.innerHTML = `
    <div class="h-top">
      <div class="h-top-left"></div>
      <div class="h-top-right">
        <div class="gift"></div>
        <div class="chip cores"><img alt=""><b>0</b></div>
        <div class="hb gear" title="系统设置"></div>
      </div>
    </div>
    <div class="logo">
      <div class="lg-badge">HONE CLEANROOM · AI DATA CENTER</div>
      <div class="lg1">HONE <i>历险记</i></div>
      <div class="lg2">穿梭算力产线 · 横扫硬件瓶颈</div>
    </div>
    <div class="h-bot">
      <div class="sec-head"><b>算力产线关卡切换</b><span class="st-sub"></span></div>
      <div class="stage-strip"></div>
      <div class="promo-bar">
        <div class="pb-l">
          <div class="pb-top"><span class="pb-rk"></span><span class="pb-goal"></span></div>
          <div class="g-bar"><i></i></div>
        </div>
        <div class="pb-go">晋升路线 →</div>
      </div>
      <div class="launch-row">
        <div class="cta"><img class="cta-h" alt=""><div class="cta-t"><b>进入产线巡检</b><small></small></div></div>
        <div class="daily"></div>
      </div>
      <div class="nav"></div>
    </div>
  `;
  document.body.appendChild(homeEl);

  $(".h-top-left", homeEl).appendChild(makeHomepageLink());

  homeRefs = {
    cores: $(".cores b", homeEl),
    gift: $(".gift", homeEl),
    stSub: $(".st-sub", homeEl),
    stStrip: $(".stage-strip", homeEl),
    pbRk: $(".pb-rk", homeEl),
    pbGoal: $(".pb-goal", homeEl),
    pbBar: $(".promo-bar .g-bar i", homeEl),
    ctaS: $(".cta small", homeEl),
    ctaH: $(".cta-h", homeEl),
    daily: $(".daily", homeEl),
    nav: $(".nav", homeEl),
  };

  $(".cores img", homeEl).src = iconUrl("core", 64);
  bindTap($(".cores", homeEl), openTalents);

  const gear = $(".gear", homeEl);
  gear.appendChild(iconImg("ui_gear", "", 48));
  bindTap(gear, openSettings);

  STAGES.forEach((st) => {
    const pill = el("div", "st-pill", `<small>${st.code.replace("SECTOR-", "S")}</small><b>${st.short}</b>`);
    pill.dataset.stage = st.id;
    bindTap(pill, () => {
      selectStage(st.id);
      showToast(`已切换产线：${st.name}（${st.sub}）`, 1800);
    });
    homeRefs.stStrip.appendChild(pill);
  });

  bindTap($(".promo-bar", homeEl), openPromotionModal);

  bindTap($(".cta", homeEl), () => {
    playSfx("select");
    startRun({});
  }, { quiet: true });

  bindTap(homeRefs.daily, openDailyModal);

  [
    ["基建", "ui_up", openTalents, "n-up"],
    ["特工", "ui_agent", openHeroes, "n-ch"],
    ["晋升", "ui_rank", openPromotionModal, "n-pm"],
    ["图鉴", "ui_book", openCodex, "n-cx"],
    ["勋章", "ui_medal", openAchievements, "n-ac"],
    ["殿堂", "ui_medal", openHallOfFame, "n-rk"],
  ].forEach(([label, ic, fn, cls]) => {
    const nb = el("div", "nb " + cls);
    nb.appendChild(iconImg(ic, "", 64));
    nb.appendChild(el("span", "", label));
    nb.appendChild(el("i", "dot"));
    bindTap(nb, fn);
    homeRefs.nav.appendChild(nb);
  });

  bindTap(homeRefs.gift, () => {
    if (!G.S.skins.gold) {
      G.S.gift = 1;
      G.S.skins.gold = 1;
      G.S.skin = "gold";
      G.S.sel = "hone";
      persistSave(G.S);
      playSfx("evo");
      showToast("已领取限定传说涂装「曜金小砺」！", 2400);
      G.hero = activeHeroSkin(G.S.sel);
      refreshHomeUI();
    } else {
      G.S.skin = G.S.skin === "gold" ? "" : "gold";
      G.S.sel = "hone";
      persistSave(G.S);
      playSfx("select");
      G.hero = activeHeroSkin(G.S.sel);
      showToast(G.S.skin === "gold" ? "已穿戴「曜金小砺」" : "已切换为标准「小砺」");
      refreshHomeUI();
    }
  }, { quiet: true });

  resultEl = el("div", "result hide");
  document.body.appendChild(resultEl);
}

function refreshHomeUI() {
  const S = G.S;
  const g = nextGoalInfo(S);
  const hero = HEROES[HERO_IDX[S.sel]];
  const curStage = STAGES[STAGE_IDX[S.stage]] || STAGES[0];
  const curTitle = getTitle(S.best.time);

  homeRefs.cores.textContent = S.cores;
  homeRefs.stSub.textContent = `${curStage.name} · ${curStage.sub.split(" · ")[1] || curStage.sub}`;

  homeRefs.stStrip.querySelectorAll(".st-pill").forEach((node) => {
    node.classList.toggle("on", node.dataset.stage === curStage.id);
  });

  homeRefs.pbRk.textContent = curTitle.name;
  homeRefs.pbGoal.textContent = g.text;
  homeRefs.pbBar.style.transform = `scaleX(${Math.max(0.04, Math.min(1, g.ratio))})`;

  homeRefs.ctaS.textContent = `${curStage.short} · ${hero.name}（${WEAPONS[WEAPON_IDX[hero.weapon]].name}）`;
  homeRefs.ctaH.src = heroUrl(activeHeroSkin(S.sel), 160);

  homeRefs.gift.classList.toggle("claimed", !!S.skins.gold);
  homeRefs.gift.innerHTML = `<img class="gf-i" alt="" src="${heroUrl("gold", 96)}"><div class="gf-t"><b>${
    S.skins.gold ? (S.skin === "gold" ? "曜金出战" : "换曜金皮") : "领曜金皮"
  }</b><small>${S.skins.gold ? "点击切换" : "免费解锁"}</small></div>`;

  const d = getDailyChallenge(dateKey());
  const dRec = S.daily[d.key];
  const rule = RULES.find((r) => r.id === d.rule);
  const dStg = STAGES[STAGE_IDX[d.stage]] || STAGES[0];
  homeRefs.daily.innerHTML = `<img alt="" src="${iconUrl("ui_cal", 64)}"><div class="d-t"><b>每日「${rule.name}」</b><small>${
    dRec && dRec.rewarded ? `今日已达标 · ${fmtTime(dRec.time)}` : `${dStg.short} · 满 2 分钟 +120 晶核`
  }</small></div>`;
  homeRefs.daily.classList.toggle("done", !!(dRec && dRec.rewarded));

  let canUp = false;
  for (const t of TALENTS) {
    const cost = talentNextCost(S, t);
    if (cost > 0 && S.cores >= cost) canUp = true;
  }
  $(".n-up", homeEl).classList.toggle("has", canUp);

  let canHero = false;
  for (const h of HEROES) if (!S.chars[h.id] && S.cores >= h.price) canHero = true;
  $(".n-ch", homeEl).classList.toggle("has", canHero);
}

// 4-Pillar × 3-Node = 12-Node AI Infrastructure Upgrade Matrix Modal
function openTalents() {
  const S = G.S;
  let activeBranch = "interconnect";
  const m = createModal({ cls: "panel talents", title: "四维算力基建升级矩阵", dismiss: true, onClose: refreshHomeUI });
  const top = el("div", "p-cores");
  const tabs = el("div", "btabs");
  const syn = el("div", "b-syn");
  const list = el("div", "tlist");
  m.body.append(top, tabs, syn, list);

  const render = () => {
    top.innerHTML = `<img alt="" src="${iconUrl("core", 64)}"><b>${S.cores}</b><span>算力晶核 · 清除产线异常均可收获</span>`;
    tabs.innerHTML = "";
    TALENT_BRANCHES.forEach((b) => {
      const pts = branchPoints(S.talents, b.id);
      const tab = el("div", "btab" + (activeBranch === b.id ? " on" : ""), `${b.name}<small>已投 ${pts} 点</small>`);
      bindTap(tab, () => {
        activeBranch = b.id;
        render();
      });
      tabs.appendChild(tab);
    });

    const curB = TALENT_BRANCHES.find((b) => b.id === activeBranch);
    const pts = branchPoints(S.talents, activeBranch);
    const unlockedSyn = pts >= curB.milestone;
    syn.className = "b-syn" + (unlockedSyn ? " active" : "");
    syn.innerHTML = `<span>${curB.sub}</span><b>${unlockedSyn ? "✓ 已激活" : `${pts} / ${curB.milestone}`}</b>`;

    list.innerHTML = "";
    TALENTS.filter((t) => t.branch === activeBranch).forEach((t) => {
      const lv = S.talents[t.id] || 0;
      const cost = talentNextCost(S, t);
      const afford = cost > 0 && S.cores >= cost;
      const card = el("div", "tcard" + (cost < 0 ? " max" : afford ? " can" : ""));
      card.appendChild(iconImg(t.icon, "t-ic", 96));
      let pips = "";
      for (let i = 0; i < t.max; i++) pips += `<i class="${i < lv ? "on" : ""}"></i>`;
      card.appendChild(el("div", "t-tx", `<b>${t.name}</b><small>${t.per}</small><div class="pips">${pips}</div>`));
      card.appendChild(el("div", "t-buy", cost < 0 ? "已满级" : `<img alt="" src="${iconUrl("core", 48)}">${cost}`));
      bindTap(card, () => {
        if (cost < 0) return;
        if (!afford) {
          playSfx("err");
          showToast(`还差 ${cost - S.cores} 枚算力晶核`);
          return;
        }
        S.cores -= cost;
        S.talents[t.id] = lv + 1;
        persistSave(S);
        playSfx("buy");
        if (S.set.vib) vibrate("light");
        render();
      }, { quiet: true });
      list.appendChild(card);
    });
  };
  render();
}

// 10-Tier AI Compute Architect Promotion Roadmap Modal
function openPromotionModal() {
  const S = G.S;
  const cur = getTitle(S.best.time);
  const m = createModal({ cls: "panel promo", title: "十阶算力架构师晋升路线", dismiss: true });

  const heroBox = el("div", "rk-hero");
  heroBox.innerHTML = `
    <div><span>当前最高职级</span><b style="font-size:20px">${cur.name}</b></div>
    <div style="text-align:right"><span>单局最长稳定运行</span><b>${S.best.time ? fmtTime(S.best.time) : "00:00"}</b></div>
  `;
  m.body.appendChild(heroBox);
  m.body.appendChild(el("div", "sec", "职级晋升阶梯（达成时长永久激活全局被动特权 + 晋升晶核）"));

  TITLES.forEach((item) => {
    const unlocked = S.best.time >= item.t;
    const isCur = item.rank === cur.rank;
    const row = el("div", "pm-row" + (unlocked ? " on" : "") + (isCur ? " cur" : ""));
    row.innerHTML = `
      <div class="pm-rk">L${item.rank}</div>
      <div class="pm-tx">
        <b>${item.name.replace(/^L\d+\s*·\s*/, "")}${isCur ? " · 当前职级" : ""}</b>
        <small>${item.perk}${item.reward ? ` · 首达奖励 +${item.reward} 晶核` : ""}</small>
      </div>
      <div class="pm-req">${unlocked ? "✓ 已达成" : `需坚守 ${fmtTime(item.t)}`}</div>
    `;
    m.body.appendChild(row);
  });
}

function openHeroes() {
  const S = G.S;
  const m = createModal({
    cls: "panel chars",
    title: "HONE 算力特工编队",
    dismiss: true,
    onClose: () => {
      G.hero = activeHeroSkin(S.sel);
      if (G.mode === "home") startHomeDemo();
      refreshHomeUI();
    },
  });
  const list = el("div", "clist");
  m.body.appendChild(list);

  const render = () => {
    list.innerHTML = "";
    HEROES.forEach((h) => {
      const owned = !!S.chars[h.id];
      const sel = S.sel === h.id;
      const visId = h.id === "hone" && S.skin === "gold" && S.skins.gold ? "gold" : h.id;
      const card = el("div", "ccard" + (sel ? " sel" : "") + (owned ? "" : " lock"));
      const av = el("img", "c-av");
      av.alt = "";
      av.src = heroUrl(visId, 192, !owned);
      card.appendChild(av);

      const tx = el(
        "div",
        "c-tx",
        `<b>${h.name}<small>${h.kind}</small></b><span>${h.trait}</span><span class="wp"><img alt="" src="${iconUrl(h.weapon, 48)}">起手硬件：${WEAPONS[WEAPON_IDX[h.weapon]].name}</span>`,
      );
      card.appendChild(tx);

      if (owned) {
        card.appendChild(el("div", "c-st", sel ? "执勤中" : "派遣"));
        bindTap(card, (e) => {
          if (e && e.target && e.target.closest && e.target.closest(".skins")) return;
          if (S.sel !== h.id) {
            S.sel = h.id;
            persistSave(S);
            playSfx("select");
            render();
          }
        }, { quiet: true });

        if (h.id === "hone" && S.skins.gold) {
          const skRow = el("div", "skins");
          [["", "标准涂装"], ["gold", "曜金传说"]].forEach(([val, label]) => {
            const chip = el("div", "sk" + (S.skin === val ? " on" : ""), label);
            bindTap(chip, () => {
              S.skin = val;
              S.sel = "hone";
              persistSave(S);
              playSfx("select");
              render();
            }, { quiet: true });
            skRow.appendChild(chip);
          });
          tx.appendChild(skRow);
        }
      } else {
        const [cur, need] = heroProgress(S, h);
        const afford = S.cores >= h.price;
        tx.appendChild(
          el(
            "div",
            "c-lock",
            `<div class="g-bar" style="margin-top:5px"><i style="transform:scaleX(${Math.max(0.03, Math.min(1, cur / need))})"></i></div><small style="font-size:10.5px;color:#64748b">${h.unlock.text}（${h.unlock.stat === "bestTime" ? `${fmtTime(cur)} / ${fmtTime(need)}` : `${cur} / ${need}`}）</small>`,
          ),
        );
        card.appendChild(
          el("div", "c-st buy" + (afford ? " can" : ""), `<img alt="" src="${iconUrl("core", 48)}">${h.price}<small>直接招募</small>`),
        );
        bindTap(card, () => {
          if (!afford) {
            playSfx("err");
            showToast(`达成里程碑可免费解锁；直接招募还差 ${h.price - S.cores} 晶核`);
            return;
          }
          S.cores -= h.price;
          S.chars[h.id] = 1;
          S.sel = h.id;
          persistSave(S);
          playSfx("buy");
          showToast(`特工「${h.name}」已加入编队！`);
          render();
        }, { quiet: true });
      }
      list.appendChild(card);
    });
  };
  render();
}

function openCodex() {
  const S = G.S;
  const gotW = Object.keys(S.codex.w).length;
  const gotE = Object.keys(S.codex.evo).length;
  const gotM = Object.keys(S.codex.e).filter((k) => S.codex.e[k] > 0).length;
  const total = WEAPONS.length * 2 + CODEX_ENEMY_IDS.length + BOSS_IDS.length;
  const m = createModal({ cls: "panel codex", title: `硬件与异常图鉴 ${gotW + gotE + gotM} / ${total}`, dismiss: true });

  m.body.appendChild(el("div", "sec", "硬件武装与超维流片进化"));
  const wlist = el("div", "wlist");
  WEAPONS.forEach((w) => {
    const seenW = !!S.codex.w[w.id];
    const seenEvo = !!S.codex.evo[w.id];
    const partner = PASSIVES[PASSIVE_IDX[w.partner]];
    const row = el("div", "wrow" + (seenW ? "" : " un"));
    row.innerHTML = `
      <div class="w-a"><img alt="" src="${iconUrl(w.id, 96)}"></div>
      <div class="w-t"><b>${seenW ? w.name : "？？？"}</b><small>${seenW ? w.desc : "尚未装配过"}</small></div>
      <div class="w-p"><img alt="" src="${iconUrl(w.partner, 64)}"><small>+${partner.name}</small></div>
      <div class="w-a evo${seenEvo ? "" : " un"}"><img alt="" src="${iconUrl(w.id, 96)}"></div>
      <div class="w-t"><b>${seenEvo ? w.evoName : "？？？"}</b><small>${seenEvo ? w.evoDesc : "Lv.5 + 搭档模块流片"}</small></div>
    `;
    wlist.appendChild(row);
  });
  m.body.appendChild(wlist);

  m.body.appendChild(el("div", "sec", "机房故障体与五大产线巨兽 Boss"));
  const egrid = el("div", "egrid");
  CODEX_ENEMY_IDS.concat(BOSS_IDS).forEach((id) => {
    const cnt = S.codex.e[id] || 0;
    const def = ENEMIES[ENEMY_IDX[id]];
    const cell = el("div", "ecell" + (cnt ? "" : " un") + (def.boss ? " boss" : ""));
    cell.innerHTML = `<img alt="" src="${spriteUrl("e_" + id, def.boss ? 160 : 112, !cnt)}"><b>${cnt ? def.name : "？？？"}</b><small>${cnt ? `清除 ${cnt}` : def.boss ? "产线巨兽" : "尚未遭遇"}</small>`;
    egrid.appendChild(cell);
  });
  m.body.appendChild(egrid);
}

function openAchievements() {
  const S = G.S;
  const doneCnt = ACHIEVEMENTS.filter((a) => S.ach[a.id]).length;
  const m = createModal({ cls: "panel achs", title: `工程勋章 ${doneCnt} / ${ACHIEVEMENTS.length}`, dismiss: true });
  const list = el("div", "alist");
  ACHIEVEMENTS.slice()
    .sort((a, b) => (S.ach[a.id] ? 1 : 0) - (S.ach[b.id] ? 1 : 0))
    .forEach((a) => {
      const done = !!S.ach[a.id];
      const [cur, need] = achProgress(S, a);
      const ratio = Math.max(0.02, Math.min(1, cur / need));
      const row = el("div", "arow" + (done ? " done" : ""));
      row.innerHTML = `
        <div class="a-m"><img alt="" src="${iconUrl(done ? "ui_check" : "ui_medal", 64)}"></div>
        <div class="a-t"><b>${a.name}</b><small>${a.text}</small><div class="g-bar"><i style="transform:scaleX(${done ? 1 : ratio})"></i></div></div>
        <div class="a-r"><img alt="" src="${iconUrl("core", 48)}">+${a.cores}</div>
      `;
      list.appendChild(row);
    });
  m.body.appendChild(list);
}

function openHallOfFame() {
  const S = G.S;
  const dk = dateKey();
  const m = createModal({ cls: "panel rank", title: "荣耀殿堂", dismiss: true });

  const heroBox = el("div", "rk-hero");
  heroBox.innerHTML = `
    <div><span>最长稳定运行</span><b>${S.best.time ? fmtTime(S.best.time) : "00:00"}</b></div>
    <div style="text-align:right"><span>当前架构职级</span><b style="font-size:17px">${getTitle(S.best.time).name}</b></div>
  `;
  m.body.appendChild(heroBox);

  const grid = el("div", "rk-grid");
  [
    [String(S.best.kills), "单局最高清除"],
    ["×" + S.best.combo, "最高连击纪录"],
    [S.daily[dk] ? fmtTime(S.daily[dk].time) : "未出勤", "今日集群最佳"],
    [String(S.stats.kills), `生涯累计清除（${S.stats.runs} 局）`],
  ].forEach(([v, lbl]) => {
    grid.appendChild(el("div", "rk-cell", `<b>${v}</b><span>${lbl}</span>`));
  });
  m.body.appendChild(grid);

  m.body.appendChild(el("div", "sec", "本机荣耀殿堂 Top 10"));
  if (!S.hof.length) {
    m.body.appendChild(el("div", "note", "完成首次产线巡检后，你的最佳战局与硬件构筑将自动记录于此。"));
  } else {
    S.hof.forEach((r, idx) => {
      const row = el("div", "hof-row" + (idx === 0 ? " top1" : ""));
      row.innerHTML = `
        <div class="hof-rk">#${idx + 1}</div>
        <img class="hof-av" alt="" src="${heroUrl(r.char || "hone", 96)}">
        <div class="hof-tx"><b>主力「${r.top}」· Lv.${r.level}</b><small>${r.date} · ${r.stage || "存储阵列"}${r.daily ? " · 每日挑战" : ""}</small></div>
        <div class="hof-sc"><b>${fmtTime(r.time)}</b><small>清除 ${r.kills}</small></div>
      `;
      m.body.appendChild(row);
    });
  }
}

function openDailyModal() {
  const S = G.S;
  const d = getDailyChallenge(dateKey());
  const rule = RULES.find((r) => r.id === d.rule);
  const hero = HEROES[HERO_IDX[d.char]];
  const stg = STAGES[STAGE_IDX[d.stage]] || STAGES[0];
  const rec = S.daily[d.key];
  const m = createModal({ cls: "panel dailyp", title: "每日同种子集群挑战", dismiss: true });

  const top = el("div", "dp-top");
  const av = el("img", "dp-av");
  av.alt = "";
  av.src = heroUrl(d.char, 192);
  top.appendChild(av);
  top.appendChild(
    el("div", "dp-tx", `<small>${d.key} · ${stg.name}</small><b>今日工况「${rule.name}」</b><span>${rule.desc}</span>`),
  );
  m.body.appendChild(top);

  const info = el("div", "dp-info");
  info.innerHTML = `
    <div><img alt="" src="${heroUrl(d.char, 96)}"><span>指定特工</span><b>${hero.name}${S.chars[d.char] ? "" : "（限免）"}</b></div>
    <div><img alt="" src="${iconUrl(d.weapon, 96)}"><span>起手硬件</span><b>${WEAPONS[WEAPON_IDX[d.weapon]].name}</b></div>
    <div><img alt="" src="${iconUrl("core", 96)}"><span>运行 2 分钟</span><b>${rec && rec.rewarded ? "已领取" : "+120 晶核"}</b></div>
  `;
  m.body.appendChild(info);

  m.body.appendChild(
    el(
      "div",
      "dp-rec",
      (rec ? `今日最佳 <b>${fmtTime(rec.time)}</b> · 清除 <b>${rec.kills}</b>` : "今日尚未出勤") +
        (S.streak.n > 0 ? ` · 已连续值班 <b>${S.streak.n}</b> 天` : ""),
    ),
  );
  m.body.appendChild(
    makeBtn("primary", rec ? "再次挑战" : "立即进入集群", () => {
      m.close();
      startRun({ daily: true });
    }, { quiet: true }),
  );
}

function settingRowsEl() {
  const S = G.S;
  const wrap = el("div", "sets");
  [
    ["sfx", "硬件战斗音效"],
    ["music", "动态电子合成配乐"],
    ["shake", "关键事件轻微震屏"],
    ["vib", "触感振动反馈"],
  ].forEach(([k, label]) => {
    const row = el("div", "tg" + (S.set[k] ? " on" : ""), `<span>${label}</span><i><b></b></i>`);
    bindTap(row, () => {
      S.set[k] = S.set[k] ? 0 : 1;
      row.classList.toggle("on", !!S.set[k]);
      setAudioPrefs(S.set.sfx, S.set.music);
      if (G.fx) G.fx.shakeOn = !!S.set.shake;
      persistSave(S);
      if (k === "vib" && S.set.vib) vibrate("light");
    });
    wrap.appendChild(row);
  });
  return wrap;
}

function openSettings() {
  const m = createModal({ cls: "panel settings", title: "洁净室控制台设置", dismiss: true });
  m.body.appendChild(settingRowsEl());
  m.body.appendChild(
    el(
      "div",
      "note",
      `进度自动保存在浏览器本地存储中。<br>开源项目：<a href="${HONECLAW_GITHUB_URL}" target="_blank" rel="noopener noreferrer" style="color:#2563eb;font-weight:800">${HONECLAW_REPO_LABEL}</a><br>HONE 历险记 v${VERSION}`,
    ),
  );
}

// HUD & In-Run Modals
function buildHudUI() {
  hudEl = el("div", "hud hide");
  hudEl.innerHTML = `
    <div class="xp"><i></i><b class="lv">Lv.1</b></div>
    <div class="hrow">
      <div class="hrow-left">
        <div class="hb pause" title="暂停"></div>
      </div>
      <div class="hrow-center">
        <div class="timer">00:00</div>
        <div class="stage-tag">SECTOR-01 · 存储阵列</div>
      </div>
      <div class="kills"><img class="ki" alt=""><span class="kn">0</span></div>
    </div>
    <div class="slots"></div>
    <div class="bossbar hide"><span class="bn"></span><div class="bb"><i></i></div></div>
    <div class="combo hide"><b>0</b><small>连击</small></div>
    <div class="wavetag"></div>
    <div class="banner"></div>
    <div class="hint hide">按住屏幕拖动或按 WASD 走位 · 硬件武器自动索敌发射</div>
  `;
  document.body.appendChild(hudEl);
  $(".hrow-left", hudEl).appendChild(makeHomepageLink());

  hudRefs = {
    xp: $(".xp i", hudEl),
    xpBox: $(".xp", hudEl),
    lv: $(".lv", hudEl),
    timer: $(".timer", hudEl),
    stageTag: $(".stage-tag", hudEl),
    kn: $(".kn", hudEl),
    slots: $(".slots", hudEl),
    boss: $(".bossbar", hudEl),
    bossN: $(".bn", hudEl),
    bossI: $(".bb i", hudEl),
    combo: $(".combo", hudEl),
    comboB: $(".combo b", hudEl),
    wave: $(".wavetag", hudEl),
    banner: $(".banner", hudEl),
    hint: $(".hint", hudEl),
  };
  $(".ki", hudEl).src = spriteUrl("e_packet", 48);
  const pauseBtn = $(".pause", hudEl);
  pauseBtn.appendChild(iconImg("ui_pause", "", 48));
  bindTap(pauseBtn, pauseGame);
}

const hudCache = { sec: -1, kills: -1, lv: -1, slots: "", boss: -2, combo: -1, xp: -1, stage: -1 };
function resetHudCache() {
  hudCache.sec = hudCache.kills = hudCache.lv = hudCache.combo = hudCache.xp = hudCache.stage = -1;
  hudCache.boss = -2;
  hudCache.slots = "";
  hudRefs.boss.classList.add("hide");
  hudRefs.combo.classList.add("hide");
  hudRefs.banner.innerHTML = "";
  hudRefs.wave.textContent = "";
}

function updateHud() {
  const w = G.world;
  if (!w || G.mode !== "play") return;
  const p = w.p;
  const sec = Math.floor(w.t);
  if (sec !== hudCache.sec) {
    hudCache.sec = sec;
    hudRefs.timer.textContent = fmtTime(sec);
  }
  if (w.stageIdx !== hudCache.stage) {
    hudCache.stage = w.stageIdx;
    const stg = STAGES[w.stageIdx] || STAGES[0];
    hudRefs.stageTag.textContent = `${stg.code} · ${stg.short}`;
  }
  if (w.kills !== hudCache.kills) {
    hudCache.kills = w.kills;
    hudRefs.kn.textContent = w.kills;
  }
  if (p.lvl !== hudCache.lv) {
    hudCache.lv = p.lvl;
    hudRefs.lv.textContent = "Lv." + p.lvl;
  }
  const xpR = Math.min(1, p.xp / p.need);
  const xpStep = Math.round(xpR * 200);
  if (xpStep !== hudCache.xp) {
    hudCache.xp = xpStep;
    hudRefs.xp.style.transform = `scaleX(${xpR.toFixed(3)})`;
  }
  hudRefs.xpBox.classList.toggle("glow", G.fx.gemGlow > 0.5);

  let sig = "";
  for (let i = 0; i < p.weapons.length; i++) sig += p.weapons[i].id + p.weapons[i].lv + (p.weapons[i].evo ? "e" : "") + ",";
  sig += "|";
  for (let i = 0; i < p.passives.length; i++) sig += p.passives[i].id + p.passives[i].lv + ",";
  if (sig !== hudCache.slots) {
    hudCache.slots = sig;
    hudRefs.slots.innerHTML = "";
    p.weapons.forEach((wp) => {
      const s = el("div", "slot" + (wp.evo ? " evo" : ""));
      s.appendChild(iconImg(wp.id, "", 64));
      s.appendChild(el("em", "", wp.evo ? "★" : String(wp.lv)));
      hudRefs.slots.appendChild(s);
    });
    p.passives.forEach((ps) => {
      const s = el("div", "slot ps");
      s.appendChild(iconImg(ps.id, "", 64));
      s.appendChild(el("em", "", String(ps.lv)));
      hudRefs.slots.appendChild(s);
    });
  }

  const boss = w.boss && !w.boss.dead ? w.boss : null;
  const bossPct = boss ? Math.max(0, Math.round((boss.hp / boss.maxHp) * 200)) : -1;
  if (bossPct !== hudCache.boss) {
    if (boss) {
      hudRefs.boss.classList.remove("hide");
      hudRefs.bossN.textContent = ENEMIES[boss.type].name;
      hudRefs.bossI.style.transform = `scaleX(${(bossPct / 200).toFixed(3)})`;
    } else {
      hudRefs.boss.classList.add("hide");
    }
    hudCache.boss = bossPct;
  }

  if (w.combo !== hudCache.combo) {
    hudCache.combo = w.combo;
    if (w.combo >= 5) {
      hudRefs.combo.classList.remove("hide");
      hudRefs.comboB.textContent = w.combo;
    } else {
      hudRefs.combo.classList.add("hide");
    }
  }
  setBgmIntensity(boss ? 3 : w.t > 120 ? 2 : w.t > 35 ? 1 : 0);
}

function announce(text, kind, extra) {
  if (G.mode !== "play") return;
  if (kind === "wave") {
    hudRefs.wave.textContent = text;
    hudRefs.wave.classList.remove("on");
    void hudRefs.wave.offsetWidth;
    hudRefs.wave.classList.add("on");
    return;
  }
  if (kind === "combo") text = `${text}  ${COMBO_CALLOUTS[Math.min(COMBO_CALLOUTS.length - 1, extra || 0)]}`;
  if (kind === "warn") {
    const bDef = ENEMIES[9 + (extra || 0)] || ENEMIES[9];
    text = `⚠ 产线巨兽「${bDef.name}」即将切入`;
  }
  const item = el("div", "bn-i k-" + kind, text);
  while (hudRefs.banner.children.length > 1) hudRefs.banner.removeChild(hudRefs.banner.firstChild);
  hudRefs.banner.appendChild(item);
  setTimeout(() => item.parentNode && item.parentNode.removeChild(item), kind === "warn" ? 2600 : 1800);
}

function describeChoice(world, c) {
  if (c.kind === "w") {
    const w = WEAPONS[WEAPON_IDX[c.id]];
    const p = PASSIVES[PASSIVE_IDX[w.partner]];
    return {
      icon: c.id,
      name: w.name,
      tag: c.isNew ? "新硬件" : "Lv." + c.lv,
      cls: "w",
      desc: c.isNew ? w.desc : w.up[c.lv - 1],
      hint: world.plv(w.partner) > 0 ? `已装「${p.name}」，满级即可超维流片` : `流片搭档：「${p.name}」`,
    };
  }
  if (c.kind === "p") {
    const p = PASSIVES[PASSIVE_IDX[c.id]];
    let hint = "";
    for (let i = 0; i < world.p.weapons.length; i++) {
      const ow = world.p.weapons[i];
      if (WEAPONS[ow.k].partner === c.id && !ow.evo) hint = `★「${WEAPONS[ow.k].name}」的超维流片搭档`;
    }
    return { icon: c.id, name: p.name, tag: c.isNew ? "新模块" : "Lv." + c.lv, cls: "p", desc: p.desc, hint };
  }
  if (c.kind === "evo") {
    const w = WEAPONS[WEAPON_IDX[c.id]];
    return { icon: c.id, name: w.evoName, tag: "超维流片", cls: "evo", desc: w.evoDesc, hint: `${w.name} → ${w.evoName}` };
  }
  const b = BOOSTS.find((x) => x.id === c.id);
  return { icon: c.id, name: b.name, tag: "应急补给", cls: "b", desc: b.desc, hint: "" };
}

function renderChoiceCard(world, c, allowPin = false, onPin = null) {
  const info = describeChoice(world, c);
  const card = el("div", `card ${info.cls}${c.locked ? " locked" : ""}`);
  const ic = el("div", "c-ic");
  ic.appendChild(iconImg(info.icon, "", 128));
  card.appendChild(ic);
  const tx = el("div", "c-tx");
  tx.appendChild(
    el(
      "div",
      "c-n",
      `<b>${info.name}</b><em>${info.tag}</em>${c.locked ? '<em style="background:#fef3c7;color:#b45309">已锁定</em>' : ""}`,
    ),
  );
  tx.appendChild(el("div", "c-d", info.desc));
  if (info.hint) tx.appendChild(el("div", "c-h", info.hint));
  card.appendChild(tx);
  if (allowPin) {
    const pin = el("div", "c-pin");
    pin.title = "锁定此模块（重筛或下次升级保留）";
    pin.appendChild(iconImg("ui_pin", "", 48));
    bindTap(pin, (e) => {
      e.stopPropagation();
      onPin && onPin();
    });
    card.appendChild(pin);
  }
  return card;
}

function openLevelModal() {
  const world = G.world;
  if (!world || !world.pending || world.pending.type !== "level") return;
  levelModal && levelModal.close();
  const m = (levelModal = createModal({ cls: "levelup", noClose: true }));
  const openedAt = Date.now();

  const head = el("div", "lu-head");
  const av = el("img", "lu-hero");
  av.alt = "";
  av.src = heroUrl(G.hero, 220, false, "_joy");
  head.appendChild(av);
  head.appendChild(el("div", "lu-t", `<small>集群算力扩容</small><b>Lv.${world.pending.lvl}</b>`));
  m.body.appendChild(head);
  m.body.appendChild(el("div", "lu-s", "选择一项硬件模块装配（点右侧锁扣可保留心仪卡）"));

  const cardsWrap = el("div", "cards");
  m.body.appendChild(cardsWrap);

  const pickIdx = (idx) => {
    if (Date.now() - openedAt < 240 || !levelModal) return;
    const picked = world.pending.choices[idx];
    playSfx(picked && picked.kind === "evo" ? "evo" : "select", 0, picked && picked.kind === "evo" ? 1.1 : 1);
    if (G.S.set.vib) vibrate("light");
    world.choose(idx);
    m.close();
    levelModal = null;
    if (world.pending && world.pending.type === "level") {
      G.fx.levelFx(world.p.x, world.p.y);
      openLevelModal();
    } else {
      G.resumeT = 0.32;
      G.fx.ring(world.p.x, world.p.y, 14, 150, 0.34);
    }
  };

  const renderCards = () => {
    cardsWrap.innerHTML = "";
    world.pending.choices.forEach((c, idx) => {
      const card = renderChoiceCard(world, c, true, () => {
        world.toggleLockChoice(idx);
        renderCards();
      });
      bindTap(card, (e) => {
        if (e && e.target && e.target.closest && e.target.closest(".c-pin")) return;
        pickIdx(idx);
      }, { quiet: true });
      cardsWrap.appendChild(card);
    });
  };
  renderCards();

  const rerollBtn = makeBtn("ghost reroll", "", () => {
    if (Date.now() - openedAt < 240) return;
    if (world.reroll()) {
      playSfx("select", 0, 0.85);
      renderCards();
      syncReroll();
    } else {
      playSfx("err");
    }
  }, { quiet: true });

  const syncReroll = () => {
    rerollBtn.innerHTML = `晶圆良率重筛<small>剩 ${world.rerolls} 次</small>`;
    rerollBtn.classList.toggle("off", world.rerolls <= 0);
  };
  syncReroll();
  m.body.appendChild(rerollBtn);
  G.pickCard = pickIdx;
}

function openChestModal() {
  const world = G.world;
  if (!world || !world.pending || world.pending.type !== "chest" || chestModal) return;
  const m = (chestModal = createModal({ cls: "chest", noClose: true }));
  hudRefs.banner.innerHTML = "";
  const openedAt = Date.now();
  const items = world.pending.items;

  const top = el("div", "ch-top");
  top.appendChild(iconImg("chest", "ch-ic", 192));
  top.appendChild(el("div", "ch-t", items.length > 1 ? `量子超算金库！连获 ${items.length} 项模块` : "量子金库！免费升级 1 项模块"));
  m.body.appendChild(top);

  const cards = el("div", "cards");
  items.forEach((it, idx) => {
    const c = renderChoiceCard(world, it, false);
    cards.appendChild(c);
    setTimeout(() => {
      if (chestModal === m) playSfx(it.kind === "evo" ? "evo" : "select", 0, 1 + idx * 0.12);
    }, 150 + idx * 200);
  });
  m.body.appendChild(cards);

  m.body.appendChild(
    makeBtn("primary", "装载完毕，继续巡检", () => {
      if (Date.now() - openedAt < 260) return;
      m.close();
      chestModal = null;
      world.closeChest();
      if (world.pending && world.pending.type === "level") {
        G.fx.levelFx(world.p.x, world.p.y);
        openLevelModal();
      } else {
        G.resumeT = 0.32;
      }
    }),
  );
}

function pauseGame() {
  if (G.mode !== "play" || G.paused || G.deadT > 0 || !G.world || G.world.over) return;
  G.paused = true;
  stopBgm(0.15);
  const w = G.world;
  const stg = STAGES[w.stageIdx] || STAGES[0];
  const m = (pauseModal = createModal({ cls: "pause", title: "产线巡检已暂停", noClose: true }));
  m.body.appendChild(
    el("div", "p-info", `<span>${stg.short}</span><span>${fmtTime(w.t)}</span><span>清除 ${w.kills}</span><span>Lv.${w.p.lvl}</span>`),
  );

  const build = el("div", "p-build");
  w.p.weapons.forEach((wp) => {
    const s = el("div", "slot big" + (wp.evo ? " evo" : ""));
    s.appendChild(iconImg(wp.id, "", 96));
    s.appendChild(el("span", "", wp.evo ? WEAPONS[wp.k].evoName : `${WEAPONS[wp.k].name} Lv.${wp.lv}`));
    build.appendChild(s);
  });
  w.p.passives.forEach((ps) => {
    const s = el("div", "slot big ps");
    s.appendChild(iconImg(ps.id, "", 96));
    s.appendChild(el("span", "", `${PASSIVES[PASSIVE_IDX[ps.id]].name} Lv.${ps.lv}`));
    build.appendChild(s);
  });
  m.body.appendChild(build);

  m.body.appendChild(makeBtn("primary", "继续巡检", resumeGame));
  const row = el("div", "row2");
  row.appendChild(makeBtn("ghost", "重新部署", () => {
    closePause();
    startRun({ daily: w.daily });
  }));
  row.appendChild(makeBtn("ghost", "切换产线大厅", () => {
    closePause();
    goHome();
  }));
  m.body.appendChild(row);
  const rowHome = el("div", "row2");
  rowHome.appendChild(makeBtn("ghost", "← 回到 HONE 首页", () => {
    window.location.href = "/";
  }));
  m.body.appendChild(rowHome);
  m.body.appendChild(settingRowsEl());
}

function closePause() {
  if (pauseModal) {
    pauseModal.close();
    pauseModal = null;
  }
  G.paused = false;
}

function resumeGame() {
  if (!G.paused) return;
  closePause();
  G.last = performance.now();
  if (G.mode === "play") startBgm();
}

// Result Screen — Cleanroom Battle Report + honeclaw GitHub Star CTA!
function showResultScreen(sum, res) {
  const S = G.S;
  resultEl.innerHTML = "";
  const posterCanvas = renderBattlePoster(sum, S, G.floorImg);
  let posterJpg = null;
  const getPosterDataUrl = () => posterJpg || (posterJpg = posterCanvas.toDataURL("image/jpeg", 0.92));

  // Top bar with "回到首页" on top-left
  const topbar = el("div", "r-topbar");
  topbar.appendChild(makeHomepageLink());
  const coresBadge = el("div", "chip cores", `<img alt="" src="${iconUrl("core", 48)}"><b>${S.cores}</b>`);
  bindTap(coresBadge, openTalents);
  topbar.appendChild(coresBadge);
  resultEl.appendChild(topbar);

  const wrap = el("div", "r-wrap");
  const card = el("div", "r-card");
  wrap.appendChild(card);
  resultEl.appendChild(wrap);

  const visHero = activeHeroSkin(sum.char);
  const stg = STAGES[STAGE_IDX[sum.endStage || sum.stage || "storage"]] || STAGES[0];
  const head = el("div", "r-head");
  head.innerHTML = `
    <img class="r-hero" alt="" src="${heroUrl(visHero, 220, false, "_hurt")}">
    <div class="r-ht">
      <small>${sum.daily ? "每日集群挑战" : stg.name}${res.bestTime ? "<em>新纪录</em>" : ""}</small>
      <b>${fmtTime(sum.time)}</b>
      <span>${getTitle(S.best.time).name}${sum.cause ? ` · 遭遇「${sum.cause}」阻断` : ""}</span>
    </div>
  `;
  card.appendChild(head);

  const mid = el("div", "r-mid");
  const posterImg = el("img", "r-poster");
  posterImg.alt = "";
  mid.appendChild(posterImg);
  setTimeout(() => { posterImg.src = getPosterDataUrl(); }, 0);

  const stats = el("div", "r-stats");
  stats.innerHTML = `
    <div><b>${sum.kills}</b><small>清除异常</small></div>
    <div><b>Lv.${sum.level}</b><small>集群等级</small></div>
    <div><b>×${sum.maxCombo}</b><small>最高连击</small></div>
  `;
  const buildRow = el("div", "r-build");
  sum.weapons.forEach((wp) => {
    const s = el("div", "slot" + (wp.evo ? " evo" : ""));
    s.appendChild(iconImg(wp.id, "", 64));
    s.appendChild(el("em", "", wp.evo ? "★" : String(wp.lv)));
    buildRow.appendChild(s);
  });
  stats.appendChild(buildRow);
  mid.appendChild(stats);
  card.appendChild(mid);

  const rew = el("div", "r-rew");
  rew.innerHTML = `<div class="rw-main"><img alt="" src="${iconUrl("core", 64)}"><b>+${res.total}</b><span>算力晶核</span></div>`;
  const chips = el("div", "rw-chips");
  res.bonus.slice(0, 3).forEach((b) => chips.appendChild(el("span", "chip2", `${b.label} +${b.n}`)));
  res.unlocked.forEach((h) => chips.appendChild(el("span", "chip2", `新特工「${h.name}」已解锁`)));
  if (res.titleUp) chips.appendChild(el("span", "chip2", `晋升「${res.titleUp}」`));
  if (chips.children.length) rew.appendChild(chips);
  card.appendChild(rew);

  // Prominent honeclaw GitHub Star CTA Card!
  const ghCard = el("a", "gh-star-card");
  ghCard.href = HONECLAW_GITHUB_URL;
  ghCard.target = "_blank";
  ghCard.rel = "noopener noreferrer";
  ghCard.innerHTML = `
    <div class="gh-l">
      <div class="gh-title">★ 喜欢 HONE 历险记？去 GitHub 给 honeclaw 点个 Star！</div>
      <span class="gh-url">${HONECLAW_GITHUB_URL}</span>
    </div>
    <div class="gh-btn"><img alt="" src="${iconUrl("ui_star", 48)}"><span>Star · <b class="gh-stars-cnt">${ghStarsLabel}</b></span></div>
  `;
  ghCard.addEventListener("click", () => {
    if (!S.starred) {
      S.starred = 1;
      S.cores += 88;
      S.skins.gold = 1;
      persistSave(S);
      const badgeB = $("b", coresBadge);
      if (badgeB) badgeB.textContent = S.cores;
      showToast("感谢支持开源 honeclaw！已赠送 +88 算力晶核与曜金涂装！", 2800);
    }
  });
  card.appendChild(ghCard);
  refreshGithubStars();

  card.appendChild(
    makeBtn("primary again", "<b>再次进入产线巡检</b>", () => startRun({ daily: sum.daily }), { quiet: true }),
  );

  const soc = el("div", "r-soc");
  const topW = sum.weapons.slice().sort((a, b) => b.dmg - a.dmg)[0];
  const topWName = topW && WEAPONS[WEAPON_IDX[topW.id]]
    ? (topW.evo ? WEAPONS[WEAPON_IDX[topW.id]].evoName : WEAPONS[WEAPON_IDX[topW.id]].name)
    : "HBM 带宽飞梭";
  const shareText = `我在《HONE 历险记》${stg.name}坚守 ${fmtTime(sum.time)}，清除 ${sum.kills} 个硬件异常，主力构筑「${topWName}」，晋升「${getTitle(S.best.time).name}」！开源仓库：${HONECLAW_GITHUB_URL}`;

  const dlBtn = el("div", "sbtn");
  dlBtn.append(iconImg("ui_save", "", 48), el("span", "", "下载战报图"));
  bindTap(dlBtn, () => {
    const ok = downloadPoster(posterCanvas, `hone-adventure-${fmtTime(sum.time).replace(":", "m")}s.png`);
    showToast(ok ? "高清战报海报已开始下载" : "保存失败，请长按左侧海报保存");
  });
  soc.appendChild(dlBtn);

  const cpBtn = el("div", "sbtn");
  cpBtn.append(iconImg("ui_copy", "", 48), el("span", "", "复制战报与仓库链接"));
  bindTap(cpBtn, async () => {
    const mode = await copyPosterToClipboard(posterCanvas, shareText);
    showToast(mode === "image" ? "战报图片已复制到剪贴板！" : mode === "text" ? "战绩与 honeclaw 仓库链接已复制！" : "请点击下载战报图");
  });
  soc.appendChild(cpBtn);
  card.appendChild(soc);

  const links = el("div", "r-links");
  [
    ["升级基建", openTalents],
    ["晋升路线", openPromotionModal],
    ["切换关卡", goHome],
    ["荣耀殿堂", openHallOfFame],
  ].forEach(([lbl, fn]) => {
    const lk = el("div", "lk", lbl);
    bindTap(lk, fn);
    links.appendChild(lk);
  });
  card.appendChild(links);

  resultEl.classList.remove("hide");
}

// Run Lifecycle & Live Cleanroom AI Demo
function viewWorldDims() {
  return [G.cw / G.sc, G.ch / G.sc];
}

function startHomeDemo() {
  const [vw, vh] = viewWorldDims();
  G.hero = activeHeroSkin(G.S.sel);
  G.fx.clear();
  G.fx.quiet = true;
  G.world = new GameWorld({
    seed: (Math.random() * 1e9) | 0,
    char: G.S.sel,
    stage: G.S.stage || "storage",
    talents: { might: 3, hp: 4, speed: 2, pickup: 3 },
    bestTime: G.S.best.time || 0,
    vw,
    vh,
    fx: G.fx.on,
  });
  G.world.t = 32;
  G.world.wave = 1;
  G.world.nextElite = 75;
  G.bot = createAutoBot(1);
  for (let i = 0; i < 180 && !G.world.over; i++) {
    if (G.world.pending) {
      if (G.world.pending.type === "level") G.world.choose(G.bot.pick(G.world, G.world.pending.choices));
      else G.world.closeChest();
      continue;
    }
    G.bot.move(G.world, 1 / 30);
    G.world.update(1 / 30);
  }
  // keep selected stage active on home demo
  G.world.setStage(G.S.stage || "storage");
  G.fx.clear();
  G.camX = G.world.p.x;
  G.camY = G.world.p.y + 24;
}

function startRun(opts = {}) {
  const S = G.S;
  const [vw, vh] = viewWorldDims();
  closeAllModals();
  levelModal = chestModal = pauseModal = null;
  homeEl.classList.add("hide");
  resultEl.classList.add("hide");
  resultEl.innerHTML = "";

  const dailyCfg = opts.daily ? getDailyChallenge(dateKey()) : null;
  const charId = dailyCfg ? dailyCfg.char : S.sel;
  const stageId = dailyCfg ? dailyCfg.stage : opts.stage || S.stage || "storage";
  G.hero = activeHeroSkin(charId);
  G.fx.clear();
  G.fx.quiet = false;

  G.world = new GameWorld({
    seed: dailyCfg ? dailyCfg.seed : (Math.random() * 1e9) | 0,
    char: charId,
    stage: stageId,
    talents: S.talents,
    bestTime: S.best.time || 0,
    rule: dailyCfg ? dailyCfg.rule : null,
    weapon: dailyCfg ? dailyCfg.weapon : null,
    daily: !!dailyCfg,
    manualLevel: true,
    intro: !dailyCfg && S.stats.runs < 3,
    vw,
    vh,
    fx: G.fx.on,
  });

  G.run = { daily: !!dailyCfg, moved: false };
  G.mode = "play";
  G.paused = false;
  G.deadT = 0;
  G.hitstop = 0;
  G.slowT = 0;
  G.lvRamp = 0;
  G.lvArmed = false;
  G.resumeT = 0;
  G.camX = 0;
  G.camY = 0;
  G.zoom = 0;
  G.joy.on = false;
  G.joy.mx = 0;
  G.joy.my = 0;

  resetHudCache();
  hudEl.classList.remove("hide");
  hudRefs.hint.classList.toggle("hide", S.stats.runs >= 2);
  const stg = STAGES[STAGE_IDX[stageId]] || STAGES[0];
  announce(dailyCfg ? `今日工况「${G.world.rule.name}」` : `进入${stg.name}`, "big");
  startBgm();
  G.last = performance.now();
}

function finishRun() {
  const sum = G.world.summary();
  const res = settleRun(G.S, sum);
  persistSave(G.S);
  G.mode = "result";
  hudEl.classList.add("hide");
  stopBgm(0.4);
  G.fx.quiet = true;
  showResultScreen(sum, res);
}

function goHome() {
  closeAllModals();
  levelModal = chestModal = pauseModal = null;
  resultEl.classList.add("hide");
  resultEl.innerHTML = "";
  hudEl.classList.add("hide");
  stopBgm(0.2);
  G.paused = false;
  G.mode = "home";
  startHomeDemo();
  refreshHomeUI();
  homeEl.classList.remove("hide");
}

function stepWorldFixed(world, dt) {
  let rem = dt;
  let steps = 0;
  const MAX_STEP = 1 / 45;
  while (rem > 1e-5 && steps < 4) {
    const s = rem > MAX_STEP ? MAX_STEP : rem;
    world.update(s);
    rem -= s;
    steps++;
  }
}

function resizeStage() {
  G.cw = window.innerWidth || document.documentElement.clientWidth || 390;
  G.ch = window.innerHeight || document.documentElement.clientHeight || 844;
  G.sc = Math.min(G.cw / 390, G.ch / 700);
  let dpr = G.dpr;
  while (dpr > 1 && G.cw * G.ch * dpr * dpr > 2e6) dpr -= 0.25;
  G.R.resize(G.cw, G.ch, dpr);
}

function bindJoystick(canvas) {
  const joy = G.joy;
  let activeId = null;

  const onStart = (x, y) => {
    if (G.mode !== "play" || G.paused) return;
    joy.on = true;
    joy.bx = joy.kx = x;
    joy.by = joy.ky = y;
    joy.mx = joy.my = 0;
  };
  const onMove = (x, y) => {
    if (!joy.on) return;
    let dx = x - joy.bx;
    let dy = y - joy.by;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const tether = 62;
    const maxR = 46;
    if (dist > tether) {
      joy.bx += (dx / dist) * (dist - tether);
      joy.by += (dy / dist) * (dist - tether);
      dx = x - joy.bx;
      dy = y - joy.by;
    }
    const d = Math.sqrt(dx * dx + dy * dy);
    if (d < 6) {
      joy.mx = joy.my = 0;
      joy.kx = joy.bx;
      joy.ky = joy.by;
      return;
    }
    const mag = Math.min(1, d / maxR);
    const clamp = Math.min(d, maxR);
    joy.mx = (dx / d) * mag;
    joy.my = (dy / d) * mag;
    joy.kx = joy.bx + (dx / d) * clamp;
    joy.ky = joy.by + (dy / d) * clamp;
  };
  const onEnd = () => {
    activeId = null;
    joy.on = false;
    joy.mx = joy.my = 0;
  };

  canvas.addEventListener("pointerdown", (e) => {
    if (activeId !== null || e.button > 0) return;
    activeId = e.pointerId;
    onStart(e.clientX, e.clientY);
    try { canvas.setPointerCapture(e.pointerId); } catch {}
  });
  canvas.addEventListener("pointermove", (e) => {
    if (e.pointerId === activeId) onMove(e.clientX, e.clientY);
  });
  window.addEventListener("pointerup", (e) => {
    if (e.pointerId === activeId) onEnd();
  }, true);
  window.addEventListener("pointercancel", (e) => {
    if (e.pointerId === activeId) onEnd();
  }, true);
}

const viewFrame = {};
function frameLoop(now) {
  requestAnimationFrame(frameLoop);
  let dt = (now - G.last) / 1000;
  G.last = now;
  if (!(dt > 0)) dt = 1e-4;
  if (dt > 0.1) dt = 0.1;
  if (document.hidden || !G.world) return;

  const t0 = performance.now();
  G.time += dt;
  const w = G.world;
  const fx = G.fx;
  let simDt = dt;

  if (G.mode === "play") {
    if (G.paused) {
      simDt = 0;
    } else if (w.over) {
      if (G.deadT <= 0) {
        G.deadT = 1.15;
        if (levelModal) { levelModal.close(); levelModal = null; }
      }
      G.deadT -= dt;
      simDt = dt * 0.45;
      if (G.deadT <= 0) {
        G.deadT = 0;
        finishRun();
      }
    } else if (w.pending) {
      simDt = 0;
      if (w.pending.type === "chest" && !chestModal) openChestModal();
      if (w.pending.type === "level" && !levelModal) openLevelModal();
    } else {
      let scale = 1;
      if (G.hitstop > 0) {
        G.hitstop -= dt;
        scale = 0;
      }
      if (G.slowT > 0) {
        G.slowT -= dt;
        const ramp = G.slowT < 0.25 ? G.slowT / 0.25 : 1;
        scale = Math.min(scale, 1 - (1 - G.slowS) * ramp);
      }
      if (G.resumeT > 0) {
        G.resumeT -= dt;
        scale = Math.min(scale, 1 - (G.resumeT / 0.32) * 0.7);
      }
      if (w.pendLv > 0) {
        if (!G.lvArmed) {
          G.lvArmed = true;
          G.joyT = 0.9;
          G.lvRamp = 0.28;
          fx.levelFx(w.p.x, w.p.y);
          playSfx("level");
          if (G.S.set.vib) vibrate("light");
          w.p.inv = Math.max(w.p.inv, 0.9);
          G.zoom = 0.06;
        }
        G.lvRamp -= dt;
        scale = Math.min(scale, 0.16);
        if (G.lvRamp <= 0) {
          G.lvArmed = false;
          w.openLevel();
          openLevelModal();
          scale = 0;
        }
      }
      simDt = dt * scale;
      let mx = G.joy.mx;
      let my = G.joy.my;
      const K = G.keys;
      if (K.l || K.r || K.u || K.d) {
        mx = (K.r ? 1 : 0) - (K.l ? 1 : 0);
        my = (K.d ? 1 : 0) - (K.u ? 1 : 0);
      }
      if ((mx || my) && !G.run.moved) {
        G.run.moved = true;
        hudRefs.hint.classList.add("hide");
      }
      w.input(mx, my);
      const [vw, vh] = viewWorldDims();
      w.vw = vw;
      w.vh = vh;
      if (simDt > 0) stepWorldFixed(w, simDt);
    }
    updateHud();
    tickBgm();
  } else if (G.mode === "home") {
    if (w.pending) {
      if (w.pending.type === "level") w.choose(G.bot.pick(w, w.pending.choices));
      else w.closeChest();
    }
    if (w.over || w.t > 180) {
      startHomeDemo();
    } else {
      const curStage = G.S.stage || "storage";
      G.bot.move(w, dt);
      const [vw, vh] = viewWorldDims();
      w.vw = vw;
      w.vh = vh;
      stepWorldFixed(w, dt);
      if (STAGES[w.stageIdx].id !== curStage) w.setStage(curStage);
    }
  } else {
    simDt = dt * 0.3;
  }

  const curW = G.world;
  fx.update(simDt, dt);
  const zKey = pickZoneKey(curW);
  if (zKey !== G.zoneId) {
    G.zoneId = zKey;
  }
  lerpZone(G.zone, zKey, 1 - Math.exp(-dt * 2.2));
  if (G.hurtT > 0) G.hurtT -= dt;

  const isJoy = G.mode === "play" && (G.lvArmed || (curW.pending && !curW.over) || G.joyT > 0);
  if (G.joyT > 0 && !curW.pending) G.joyT -= dt;
  G.joyK += ((isJoy ? 1 : 0) - G.joyK) * Math.min(1, dt * 10);

  const follow = 1 - Math.exp(-dt * 9);
  const yOff = G.mode === "home" ? 28 : -10;
  G.camX += (curW.p.x - G.camX) * follow;
  G.camY += (curW.p.y + yOff - G.camY) * follow;
  if (G.zoom > 0) G.zoom = Math.max(0, G.zoom - dt * 0.22);

  viewFrame.t = G.time;
  viewFrame.cw = G.cw;
  viewFrame.ch = G.ch;
  viewFrame.sc = G.sc * (1 + G.zoom) * (G.mode === "home" ? 1.15 : 1);
  viewFrame.camX = G.camX + (G.mode === "play" ? fx.shX : 0);
  viewFrame.camY = G.camY + (G.mode === "play" ? fx.shY : 0);
  viewFrame.hero = G.hero;
  viewFrame.state = G.hurtT > 0 ? "_hurt" : G.joyK > 0.3 ? "_joy" : "";
  viewFrame.joy = G.joyK;
  viewFrame.hurt = G.hurtT > 0 ? Math.min(1, G.hurtT / 0.3) : 0;
  viewFrame.joyst = G.mode === "play" && !G.paused ? G.joy : null;
  viewFrame.dim = G.mode === "home" ? 0.04 : G.mode === "result" ? 0.35 : 0;
  viewFrame.lite = G.R.kind !== "webgl" || fx.q < 0.5;
  viewFrame.zone = G.zone;
  viewFrame.uz = 1;
  renderWorld(G.R, curW, fx, viewFrame);

  // Adaptive Quality Governor
  const elapsedMs = performance.now() - t0;
  const pf = G.perf;
  pf.acc += dt;
  pf.accMs += elapsedMs;
  pf.n++;
  if (pf.acc >= 1) {
    pf.fps = pf.n / pf.acc;
    pf.ms = pf.accMs / pf.n;
    pf.acc = pf.accMs = pf.n = 0;
    if (!G.fixedQ) {
      if (pf.fps < 44 || pf.ms > 15) {
        pf.low++;
        pf.high = 0;
      } else if (pf.fps > 56 && pf.ms < 7) {
        pf.high++;
        pf.low = 0;
      } else {
        pf.low = pf.high = 0;
      }
      if (pf.low >= 3) {
        pf.low = 0;
        if (fx.q > 0.6) fx.q = 0.6;
        else if (G.dpr > 1) { G.dpr = Math.max(1, G.dpr - 0.25); resizeStage(); }
        else fx.q = 0.35;
      }
      if (pf.high >= 6) {
        pf.high = 0;
        if (fx.q < 1) fx.q = 1;
        else if (G.dpr < G.dprMax) { G.dpr = Math.min(G.dprMax, G.dpr + 0.25); resizeStage(); }
      }
    }
  }
}

async function boot() {
  stageEl = el("canvas", "stage");
  stageEl.id = "stage";
  document.body.appendChild(stageEl);

  const params = new URLSearchParams(location.search);
  const noArt = params.get("noart") === "1";
  const [sprImg, flrImg] = await Promise.all([
    noArt ? null : loadImage("./sprites.webp", 7000),
    noArt ? null : loadImage("./floor.webp", 7000),
  ]);

  if (sprImg) bindSpriteSheet(sprImg);
  G.floorImg = flrImg || makeFallbackFloor(512);
  G.atlas = buildAtlas(2, 2048);
  G.R = createRenderer(stageEl, G.atlas, [G.floorImg, makeQuantOverlay(256)], params.get("r") === "2d");

  const devDpr = window.devicePixelRatio || 1;
  G.dprMax = G.R.kind === "webgl" ? Math.min(devDpr, 2) : Math.min(devDpr, 1.25);
  G.dpr = G.R.kind === "webgl" ? Math.min(devDpr, 1.5) : 1;

  G.fx = createFxSystem({
    hitstop: (d) => { if (G.mode === "play" && d > G.hitstop) G.hitstop = d; },
    slowmo: (s, d) => { if (G.mode === "play") { G.slowS = s; G.slowT = d; } },
    vibrate: (k) => { if (G.mode === "play" && G.S && G.S.set.vib) vibrate(k); },
    combo: () => (G.world ? G.world.combo : 0),
    camX: () => G.camX,
    camY: () => G.camY,
    viewW: () => G.cw / G.sc,
    viewH: () => G.ch / G.sc,
    announce,
    hurt: () => { G.hurtT = 0.42; },
  });
  if (G.R.kind !== "webgl") G.fx.q = 0.6;

  resizeStage();
  window.addEventListener("resize", resizeStage);
  bindJoystick(stageEl);

  const keyMap = {
    ArrowLeft: "l", a: "l", A: "l",
    ArrowRight: "r", d: "r", D: "r",
    ArrowUp: "u", w: "u", W: "u",
    ArrowDown: "d", s: "d", S: "d",
  };
  window.addEventListener("keydown", (e) => {
    const k = keyMap[e.key];
    if (k) G.keys[k] = 1;
    if (e.key === "Escape" || e.key === "p" || e.key === "P") {
      if (G.mode === "play") G.paused ? resumeGame() : pauseGame();
    }
    if (levelModal && (e.key === "1" || e.key === "2" || e.key === "3")) {
      G.pickCard && G.pickCard(Number(e.key) - 1);
    }
  });
  window.addEventListener("keyup", (e) => {
    const k = keyMap[e.key];
    if (k) G.keys[k] = 0;
  });
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      if (G.mode === "play" && !G.paused && G.world && !G.world.pending && !G.world.over) pauseGame();
      else stopBgm(0.1);
    } else {
      G.last = performance.now();
    }
  });
  document.addEventListener("contextmenu", (e) => e.preventDefault());

  G.S = loadSave();
  G.zone = cloneZone(G.S.stage || "storage");
  G.zoneId = G.S.stage || "storage";
  setAudioPrefs(G.S.set.sfx, G.S.set.music);
  G.fx.shakeOn = !!G.S.set.shake;
  buildHudUI();
  buildHomeUI();
  G.mode = "home";
  startHomeDemo();
  refreshHomeUI();
  homeEl.classList.remove("hide");
  refreshGithubStars();

  const bootMask = $("#boot");
  if (bootMask) {
    bootMask.classList.add("out");
    setTimeout(() => bootMask.parentNode && bootMask.parentNode.removeChild(bootMask), 340);
  }

  G.last = performance.now();
  requestAnimationFrame(frameLoop);

  const warmLoop = () => {
    if (!warmAudioStep(false)) setTimeout(warmLoop, 16);
  };
  setTimeout(warmLoop, 60);
}

window.__honeAdventure = {
  G,
  version: VERSION,
  githubUrl: HONECLAW_GITHUB_URL,
  art: ART_STATE,
  startRun,
  goHome,
  selectStage,
  openTalents,
  openPromotionModal,
  openHeroes,
  openCodex,
  openHallOfFame,
  state() {
    const w = G.world;
    return {
      mode: G.mode,
      paused: G.paused,
      stage: w ? STAGES[w.stageIdx].id : G.S ? G.S.stage : "storage",
      t: w ? w.t : 0,
      kills: w ? w.kills : 0,
      lvl: w ? w.p.lvl : 0,
      hp: w ? w.p.hp : 0,
      maxHp: w ? w.p.maxHp : 0,
      over: w ? w.over : false,
      pending: w && w.pending ? w.pending.type : "",
      nE: w ? w.nE : 0,
      cores: G.S ? G.S.cores : 0,
      renderer: G.R ? G.R.kind : "",
      sheetLoaded: ART_STATE.sheet,
      weapons: w ? w.p.weapons.map((x) => x.id + x.lv + (x.evo ? "*" : "")) : [],
    };
  },
};

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", boot);
} else {
  boot();
}
