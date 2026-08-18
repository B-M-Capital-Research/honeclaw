import { Suspense, lazy, type JSX } from "solid-js";

/**
 * The panel each research key opens.
 *
 * Both the research desk and the conversation's tool menu need to mount the
 * same panel, so the mapping lives here rather than being restated at each
 * entry point. Every panel is `lazy`: the conversation should not carry seven
 * dashboards in its bundle for a menu the reader may never open.
 */

const DailySignal = lazy(() =>
  import("@/components/daily-signal-dashboard").then((m) => ({ default: m.DailySignalPanel })),
);
const CompanyRating = lazy(() =>
  import("@/components/company-rating-dashboard").then((m) => ({ default: m.CompanyRatingPanel })),
);
const PortfolioNews = lazy(() =>
  import("@/components/portfolio-news-dashboard").then((m) => ({ default: m.PortfolioNewsPanel })),
);
const PositionManagement = lazy(() =>
  import("@/components/position-management-dashboard").then((m) => ({
    default: m.PositionManagementPanel,
  })),
);
const InfluencerDigest = lazy(() =>
  import("@/components/influencer-digest-dashboard").then((m) => ({
    default: m.InfluencerDigestPanel,
  })),
);
const WeeklyBrief = lazy(() =>
  import("@/components/weekly-brief-dashboard").then((m) => ({ default: m.WeeklyBriefPanel })),
);
const KeyEventChain = lazy(() =>
  import("@/components/key-event-chain-dashboard").then((m) => ({
    default: m.KeyEventChainPanel,
  })),
);

export const RESEARCH_PANEL_KEYS = [
  "daily-signal-macro",
  "daily-signal-ai",
  "company-ratings",
  "portfolio-news",
  "position-management",
  "influencer-digest",
  "weekly-brief",
  "key-event-chain",
] as const;

export type ResearchPanelKey = (typeof RESEARCH_PANEL_KEYS)[number];

export function isResearchPanelKey(value: string | undefined): value is ResearchPanelKey {
  return !!value && (RESEARCH_PANEL_KEYS as readonly string[]).includes(value);
}

/** Renders the panel for `key`, or nothing when the key is unknown. */
export function ResearchPanelFor(props: {
  panel: ResearchPanelKey;
  onClose: () => void;
}): JSX.Element {
  const body = () => {
    switch (props.panel) {
      case "daily-signal-macro":
        return <DailySignal kind="macro" onClose={props.onClose} />;
      case "daily-signal-ai":
        return <DailySignal kind="ai" onClose={props.onClose} />;
      case "company-ratings":
        return <CompanyRating onClose={props.onClose} />;
      case "portfolio-news":
        return <PortfolioNews onClose={props.onClose} />;
      case "position-management":
        return <PositionManagement onClose={props.onClose} />;
      case "influencer-digest":
        return <InfluencerDigest onClose={props.onClose} />;
      case "weekly-brief":
        return <WeeklyBrief onClose={props.onClose} />;
      case "key-event-chain":
        return <KeyEventChain onClose={props.onClose} />;
    }
  };
  return <Suspense>{body()}</Suspense>;
}
