// public-me.tsx — Hone Public Site Account / Me page

import { createSignal, onMount, Show, type ParentProps } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { PublicNav, PublicFooter } from "@/components/public-nav";
import { PublicLoginForm } from "@/components/public-login-form";
import { CONTENT } from "@/lib/public-content";
import { getPublicAuthMe, publicLogout } from "@/lib/api";
import type { PublicAuthUserInfo } from "@/lib/types";
import "./public-site.css";

function formatDate(iso: string | undefined): string {
  if (!iso) return CONTENT.me.date_placeholder;
  try {
    return new Date(iso).toLocaleDateString(CONTENT.me.date_locale, {
      year: "numeric",
      month: "long",
      day: "numeric",
    });
  } catch {
    return iso;
  }
}

function InfoRow(props: { label: string; value: string }) {
  return (
    <div
      style={{
        display: "flex",
        "align-items": "center",
        "justify-content": "space-between",
        padding: "14px 0",
        "border-bottom": "1px solid rgba(0,0,0,0.06)",
      }}
    >
      <span
        style={{ "font-size": "13px", color: "#94a3b8", "font-weight": "500" }}
      >
        {props.label}
      </span>
      <span
        style={{
          "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
          "font-size": "13px",
          color: "#0f172a",
          "font-weight": "500",
        }}
      >
        {props.value}
      </span>
    </div>
  );
}

type ActionBtnVariant = "default" | "primary" | "ghost" | "danger";

function ActionBtn(
  props: ParentProps<{
    onClick?: () => void;
    href?: string;
    variant?: ActionBtnVariant;
  }>,
) {
  const variant = () => props.variant ?? "default";

  const getStyle = () => {
    const base = {
      padding: "10px 20px",
      "border-radius": "8px",
      cursor: "pointer",
      "font-family": "inherit",
      "font-size": "14px",
      "font-weight": "600",
      "letter-spacing": "0.01em",
      transition: "all 0.2s",
      display: "inline-flex",
      "align-items": "center",
      gap: "6px",
      "text-decoration": "none",
    };
    if (variant() === "primary") {
      return {
        ...base,
        background: "#f59e0b",
        border: "1px solid #f59e0b",
        color: "#fff",
        "box-shadow": "0 2px 8px rgba(245,158,11,0.25)",
      };
    }
    if (variant() === "ghost") {
      return {
        ...base,
        background: "transparent",
        border: "1px solid rgba(0,0,0,0.08)",
        color: "#94a3b8",
      };
    }
    if (variant() === "danger") {
      return {
        ...base,
        background: "transparent",
        border: "1px solid rgba(239,68,68,0.20)",
        color: "#ef4444",
      };
    }
    return {
      ...base,
      background: "#fff",
      border: "1px solid rgba(0,0,0,0.10)",
      color: "#475569",
    };
  };

  return (
    <Show
      when={!props.href}
      fallback={
        <a href={props.href} style={getStyle()}>
          {props.children}
        </a>
      }
    >
      <button onClick={props.onClick} style={getStyle()}>
        {props.children}
      </button>
    </Show>
  );
}

// ── Logged in ─────────────────────────────────────────────────────────────────
function LoggedInView(props: {
  user: PublicAuthUserInfo;
  onLogout: () => void;
}) {
  const navigate = useNavigate();
  const C = CONTENT.me;

  return (
    <div
      style={{
        "padding-top": "56px",
        "min-height": "100vh",
        background: "#f8fafc",
      }}
    >
      <div
        style={{ "max-width": "800px", margin: "0 auto", padding: "56px 32px" }}
      >
        {/* Header */}
        <div
          style={{
            display: "flex",
            "align-items": "center",
            "justify-content": "space-between",
            "margin-bottom": "48px",
          }}
        >
          <div>
            <div
              style={{
                "font-size": "11px",
                "font-weight": "700",
                "letter-spacing": "0.30em",
                "text-transform": "uppercase",
                color: "#f59e0b",
                "margin-bottom": "8px",
              }}
            >
              {C.logged_in_eyebrow}
            </div>
            <h1
              style={{
                "font-size": "28px",
                "font-weight": "700",
                color: "#0f172a",
                margin: "0",
                "letter-spacing": "-0.01em",
              }}
            >
              {C.logged_in_title}
            </h1>
          </div>
          <div
            style={{
              width: "48px",
              height: "48px",
              "border-radius": "50%",
              background: "#0f172a",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
            }}
          >
            <span
              style={{
                "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                color: "#f59e0b",
                "font-size": "18px",
              }}
            >
              ✦
            </span>
          </div>
        </div>

        {/* Account info */}
        <div
          style={{
            padding: "24px 28px",
            "border-radius": "12px",
            border: "1px solid rgba(0,0,0,0.08)",
            background: "#fff",
            "margin-bottom": "24px",
          }}
        >
          <h3
            style={{
              "font-size": "13px",
              "font-weight": "700",
              "letter-spacing": "0.10em",
              "text-transform": "uppercase",
              color: "#94a3b8",
              margin: "0 0 4px",
            }}
          >
            {C.account_info_title}
          </h3>
          <div>
            <InfoRow label={C.fields.user_id} value={props.user.user_id} />
            <InfoRow
              label={C.fields.created_at}
              value={formatDate(props.user.created_at)}
            />
            <InfoRow
              label={C.fields.last_login}
              value={formatDate(props.user.last_login_at)}
            />
          </div>
        </div>

        {/* Actions */}
        <div
          style={{
            display: "flex",
            "flex-wrap": "wrap",
            gap: "10px",
            "margin-bottom": "40px",
          }}
        >
          <ActionBtn variant="primary" onClick={() => navigate("/chat")}>
            {C.actions.chat}
          </ActionBtn>
          <ActionBtn variant="default" onClick={() => navigate("/roadmap")}>
            {C.actions.roadmap}
          </ActionBtn>
          <ActionBtn variant="ghost" href="#">
            {C.actions.community}
          </ActionBtn>
          <ActionBtn variant="danger" onClick={props.onLogout}>
            {C.actions.logout}
          </ActionBtn>
        </div>

        {/* Membership placeholder */}
        <div
          style={{
            padding: "24px 28px",
            "border-radius": "12px",
            border: "1px dashed rgba(245,158,11,0.20)",
            background: "rgba(245,158,11,0.03)",
          }}
        >
          <div
            style={{ display: "flex", "align-items": "center", gap: "12px" }}
          >
            <span
              style={{
                "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                "font-size": "16px",
                color: "#f59e0b",
                opacity: "0.5",
              }}
            >
              ∞
            </span>
            <div>
              <div
                style={{
                  "font-size": "13px",
                  "font-weight": "700",
                  color: "#0f172a",
                  "margin-bottom": "4px",
                }}
              >
                {C.membership.title}
              </div>
              <div style={{ "font-size": "12px", color: "#94a3b8" }}>
                {C.membership.desc}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// ── PublicMePage ──────────────────────────────────────────────────────────────
export default function PublicMePage() {
  const navigate = useNavigate();
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(null);
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    try {
      const me = await getPublicAuthMe();
      setUser(me);
    } catch {
      setUser(null);
    } finally {
      setLoading(false);
    }
  });

  const handleLogout = async () => {
    try {
      await publicLogout();
    } catch {
      // ignore
    }
    setUser(null);
    navigate("/chat");
  };

  return (
    <div
      class="pub-page"
      style={{
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
        "-webkit-font-smoothing": "antialiased",
      }}
    >
      <PublicNav />
      <Show
        when={!loading()}
        fallback={
          <div
            style={{
              "padding-top": "56px",
              "min-height": "100vh",
              background: "#f8fafc",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
            }}
          >
            <div style={{ "font-size": "13px", color: "#94a3b8" }}>
              {CONTENT.me.loading}
            </div>
          </div>
        }
      >
        <Show when={user()} fallback={<PublicLoginForm onLogin={setUser} />}>
          {(u) => <LoggedInView user={u()} onLogout={handleLogout} />}
        </Show>
      </Show>
      <PublicFooter />
    </div>
  );
}
