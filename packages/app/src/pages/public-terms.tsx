// public-terms.tsx — 用户协议(简体中文,v1.0)

import { For, type JSX, type ParentProps } from "solid-js"
import { PublicNav, PublicFooter } from "@/components/public-nav"
import { TOS_VERSION, TOS_EFFECTIVE_DATE } from "@/lib/tos"
import "./public-site.css"

type Section = { title: string; body: JSX.Element }

const SECTIONS: Section[] = [
  {
    title: "1. 协议接受与生效",
    body: (
      <>
        <p>
          欢迎使用 Hone(以下简称"我们"或"本服务")。本《用户协议》(以下简称"本协议")是您与本服务运营方之间就您使用本服务所订立的有效合同。
        </p>
        <p>
          您在勾选同意或继续使用本服务时,即视为您已充分阅读并同意本协议全部条款。若您不同意本协议任何条款,请立即停止使用本服务。
        </p>
      </>
    ),
  },
  {
    title: "2. 服务说明",
    body: (
      <>
        <p>
          Hone 是一款面向个人投资者的研究与决策辅助工具,提供资料检索、对话式研究、投资笔记、定时提醒等能力。
        </p>
        <p>
          <strong>本服务不构成任何形式的投资建议、要约或推荐。</strong>本服务输出的全部内容仅供参考,任何投资决策均应由您本人独立作出并自行承担相应风险与后果。
        </p>
      </>
    ),
  },
  {
    title: "3. 账号与密码",
    body: (
      <>
        <p>
          您需要使用经我们登记的手机号作为账号,并设置个人密码用于身份验证。您应妥善保管账号密码,不得将账号借予他人使用。
        </p>
        <p>
          您应对在您账号下发生的所有行为负责。若发现账号被未经授权使用,您应立即通知我们并修改密码。
        </p>
      </>
    ),
  },
  {
    title: "4. 用户行为规范",
    body: (
      <>
        <p>使用本服务时,您承诺不从事下列行为:</p>
        <ul>
          <li>违反国家法律法规或公序良俗;</li>
          <li>侵犯他人合法权益,包括知识产权、隐私权、商业秘密等;</li>
          <li>对本服务进行反向工程、爬取、批量自动化访问、漏洞利用或其他形式的滥用;</li>
          <li>上传或传播恶意代码、垃圾信息、违法或不良信息;</li>
          <li>冒用他人身份或伪造账号信息。</li>
        </ul>
      </>
    ),
  },
  {
    title: "5. 内容与知识产权",
    body: (
      <>
        <p>
          本服务及其相关界面、文案、代码、商标等所有相关知识产权归我们或合法权利人所有,受著作权法及相关法律法规保护。
        </p>
        <p>
          您在本服务中输入的内容(包括对话、笔记、附件等)的著作权归您本人所有。您授予我们必要的、为提供和改进本服务所需的非排他性使用权。
        </p>
      </>
    ),
  },
  {
    title: "6. 第三方服务与数据源",
    body: (
      <>
        <p>
          本服务可能调用第三方大型语言模型(LLM)、行情数据、搜索引擎等第三方服务以完成功能交付。第三方服务由其运营方独立提供,其稳定性、准确性及合规性以其官方声明为准。
        </p>
        <p>
          您理解并同意,在调用第三方服务的过程中,我们可能向第三方传递必要的请求内容。我们将依照第三方服务条款选择正规、可信的合作方。
        </p>
      </>
    ),
  },
  {
    title: "7. 服务变更、中断与终止",
    body: (
      <>
        <p>
          我们可能因升级维护、安全事件、不可抗力或经营调整等原因暂停、变更或终止部分或全部服务。我们将在合理范围内事先通过本服务内通知或其他方式告知。
        </p>
        <p>
          若您严重违反本协议,我们有权立即暂停或终止向您提供服务,并保留依法追究责任的权利。
        </p>
      </>
    ),
  },
  {
    title: "8. 免责与责任限制",
    body: (
      <>
        <p>
          在适用法律允许的最大范围内,本服务以"现状"和"现有"方式提供。我们不对服务的连续性、准确性、完整性、及时性作出任何明示或默示保证。
        </p>
        <p>
          因您依赖本服务输出内容作出的投资或交易决定所导致的任何损失,除依法应承担的责任外,我们不对此承担责任。我们的累计赔偿责任以您过去 12 个月内为本服务实际支付的费用为限。
        </p>
      </>
    ),
  },
  {
    title: "9. 协议变更与通知",
    body: (
      <>
        <p>
          我们可能根据法律法规或业务调整需要修改本协议。修改后的协议将在本服务内公布,并标明版本号与生效日期。
        </p>
        <p>
          重大修改将以站内提醒等方式提示您再次确认。若您在协议变更后继续使用本服务,即视为您接受修改后的协议。
        </p>
      </>
    ),
  },
  {
    title: "10. 争议解决与法律适用",
    body: (
      <>
        <p>
          本协议的订立、效力、解释、履行及争议解决,均适用中华人民共和国大陆地区法律(不含港澳台地区法律)。
        </p>
        <p>
          因本协议引起的或与之相关的任何争议,双方应首先协商解决;协商不成的,任何一方可向运营方主要办公地有管辖权的人民法院提起诉讼。
        </p>
      </>
    ),
  },
  {
    title: "11. 联系方式",
    body: (
      <>
        <p>
          若您对本协议有任何疑问、意见或建议,可通过本服务"个人页面"中的反馈入口联系我们。我们将在合理时间内回复并处理。
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

export default function PublicTermsPage() {
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
          用户协议
        </h1>
        <p
          style={{
            "font-size": "14px",
            color: "#94a3b8",
            "margin-bottom": "40px",
            "line-height": "1.6",
          }}
        >
          请仔细阅读以下条款。继续使用 Hone 即表示您接受本协议。
        </p>

        <For each={SECTIONS}>
          {(s) => <Section title={s.title}>{s.body}</Section>}
        </For>
      </div>

      <PublicFooter />
    </div>
  )
}
