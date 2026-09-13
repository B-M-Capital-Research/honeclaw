import { apiFetch } from "./backend";

export type CompanyAnalysisTask = {
  task_id: string;
  company: string;
  status: "running" | "interrupted" | "failed" | "completed";
  progress: number;
  info: string;
  created_at: string;
  updated_at: string;
  pdf_ready: boolean;
  completed_stages: string[];
  warnings?: string[];
};

const ROOT = "/api/public/company-analysis/tasks";
async function checked(response: Response): Promise<Response> {
  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    throw new Error(body.error || body.message || "公司分析暂时不可用，请稍后重试");
  }
  return response;
}

export async function listCompanyAnalyses(signal?: AbortSignal): Promise<CompanyAnalysisTask[]> {
  const response = await checked(await apiFetch(ROOT, { signal }));
  return (await response.json()).tasks;
}

export async function startCompanyAnalysis(company: string): Promise<CompanyAnalysisTask> {
  const response = await checked(await apiFetch(ROOT, {
    method: "POST", headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ company: company.trim() }),
  }));
  return response.json();
}

export async function getCompanyAnalysis(id: string, signal?: AbortSignal): Promise<CompanyAnalysisTask> {
  return (await checked(await apiFetch(`${ROOT}/${encodeURIComponent(id)}`, { signal }))).json();
}

export async function resumeCompanyAnalysis(id: string): Promise<CompanyAnalysisTask> {
  return (await checked(await apiFetch(`${ROOT}/${encodeURIComponent(id)}/resume`, { method: "POST" }))).json();
}

export async function downloadCompanyAnalysis(id: string): Promise<Blob> {
  const response = await checked(await apiFetch(`${ROOT}/${encodeURIComponent(id)}/pdf`));
  if (!response.headers.get("Content-Type")?.startsWith("application/pdf")) {
    throw new Error("PDF 下载响应异常，请重试");
  }
  return response.blob();
}

export function canResumeCompanyAnalysis(task: CompanyAnalysisTask): boolean {
  return task.status === "failed" || task.status === "interrupted";
}

export function companyAnalysisProgress(task: CompanyAnalysisTask): number {
  return task.status === "completed" && task.pdf_ready
    ? 100
    : Math.max(0, Math.min(99, Number.isFinite(task.progress) ? task.progress : 0));
}
