// community-influencer-feed.tsx — Insights「大V速报」tab.
//
// Insights and the research desk read the same rolling influencer snapshot
// (`GET /api/public/influencer-digest`), so a post collected for 大V速报 also
// appears here. This view is deliberately separate from the official feed:
// the official timeline's PostgreSQL cursor, read receipts and edge snapshots
// only ever describe HONE-published rows, and attributed author opinion must
// never be labelled 官方 / 只读.

import { Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import {
  InfluencerTimeline,
  createInfluencerDigestView,
  influencerAskQuestion,
  influencerStatusLabel,
  influencerStatusSignal,
} from "@/components/influencer-digest-dashboard";
import type { InfluencerDigestItem } from "@/lib/types";
import "./influencer-digest-dashboard.css";

export function CommunityInfluencerFeed(props: { query: string }) {
  const navigate = useNavigate();
  const view = createInfluencerDigestView({ query: () => props.query });
  const { snapshot, loading, metaLine, visible } = view;

  const askAbout = (item: InfluencerDigestItem) => {
    const question = influencerAskQuestion(item);
    navigate(`/chat?q=${encodeURIComponent(question)}&send=1`);
  };

  return (
    <section class="community-influencer" aria-labelledby="community-influencer-title">
      <header class="community-influencer__head">
        <div>
          <div class="community-influencer__title">
            <h2 id="community-influencer-title">大V速报</h2>
            <Show when={snapshot()}>
              {(current) => (
                <span class={`community-influencer__status is-${influencerStatusSignal(current().status)}`}>
                  {influencerStatusLabel(current().status)}
                </span>
              )}
            </Show>
            <Show when={snapshot()}>
              <span class="community-influencer__count">{visible().length} 条</span>
            </Show>
          </div>
          <p>{metaLine() ?? "Serenity（白毛）、SemiAnalysis 等作者的公开动态，与研究台大V速报同步。"}</p>
        </div>
        <button type="button" disabled={loading()} onClick={() => void view.load()}>
          {loading() && snapshot() ? "刷新中…" : "刷新"}
        </button>
      </header>
      <InfluencerTimeline
        view={view}
        askAbout={askAbout}
        emptyDetail="换一位作者、类型或搜索词再看；未配置来源不会被补造内容。"
      />
      <p class="community-influencer__note">
        {snapshot()?.disclaimer || "作者观点仅供信息参考，不代表 HONE 判断，不构成投资建议。"}
      </p>
    </section>
  );
}
