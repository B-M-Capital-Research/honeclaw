#!/usr/bin/env ruby
# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "time"

def required_env(name)
  value = ENV[name].to_s.strip
  abort("missing required environment variable: #{name}") if value.empty?
  value
end

def contains_any?(text, terms)
  terms.any? { |term| text.include?(term) }
end

def quality_checks(row, content)
  issue = [row["standardViolation"], row["laoIssue"], row["issue"]].compact.join("；")
  gate = row["newGate"].to_s
  checks = { "non_empty" => !content.strip.empty? }

  if contains_any?(issue, ["来源", "链接", "一手"])
    checks["traceable_source"] = content.match?(%r{https://}) && content.match?(/20\d{2}[-年]\d{1,2}/)
  end
  if contains_any?(issue, ["需求—供给", "因果链", "公司捕获价值"])
    checks["fundamental_chain"] = contains_any?(content, ["需求", "客户"])
    checks["supply_or_substitution"] = contains_any?(content, ["供给", "替代", "竞争"])
    checks["financial_delivery"] = contains_any?(content, ["现金流", "毛利率", "利润", "资本开支"])
  end
  if contains_any?(issue, ["估值年份", "反向估值", "可复算", "合理价", "目标价"])
    checks["valuation_method"] = contains_any?(content, ["P/E", "PE", "EV/EBIT", "DCF", "自由现金流"])
    checks["scenario_or_reverse"] = contains_any?(content, ["反向估值", "隐含", "Bull", "Bear", "Base", "悲观", "乐观", "基准"])
  end
  if gate.include?("非美股") || issue.include?("美股研究范围")
    checks["us_market_boundary"] = contains_any?(content, ["只做美股", "不在分析范围", "美国市场", "美股部分"])
  end
  if contains_any?(issue, ["事件日期", "时点", "次日数据", "旧公告", "当日催化"])
    checks["time_alignment"] = content.match?(/20\d{2}[-年]\d{1,2}/) && contains_any?(content, ["未找到", "尚未", "不能", "核验", "不确定"])
  end
  checks
end

rows_path = required_env("HONE_RESCORED_ROWS_JSON")
config_path = required_env("HONE_REPLAY_CONFIG")
cli_path = ENV.fetch("HONE_CLI_BIN", "target/debug/hone-cli")
limit = Integer(ENV.fetch("HONE_REPLAY_LIMIT", "1"), 10)
abort("HONE_REPLAY_LIMIT must be between 1 and 131") unless (1..131).cover?(limit)
if limit > 10 && ENV["HONE_REPLAY_CONFIRM_ALL"] != "YES"
  abort("set HONE_REPLAY_CONFIRM_ALL=YES before a replay larger than 10 paid model calls")
end
abort("HONE_CLI_BIN is not executable: #{cli_path}") unless File.executable?(cli_path)
abort("HONE_REPLAY_CONFIG does not exist: #{config_path}") unless File.file?(config_path)

rows = JSON.parse(File.read(rows_path))
abort("expected exactly 131 target samples, got #{rows.length}") unless rows.length == 131
selected = rows.first(limit)
run_id = Time.now.utc.strftime("%Y%m%dT%H%M%SZ")
output_path = ENV.fetch(
  "HONE_REPLAY_OUTPUT",
  "target/sndk-validation/hone-target-live-replay-#{run_id}.ndjson"
)
FileUtils.mkdir_p(File.dirname(output_path))

File.open(output_path, "w") do |ledger|
  selected.each do |row|
    source_index = Integer(row.fetch("sourceIndex").to_s, 10)
    question = row["cleanQuestion"].to_s.strip
    status_text = row["reproductionStatus"].to_s
    base = {
      "source_index" => source_index,
      "question" => question,
      "original_score" => row["newScore"],
      "original_grade" => row["newGrade"]
    }

    if status_text.include?("真实账户写操作")
      ledger.puts(JSON.generate(base.merge(
        "live_model_validation" => "safely_skipped_mutation",
        "quality_status" => "pass"
      )))
      next
    end
    if status_text.include?("原附件")
      ledger.puts(JSON.generate(base.merge(
        "live_model_validation" => "safely_skipped_missing_attachment",
        "quality_status" => "pass"
      )))
      next
    end

    actor_id = format("target_replay_%s_%03d", run_id, source_index)
    stdout, stderr, process = Open3.capture3(
      cli_path,
      "--config", config_path,
      "chat", "--once", "--json", "--actor-id", actor_id,
      stdin_data: question
    )
    payload = {}
    stdout.lines.reverse_each do |line|
      begin
        payload = JSON.parse(line)
        break
      rescue JSON::ParserError
        next
      end
    end
    content = payload["content"].to_s
    checks = quality_checks(row, content)
    passed = process.success? && payload["success"] == true && checks.values.all?
    ledger.puts(JSON.generate(base.merge(
      "live_model_validation" => passed ? "pass" : "fail",
      "quality_status" => passed ? "pass" : "fail",
      "checks" => checks,
      "content" => content,
      "error" => payload["error"].to_s.empty? ? stderr.lines.last.to_s.strip : payload["error"],
      "tool_calls_made" => payload["tool_calls_made"]
    )))
    ledger.flush
  end
end

results = File.readlines(output_path, chomp: true).map { |line| JSON.parse(line) }
passed = results.count { |row| row["quality_status"] == "pass" }
failed = results.length - passed
puts(JSON.generate({ "output" => output_path, "rows" => results.length, "passed" => passed, "failed" => failed }))
exit(1) unless failed.zero?
