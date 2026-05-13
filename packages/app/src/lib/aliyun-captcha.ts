export type PublicCaptchaConfig = {
  enabled: boolean;
  region: string;
  prefix: string;
  scene_id: string;
  script_url: string;
};

type AliyunCaptchaInstance = {
  show?: () => void;
  hide?: () => void;
};

type AliyunCaptchaOptions = {
  SceneId: string;
  mode: "popup";
  element: string;
  button: string;
  region: string;
  prefix: string;
  language: string;
  timeout: number;
  slideStyle: { width: number; height: number };
  success: (captchaVerifyParam: string) => void;
  fail: (error: unknown) => void;
  error: (error: unknown) => void;
  getInstance: (instance: AliyunCaptchaInstance) => void;
};

declare global {
  interface Window {
    AliyunCaptchaConfig?: {
      region: string;
      prefix: string;
    };
    initAliyunCaptcha?: (options: AliyunCaptchaOptions) => void;
  }
}

let scriptPromise: Promise<void> | undefined;

export class AliyunCaptchaController {
  private instance: AliyunCaptchaInstance | undefined;
  private initPromise: Promise<void> | undefined;
  private pending:
    | {
        resolve: (value: string) => void;
        reject: (error: Error) => void;
      }
    | undefined;

  constructor(
    private readonly config: PublicCaptchaConfig,
    private readonly elementId: string,
    private readonly buttonId: string,
  ) {}

  async prepare() {
    await this.ensureInit();
  }

  async verify() {
    await this.ensureInit();
    if (this.pending) {
      this.pending.reject(new Error("上一次图形验证还未完成"));
      this.pending = undefined;
    }
    return new Promise<string>((resolve, reject) => {
      this.pending = { resolve, reject };
      this.instance?.show?.();
    });
  }

  private async ensureInit() {
    if (!this.initPromise) {
      this.initPromise = this.init();
    }
    return this.initPromise;
  }

  private async init() {
    await loadAliyunCaptchaScript(this.config);
    if (!window.initAliyunCaptcha) {
      throw new Error("图形验证码组件加载失败");
    }

    await new Promise<void>((resolve, reject) => {
      window.initAliyunCaptcha!({
        SceneId: this.config.scene_id,
        mode: "popup",
        element: `#${this.elementId}`,
        button: `#${this.buttonId}`,
        region: this.config.region,
        prefix: this.config.prefix,
        language: "cn",
        timeout: 8000,
        slideStyle: {
          width: Math.max(300, Math.min(360, window.innerWidth - 32)),
          height: 40,
        },
        success: (captchaVerifyParam) => {
          this.instance?.hide?.();
          if (!this.pending) return;
          this.pending.resolve(captchaVerifyParam);
          this.pending = undefined;
        },
        fail: (error) => this.rejectPending(error),
        error: (error) => {
          this.rejectPending(error);
          reject(asError(error));
        },
        getInstance: (instance) => {
          this.instance = instance;
          resolve();
        },
      });
    });
  }

  private rejectPending(error: unknown) {
    if (!this.pending) return;
    this.pending.reject(asError(error));
    this.pending = undefined;
  }
}

function loadAliyunCaptchaScript(config: PublicCaptchaConfig) {
  window.AliyunCaptchaConfig = {
    region: config.region,
    prefix: config.prefix,
  };
  if (window.initAliyunCaptcha) {
    return Promise.resolve();
  }
  if (!scriptPromise) {
    scriptPromise = new Promise<void>((resolve, reject) => {
      const existing = document.querySelector<HTMLScriptElement>(
        `script[data-aliyun-captcha="true"]`,
      );
      if (existing) {
        existing.addEventListener("load", () => resolve(), { once: true });
        existing.addEventListener("error", () => reject(new Error("图形验证码组件加载失败")), {
          once: true,
        });
        return;
      }
      const script = document.createElement("script");
      script.src = config.script_url;
      script.async = true;
      script.defer = true;
      script.dataset.aliyunCaptcha = "true";
      script.addEventListener("load", () => resolve(), { once: true });
      script.addEventListener("error", () => reject(new Error("图形验证码组件加载失败")), {
        once: true,
      });
      document.head.appendChild(script);
    });
  }
  return scriptPromise;
}

function asError(error: unknown) {
  if (error instanceof Error) return error;
  if (typeof error === "string" && error.trim()) return new Error(error);
  return new Error("图形验证未通过");
}
