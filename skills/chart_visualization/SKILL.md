---
name: Chart Visualization
description: 把已有数字渲染成 PNG 图表（趋势/对比/结构占比/分布），当纯文字讲不清楚、或用户直接要图时使用；渲染器支持 line/area/bar/scatter/histogram/horizontal_bar，饼图用 horizontal_bar 顶替
when_to_use: 用户直接要图时使用：饼图、占比图、结构图、柱状图、条形图、折线图、趋势图、分布图、散点图、图表、画图、画一张图、出一张图、出个图、做个图、做一张图、要图片、可视化、chart、plot、graph、pie chart；用户嫌上一轮只有纯文字、没有达到我的要求、要求把结论改画成图片时同样适用
aliases:
  - chart visualization
  - chart
  - plot
  - visualize
  - trend chart
  - 饼图
  - 画图
  - 做图
  - 出图
  - 做个图
  - 出张图
  - 占比图
  - 可视化
  - pie chart
  - 图表
  - 画一张图
  - 柱状图
  - 折线图
  - 趋势图
  - 分布图
  - 散点图
  - 出一张图
  - 出个图
allowed-tools:
  - skill_tool
user-invocable: true
context: inline
arguments:
  - spec_json
script: scripts/render_chart.py
shell: python3
---

## Chart Visualization

Use this skill when a chart will materially improve the answer. Do not render a chart just because it is possible.

### Invocation

On native Codex, resolve this skill's directory from the disclosed `SKILL.md`
path and run the bundled script directly:

```text
python3 <skill-dir>/scripts/render_chart.py '<JSON chart spec>'
```

Parse the script's JSON stdout and use the returned `artifacts` path. On a
legacy Hone runner without native skill execution, call:

```text
skill_tool(
  skill_name="chart_visualization",
  execute_script=true,
  script_arguments={"spec_json":"<JSON chart spec>"}
)
```

The script expects one JSON object string as `spec_json`.

### Required Spec Fields

- `chart_type`
- `title`
- `series`

### Optional Spec Fields

- `subtitle`
- `x_label`
- `y_label`
- `x_values`
- `annotations`
- `footnotes`
- `palette`
- `output_name`

### Supported Chart Types

- `line`
- `area`
- `bar`
- `scatter`
- `histogram`
- `horizontal_bar`

### Series Shape

Each `series` item should usually be:

```json
{
  "name": "Revenue",
  "values": [100, 120, 135]
}
```

You may optionally add a `color`.

### Response Rules

1. Only render a chart when the underlying numbers are concrete and the visual actually clarifies the answer.
2. Keep v1 simple. Prefer one chart, at most two.
3. After a successful render:
   - read `artifacts`
   - place the exact `file:///abs/path/to/chart.png` URI into the final answer where the chart should appear
   - do not wrap that URI in markdown link syntax, HTML `<a>` tags, or image syntax
   - add a short takeaway before and/or after the URI
   - do not expose raw debug output unless the user asked for it
4. If rendering fails, artifacts are empty, or the chart would be misleading, answer in text only.
5. Do not invent numbers just to make a chart.

### Pie Charts And Capability Claims

This skill has no pie renderer. For a 饼图 / 占比图 request, render `horizontal_bar` instead and say in the body that a horizontal share bar is standing in for the pie chart — still return the `file://` image path.

Never tell the user that Hone or the current surface cannot produce a PNG/JPG, and never substitute mermaid code or a text bar (`████ 40%`) for an actual rendered image.

### Cross-Skill Usage

When another finance/research skill already has the needed numbers, that skill should hand off to this skill instead of describing the chart in prose only.
