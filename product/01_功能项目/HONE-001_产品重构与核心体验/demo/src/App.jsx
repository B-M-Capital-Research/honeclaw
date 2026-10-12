import { lazy, Suspense } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
import { AppShell } from "@/components/AppShell";

const MarketingHome = lazy(() => import("@/pages/MarketingHome").then((m) => ({ default: m.MarketingHome })));
const InvestmentPage = lazy(() => import("@/pages/InvestmentPage").then((m) => ({ default: m.InvestmentPage })));
const CompanyPage = lazy(() => import("@/pages/CompanyPage").then((m) => ({ default: m.CompanyPage })));
const InsightsPage = lazy(() => import("@/pages/InsightsPage").then((m) => ({ default: m.InsightsPage })));
const InsightArticlePage = lazy(() => import("@/pages/InsightArticlePage").then((m) => ({ default: m.InsightArticlePage })));
const AgentPage = lazy(() => import("@/pages/AgentPage").then((m) => ({ default: m.AgentPage })));
const TrackingPage = lazy(() => import("@/pages/TrackingPage").then((m) => ({ default: m.TrackingPage })));
const ProfilePage = lazy(() => import("@/pages/ProfilePage").then((m) => ({ default: m.ProfilePage })));

function RouteFallback() {
  return <div className="grid min-h-svh place-items-center text-sm text-muted-foreground">正在打开 HONE…</div>;
}

export function App() {
  return (
    <TooltipProvider>
      <BrowserRouter>
        <Suspense fallback={<RouteFallback />}>
          <Routes>
            <Route path="/" element={<MarketingHome />} />
            <Route path="/app" element={<AppShell />}>
              <Route index element={<Navigate to="invest" replace />} />
              <Route path="invest" element={<InvestmentPage />} />
              <Route path="invest/company/:ticker" element={<CompanyPage />} />
              <Route path="insights" element={<InsightsPage />} />
              <Route path="insights/:slug" element={<InsightArticlePage />} />
              <Route path="agent" element={<AgentPage />} />
              <Route path="tracking" element={<TrackingPage />} />
              <Route path="me" element={<ProfilePage />} />
            </Route>
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </Suspense>
      </BrowserRouter>
      <Toaster position="top-center" />
    </TooltipProvider>
  );
}
