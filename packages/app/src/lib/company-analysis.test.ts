import { afterEach, expect, test } from "bun:test";
import { canResumeCompanyAnalysis, companyAnalysisProgress, companyAnalysisToShow, downloadCompanyAnalysis,
  downloadCompanyAnalysisMarkdown, startCompanyAnalysis, type CompanyAnalysisTask } from "./company-analysis";
const originalFetch = globalThis.fetch;
afterEach(() => { globalThis.fetch = originalFetch; });
const task = {task_id:"task",company:"TEM",status:"running",progress:95,info:"保存中",created_at:"",updated_at:"",pdf_ready:false,completed_stages:[]} satisfies CompanyAnalysisTask;

test("company start sends only the company and never retries an uncertain POST", async () => {
  let calls = 0; let captured: RequestInit | undefined;
  globalThis.fetch = (async (_: unknown, init?: RequestInit) => { calls++; captured = init; throw new Error("lost response"); }) as unknown as typeof fetch;
  await expect(startCompanyAnalysis(" TEM ")).rejects.toThrow();
  expect(calls).toBe(1); expect(JSON.parse(String(captured?.body))).toEqual({company:"TEM"});
  expect(captured?.credentials).toBe("include");
});
test("progress cannot display completion before the PDF is ready", () => {
  expect(companyAnalysisProgress({...task,progress:100})).toBe(99);
  expect(companyAnalysisProgress({...task,status:"completed",progress:100,pdf_ready:true})).toBe(100);
  expect(canResumeCompanyAnalysis({...task,status:"interrupted"})).toBe(true);
  expect(canResumeCompanyAnalysis(task)).toBe(false);
});
test("the Markdown report downloads separately and rejects HTML", async () => {
  let url = "";
  globalThis.fetch = (async (input: unknown) => { url = String(input); return new Response("# TEM",{headers:{"Content-Type":"text/markdown; charset=utf-8"}}); }) as unknown as typeof fetch;
  expect(await (await downloadCompanyAnalysisMarkdown("task")).text()).toBe("# TEM");
  expect(url).toContain("/api/public/company-analysis/tasks/task/markdown");
  globalThis.fetch = (async () => new Response("<html>login</html>",{headers:{"Content-Type":"text/html"}})) as unknown as typeof fetch;
  await expect(downloadCompanyAnalysisMarkdown("task")).rejects.toThrow("Markdown 下载响应异常");
});
test("opening the dialog shows the active task, else the latest finished or failed one", () => {
  const failed = {...task,task_id:"failed",status:"failed",markdown_ready:true} satisfies CompanyAnalysisTask;
  const done = {...task,task_id:"done",status:"completed",pdf_ready:true} satisfies CompanyAnalysisTask;
  const interrupted = {...task,task_id:"interrupted",status:"interrupted"} satisfies CompanyAnalysisTask;
  expect(companyAnalysisToShow([failed, interrupted])?.task_id).toBe("interrupted");
  expect(companyAnalysisToShow([failed, done])?.task_id).toBe("failed");
  expect(companyAnalysisToShow([])).toBeUndefined();
});
test("PDF download rejects a login or HTML response instead of downloading a false PDF", async () => {
  globalThis.fetch = (async () => new Response("<html>login</html>",{headers:{"Content-Type":"text/html"}})) as unknown as typeof fetch;
  await expect(downloadCompanyAnalysis("task")).rejects.toThrow("PDF 下载响应异常");
});
