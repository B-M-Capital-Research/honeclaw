//! Host-side closure for the dedicated earnings workflow when the model's own
//! renderer step does not end with a persisted PDF.
//!
//! The report the model handed to the renderer is the deliverable. The host
//! first reuses a completed renderer PDF, then renders the submitted Markdown
//! itself, and only as a last resort publishes that Markdown with a short note
//! that the PDF is missing. Model failure prose is never published as a report,
//! and nothing here inspects or judges report content.

use std::path::{Path, PathBuf};
use std::time::Duration;

use hone_core::agent::ToolCallMade;
use serde_json::{Value, json};

pub(crate) use crate::earnings_materials::workflow_parameter;
use crate::tool_trace::{
    CompletedEarningsPdf, SubmittedEarningsReport, completed_earnings_pdf,
    latest_submitted_earnings_report,
};

/// Same bound the host gives the model's renderer call.
const HOST_RENDER_TIMEOUT: Duration = Duration::from_secs(115);
pub(crate) const PDF_MISSING_NOTICE: &str =
    "> 本次 PDF 未能生成，以下为完整报告正文；如需 PDF，可稍后重新发起。";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EarningsSalvage {
    Completed(CompletedEarningsPdf),
    Submitted(SubmittedEarningsReport),
}

/// How the terminal earnings answer was produced; the PDF check depends on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EarningsDelivery {
    /// The model's own renderer call in the final trace (the normal path).
    Renderer,
    /// A PDF produced earlier in the run or by the host; it must still attach.
    HostPdf { report_markdown: String },
    /// No PDF could be produced; the submitted report is published as text.
    MarkdownOnly,
}

pub(crate) fn salvage_candidate(tool_calls: &[ToolCallMade]) -> Option<EarningsSalvage> {
    completed_earnings_pdf(tool_calls)
        .map(EarningsSalvage::Completed)
        .or_else(|| latest_submitted_earnings_report(tool_calls).map(EarningsSalvage::Submitted))
}

/// Keep the best deliverable across recovery attempts: a completed PDF is never
/// replaced by a later unrendered draft; otherwise the newest attempt wins.
pub(crate) fn merge_salvage(
    previous: Option<EarningsSalvage>,
    next: Option<EarningsSalvage>,
) -> Option<EarningsSalvage> {
    match (previous, next) {
        (Some(previous @ EarningsSalvage::Completed(_)), Some(EarningsSalvage::Submitted(_))) => {
            Some(previous)
        }
        (previous, None) => previous,
        (_, Some(next)) => Some(next),
    }
}

pub(crate) fn content_with_pdf(report_markdown: &str, pdf_path: &str) -> String {
    format!("{}\n\n[附件: {pdf_path}]", report_markdown.trim_end())
}

pub(crate) fn content_without_pdf(report_markdown: &str) -> String {
    format!("{PDF_MISSING_NOTICE}\n\n{}", report_markdown.trim())
}

pub(crate) fn render_mode<'a>(submitted: Option<&'a str>, runtime_input: &'a str) -> &'a str {
    [submitted, workflow_parameter(runtime_input, "mode")]
        .into_iter()
        .flatten()
        .find(|mode| matches!(*mode, "preview" | "analysis"))
        .unwrap_or("analysis")
}

/// Render the submitted report with the repository renderer into the actor's
/// working directory, exactly as the model's `skill_tool` call would have.
pub(crate) async fn render_on_host(
    script: &Path,
    output_dir: &Path,
    company: &str,
    mode: &str,
    report_markdown: &str,
) -> Result<PathBuf, String> {
    if !script.is_file() {
        return Err("earnings renderer unavailable".to_string());
    }
    let input_dir = tempfile::Builder::new()
        .prefix("hone-earnings-host-render-")
        .tempdir()
        .map_err(|error| error.to_string())?;
    let result = render_with_input(
        script,
        &input_dir.path().join("report.json"),
        output_dir,
        company,
        mode,
        report_markdown,
    )
    .await;
    result
}

async fn render_with_input(
    script: &Path,
    input: &Path,
    output_dir: &Path,
    company: &str,
    mode: &str,
    report_markdown: &str,
) -> Result<PathBuf, String> {
    let spec = json!({
        "company": company,
        "mode": mode,
        "report_markdown": report_markdown,
        "output_name": format!("{company}-{mode}"),
    });
    tokio::fs::write(input, spec.to_string())
        .await
        .map_err(|error| error.to_string())?;
    let mut command = tokio::process::Command::new("python3");
    // Same credential boundary as skill_tool: the renderer inherits no secrets.
    command.env_clear();
    for name in ["PATH", "HOME", "LANG", "LC_ALL", "TMPDIR"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
        .arg(script)
        .arg("--input")
        .arg(input)
        .env("HONE_SKILL_OUTPUT_DIR", output_dir)
        .kill_on_drop(true);
    if let Some(dir) = script.parent() {
        command.current_dir(dir);
    }
    let output = tokio::time::timeout(HOST_RENDER_TIMEOUT, command.output())
        .await
        .map_err(|_| "earnings host render timed out".to_string())?
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "earnings host render exited with {:?}",
            output.status.code()
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("invalid renderer response: {error}"))?;
    if value["success"] != true {
        return Err(value["error"]
            .as_str()
            .unwrap_or("renderer reported failure")
            .chars()
            .take(500)
            .collect());
    }
    let path = value["artifacts"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["kind"] == "document" && item["mime"] == "application/pdf")
        })
        .and_then(|item| item["path"].as_str())
        .ok_or("renderer returned no PDF artifact")?;
    let path = tokio::fs::canonicalize(path)
        .await
        .map_err(|error| error.to_string())?;
    let root = tokio::fs::canonicalize(output_dir)
        .await
        .map_err(|error| error.to_string())?;
    if !path.starts_with(&root) {
        return Err("renderer artifact outside the working directory".to_string());
    }
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|error| error.to_string())?;
    if bytes.len() < 1_000
        || !bytes.starts_with(b"%PDF-")
        || !bytes[bytes.len().saturating_sub(1024)..]
            .windows(5)
            .any(|window| window == b"%%EOF")
    {
        return Err("incomplete PDF artifact".to_string());
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_call(arguments: Value, result: Value) -> ToolCallMade {
        ToolCallMade {
            name: "mcp__hone__skill_tool".to_string(),
            arguments,
            result,
            tool_call_id: Some("render".to_string()),
        }
    }

    fn submitted(report: &str) -> ToolCallMade {
        render_call(
            json!({"skill_name":"earnings-research","execute_script":true,
                "script_payload":{"company":"TEM","mode":"preview","report_markdown":report}}),
            json!({"success":false,"error":"skill script 执行超时","side_effect_status":"uncertain"}),
        )
    }

    fn completed(report: &str) -> ToolCallMade {
        render_call(
            json!({"skill_name":"earnings-research","execute_script":true,
                "script_payload":{"company":"TEM","mode":"preview","report_markdown":report}}),
            json!({"success":true,"render_success":true,"side_effect_status":"completed",
                "validated_report_markdown":report,
                "artifacts":[{"kind":"document","mime":"application/pdf","path":"/sandbox/TEM.pdf"}]}),
        )
    }

    #[test]
    fn a_completed_pdf_outranks_later_drafts_and_failure_prose_is_never_a_candidate() {
        let first = salvage_candidate(&[completed("# 已渲染")]);
        let later = salvage_candidate(&[submitted("# 新草稿")]);
        assert!(matches!(
            merge_salvage(first.clone(), later.clone()),
            Some(EarningsSalvage::Completed(ref pdf)) if pdf.report_markdown == "# 已渲染"
        ));
        assert!(matches!(
            merge_salvage(later.clone(), salvage_candidate(&[submitted("# 最新")])),
            Some(EarningsSalvage::Submitted(ref report)) if report.report_markdown == "# 最新"
        ));
        assert_eq!(merge_salvage(first.clone(), None), first);
        let prose_only = ToolCallMade {
            name: "hone_web_search".to_string(),
            arguments: json!({"query":"TEM"}),
            result: json!({"success":true}),
            tool_call_id: None,
        };
        assert_eq!(salvage_candidate(&[prose_only]), None);
    }

    #[test]
    fn delivered_content_keeps_the_report_and_names_the_pdf_or_its_absence() {
        assert_eq!(
            content_with_pdf("# 报告\n", "/sandbox/TEM.pdf"),
            "# 报告\n\n[附件: /sandbox/TEM.pdf]"
        );
        let text = content_without_pdf("# 报告\n\n正文");
        assert!(text.starts_with(PDF_MISSING_NOTICE));
        assert!(text.ends_with("# 报告\n\n正文"));
    }

    #[test]
    fn workflow_parameters_come_only_from_the_server_block() {
        let input = "mode: analysis\n/earnings-research\n生成前瞻\n\n【HONE 财报工作流参数】\nmode: preview\ncompany: Tempus AI\n此工作流由管理员专属入口触发\n\n研究资料\ncompany: OTHER";
        assert_eq!(workflow_parameter(input, "mode"), Some("preview"));
        assert_eq!(workflow_parameter(input, "company"), Some("Tempus AI"));
        assert_eq!(workflow_parameter("mode: preview", "mode"), None);
        assert_eq!(render_mode(Some("company"), input), "preview");
        assert_eq!(render_mode(Some("analysis"), input), "analysis");
        assert_eq!(render_mode(None, "无参数"), "analysis");
    }

    #[tokio::test]
    async fn host_render_requires_a_complete_pdf_inside_the_working_directory() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join("sandbox");
        std::fs::create_dir_all(&output).expect("sandbox");
        let script = root.path().join("render.py");
        // A stand-in renderer that honours the real `--input` contract.
        std::fs::write(
            &script,
            r#"import json, os, sys
spec = json.load(open(sys.argv[2], encoding="utf-8"))
out = os.path.join(os.environ["HONE_SKILL_OUTPUT_DIR"], spec["company"] + "-" + spec["mode"] + ".pdf")
open(out, "wb").write(b"%PDF-1.7\n" + spec["report_markdown"].encode() * 400 + b"\n%%EOF\n")
print(json.dumps({"success": True, "artifacts": [{"kind": "document", "mime": "application/pdf", "path": out}]}))
"#,
        )
        .expect("script");
        let path = render_on_host(&script, &output, "TEM", "preview", "# 报告")
            .await
            .expect("host render");
        assert!(path.starts_with(output.canonicalize().unwrap()));
        assert!(path.ends_with("TEM-preview.pdf"));

        std::fs::write(
            &script,
            "import json\nprint(json.dumps({'success': False, 'error': 'Chromium PDF render failed'}))\n",
        )
        .expect("failing script");
        assert_eq!(
            render_on_host(&script, &output, "TEM", "preview", "# 报告")
                .await
                .unwrap_err(),
            "Chromium PDF render failed"
        );
        assert!(
            render_on_host(
                &root.path().join("missing.py"),
                &output,
                "TEM",
                "preview",
                "# 报告"
            )
            .await
            .is_err()
        );
    }
}
