import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(path, import.meta.url), "utf8");

const backend = read("../../../../crates/hone-web-api/src/routes/public.rs");
const routes = read("../../../../crates/hone-web-api/src/routes/mod.rs");
const api = read("../lib/api.ts");
const login = read("../components/public-login-form.tsx");

describe("local public dev login contract", () => {
  it("fails closed outside explicit local deployment mode", () => {
    expect(backend).toContain('std::env::var("HONE_PUBLIC_DEV_LOGIN")');
    expect(backend).toContain('deployment_mode.eq_ignore_ascii_case("local")');
    expect(backend).not.toContain('cloud_mode.eq_ignore_ascii_case("local")');
    expect(backend).toContain("StatusCode::NOT_FOUND.into_response()");
  });

  it("uses a normal server session and HttpOnly cookie instead of a frontend bypass", () => {
    expect(routes).toContain('"/auth/dev-login/config"');
    expect(routes).toContain('"/auth/dev-login"');
    expect(backend).toContain("create_session_for_user");
    expect(backend).toContain("build_session_cookie");
    expect(backend).toContain("record_tos_acceptance");
    expect(api).toContain('apiFetch("/api/public/auth/dev-login"');
    expect(login).not.toContain("document.cookie");
  });

  it("shows the local entry only after the backend enables it", () => {
    expect(login).toContain("getPublicDevLoginConfig()");
    expect(login).toContain("<Show when={devLoginEnabled()}>");
    expect(login).toContain("CONTENT.auth.login.dev_login");
  });
});
