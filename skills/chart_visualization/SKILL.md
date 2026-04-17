---
name: Chart Visualization
description: Render research-style PNG charts for trends, comparisons, distributions, and scatter relationships when a visual materially improves the answer
when_to_use: Use when the user asks for a chart, plot, visualization, trend line, bar chart, histogram, scatter plot, or when a numeric answer is easier to understand as a visual
aliases:
  - chart visualization
  - chart
  - plot
  - visualize
  - trend chart
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

Call:

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

### Cross-Skill Usage

When another finance/research skill already has the needed numbers, that skill should hand off to this skill instead of describing the chart in prose only.
