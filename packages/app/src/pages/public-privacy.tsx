// public-privacy.tsx — 隐私政策(简体中文,v1.0)

import { For, type JSX, type ParentProps } from "solid-js"
import { PublicNav, PublicFooter } from "@/components/public-nav"
import { TOS_VERSION, TOS_EFFECTIVE_DATE } from "@/lib/tos"
import "./public-site.css"

type Section = { title: string; body: JSX.Element }

const SECTIONS: Section[] = [
  {
    title: "1. 引言与适用范围",
    body: (
      <>
        <p>
          本《隐私政策》说明 Hone(以下简称"我们")在提供服务过程中如何收集、使用、存储、共享和保护您的个人信息。本政策适用于您通过 Hone 网站及客户端使用本服务的全部场景。
        </p>
        <p>请您在使用本服务前完整阅读本政策。继续使用本服务即视为您已充分了解并同意本政策。</p>
      </>
    ),
  },
  {
    title: "2. 我们收集的信息",
    body: (
      <>
        <p>为提供服务,我们会按最小必要原则收集下列类别的信息:</p>
        <ul>
          <li>
            <strong>账号信息:</strong>手机号(作为账号识别)、邀请码(用于初次注册)、密码哈希(我们仅存储 Argon2id 哈希,绝不存储明文密码);
          </li>
          <li>
            <strong>使用数据:</strong>对话记录、提问与回复内容、上传的附件、笔记与定时任务;
          </li>
          <li>
            <strong>设备与日志:</strong>IP 地址、浏览器类型、访问时间戳、错误日志、Cookie 标识;
          </li>
          <li>
            <strong>授权事件:</strong>用户协议与隐私政策的接受版本与时间。
          </li>
        </ul>
      </>
    ),
  },
  {
    title: "3. 使用目的",
    body: (
      <>
        <p>我们使用上述信息用于以下目的:</p>
        <ul>
          <li>身份认证、登录会话维持、账号风控与频率限制;</li>
          <li>调用大型语言模型与外部数据源以完成您发起的查询;</li>
          <li>记录会话上下文以提供连续对话能力;</li>
          <li>系统故障排查、安全事件响应与服务优化。</li>
        </ul>
      </>
    ),
  },
  {
    title: "4. 存储、保留期与安全",
    body: (
      <>
        <p>
          您的账号与对话数据默认存储于本服务的本地 SQLite 数据库中。密码采用 <strong>Argon2id</strong> 算法配合随机盐进行哈希存储,我们无法恢复您的明文密码。
        </p>
        <p>
          我们采用 HTTPS 加密传输、最小权限访问控制、密码哈希等技术与管理措施,保护您的信息安全。在法律允许范围内,我们将在为完成相应目的所必需的期间内保留您的信息。
        </p>
      </>
    ),
  },
  {
    title: "5. 信息共享与第三方",
    body: (
      <>
        <p>
          为完成您发起的查询,我们可能将您输入的相关内容传递给以下类别的第三方服务方:
        </p>
        <ul>
          <li>大型语言模型提供方(用于生成回复);</li>
          <li>行情数据与搜索数据源(用于补充查询所需的市场或公开信息)。</li>
        </ul>
        <p>
          除上述必要场景以及法律法规另有规定外,我们不会向任何第三方出售或出租您的个人信息。
        </p>
      </>
    ),
  },
  {
    title: "6. Cookie 与追踪",
    body: (
      <>
        <p>
          我们使用名为 <code>hone_web_session</code> 的 HTTP-only Cookie 维持登录态。该 Cookie 在您勾选"保持登录"时有效期为 30 天,否则为 1 天。
        </p>
        <p>我们不使用第三方广告追踪 Cookie。</p>
      </>
    ),
  },
  {
    title: "7. 未成年人保护",
    body: (
      <>
        <p>
          本服务面向 18 周岁以上具有完全民事行为能力的成年人。若您是未成年人,请在监护人指导下使用本服务。我们不会主动收集未成年人的个人信息。
        </p>
      </>
    ),
  },
  {
    title: "8. 跨境传输",
    body: (
      <>
        <p>
          若我们调用的语言模型或数据源服务器位于中华人民共和国大陆地区以外,您的相关查询内容可能被传输至境外。我们会选择具备合规资质的合作方,并采取必要的安全措施。
        </p>
      </>
    ),
  },
  {
    title: "9. 用户权利",
    body: (
      <>
        <p>就您的个人信息,您依法享有下列权利:</p>
        <ul>
          <li>访问、更正您的账号资料;</li>
          <li>修改您的登录密码;</li>
          <li>请求删除您的账号及关联数据;</li>
          <li>撤回您此前给出的同意。</li>
        </ul>
        <p>
          您可在"个人页面"中行使前三项权利,或通过下文联系方式与我们联系。撤回同意可能导致您无法继续使用部分功能。
        </p>
      </>
    ),
  },
  {
    title: "10. 政策更新",
    body: (
      <>
        <p>
          我们可能根据法律法规变化或业务调整需要更新本政策。更新后的政策将在本服务内公布,并标明版本号与生效日期;重大变更将以站内提醒等方式向您提示。
        </p>
      </>
    ),
  },
  {
    title: "11. 联系方式",
    body: (
      <>
        <p>
          若您对本政策或您的个人信息处理有任何疑问、意见或投诉,可通过本服务"个人页面"中的反馈入口联系我们。我们将在合理时间内回复并妥善处理。
        </p>
      </>
    ),
  },
]

function VersionBanner() {
  return (
    <div
      style={{
        display: "inline-flex",
        "align-items": "center",
        gap: "10px",
        padding: "6px 12px",
        "border-radius": "999px",
        background: "rgba(245,158,11,0.08)",
        border: "1px solid rgba(245,158,11,0.25)",
        color: "#d97706",
        "font-size": "12px",
        "font-weight": "600",
        "letter-spacing": "0.02em",
      }}
    >
      v{TOS_VERSION} · {TOS_EFFECTIVE_DATE} 生效
    </div>
  )
}

function Section(props: ParentProps<{ title: string }>) {
  return (
    <section style={{ "margin-bottom": "32px" }}>
      <h2
        style={{
          "font-size": "18px",
          "font-weight": "700",
          color: "#0f172a",
          margin: "0 0 12px",
          "letter-spacing": "-0.01em",
        }}
      >
        {props.title}
      </h2>
      <div
        style={{
          "font-size": "14.5px",
          "line-height": "1.75",
          color: "#334155",
        }}
        class="pub-prose"
      >
        {props.children}
      </div>
    </section>
  )
}

export default function PublicPrivacyPage() {
  return (
    <div
      class="pub-page"
      style={{
        "min-height": "100vh",
        background: "#fff",
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
      }}
    >
      <PublicNav />

      <div
        style={{
          "max-width": "780px",
          margin: "0 auto",
          padding: "120px 24px 64px",
        }}
      >
        <VersionBanner />
        <h1
          style={{
            "font-size": "36px",
            "font-weight": "800",
            color: "#0f172a",
            margin: "20px 0 12px",
            "letter-spacing": "-0.02em",
          }}
        >
          隐私政策
        </h1>
        <p
          style={{
            "font-size": "14px",
            color: "#94a3b8",
            "margin-bottom": "40px",
            "line-height": "1.6",
          }}
        >
          我们在乎您的数据。本政策说明 Hone 如何处理您的个人信息。
        </p>

        <For each={SECTIONS}>
          {(s) => <Section title={s.title}>{s.body}</Section>}
        </For>
      </div>

      <PublicFooter />
    </div>
  )
}
