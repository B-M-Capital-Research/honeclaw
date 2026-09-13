import { afterEach, expect, test } from "bun:test";
import { canResumeCompanyAnalysis, companyAnalysisProgress, downloadCompanyAnalysis,
  startCompanyAnalysis, type CompanyAnalysisTask } from "./company-analysis";
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
test("PDF download rejects a login or HTML response instead of downloading a false PDF", async () => {
  globalThis.fetch = (async () => new Response("<html>login</html>",{headers:{"Content-Type":"text/html"}})) as unknown as typeof fetch;
  await expect(downloadCompanyAnalysis("task")).rejects.toThrow("PDF 下载响应异常");
});
