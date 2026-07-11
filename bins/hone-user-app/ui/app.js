const APP_URL = "https://hone-claw.com/chat";
const HEALTH_IMAGE_URL = "https://hone-claw.com/logo.svg";
const MINIMUM_SPLASH_MS = 620;

const shell = document.querySelector(".launch-shell");
const status = document.querySelector("#status");
const retry = document.querySelector("#retry");

function setOffline(message) {
  shell.dataset.state = "offline";
  status.textContent = message;
  retry.hidden = false;
  retry.disabled = false;
}

async function connect() {
  shell.dataset.state = "connecting";
  status.textContent = "正在安全连接 hone-claw.com…";
  retry.hidden = true;
  retry.disabled = true;

  if (!navigator.onLine) {
    setOffline("当前处于离线状态，联网后即可继续");
    return;
  }

  const startedAt = Date.now();
  const probe = new Image();
  const probeResult = new Promise((resolve, reject) => {
    probe.onload = resolve;
    probe.onerror = reject;
    probe.src = `${HEALTH_IMAGE_URL}?app_probe=${Date.now()}`;
  });
  const timeoutResult = new Promise((_, reject) => {
    window.setTimeout(() => reject(new Error("connection timeout")), 8000);
  });

  try {
    await Promise.race([probeResult, timeoutResult]);
    const remaining = Math.max(0, MINIMUM_SPLASH_MS - (Date.now() - startedAt));
    status.textContent = "连接成功，正在进入对话…";
    window.setTimeout(() => window.location.replace(APP_URL), remaining);
  } catch {
    setOffline("暂时无法连接 HONE，请检查网络后重试");
  }
}

retry.addEventListener("click", connect);
window.addEventListener("online", () => {
  if (shell.dataset.state === "offline") connect();
});

connect();
