#!/usr/bin/env python3

import json
import os
import re
import sys
import uuid
from pathlib import Path


def emit(payload: dict) -> int:
    print(json.dumps(payload, ensure_ascii=False))
    return 0


def failure(error: str, fallback_message: str) -> int:
    return emit(
        {
            "success": False,
            "error": error,
            "fallback_message": fallback_message,
            "artifacts": [],
            "warnings": [],
        }
    )


try:
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
except Exception as exc:  # pragma: no cover - runtime dependency probe
    sys.exit(
        failure(
            f"matplotlib unavailable: {exc}",
            "图表渲染环境不可用，改为文字说明。",
        )
    )


DEFAULT_PALETTE = [
    "#1f3c88",
    "#0f766e",
    "#b45309",
    "#7c3aed",
    "#b91c1c",
    "#475569",
]


def sanitize_output_name(raw: str) -> str:
    cleaned = re.sub(r"[^a-zA-Z0-9._-]+", "-", raw.strip()).strip("-._")
    return cleaned or "chart"


def output_dir() -> Path:
    data_dir = os.environ.get("HONE_DATA_DIR", "./data")
    gen_root = os.environ.get(
        "HONE_GEN_IMAGES_DIR", str(Path(data_dir).joinpath("gen_images"))
    )
    session_id = os.environ.get("HONE_SESSION_ID", "").strip() or "adhoc"
    out_dir = Path(gen_root).expanduser().resolve() / session_id
    out_dir.mkdir(parents=True, exist_ok=True)
    return out_dir


def load_spec() -> dict:
    if len(sys.argv) < 2:
        raise ValueError("missing spec_json argument")
    raw = sys.argv[1]
    payload = json.loads(raw)
    if not isinstance(payload, dict):
        raise ValueError("spec_json must decode to a JSON object")
    return payload


def require_string(spec: dict, key: str) -> str:
    value = spec.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{key} is required and must be a non-empty string")
    return value.strip()


def require_series(spec: dict) -> list[dict]:
    series = spec.get("series")
    if not isinstance(series, list) or not series:
        raise ValueError("series is required and must be a non-empty list")
    normalized = []
    for index, item in enumerate(series):
        if not isinstance(item, dict):
            raise ValueError(f"series[{index}] must be an object")
        values = item.get("values")
        if not isinstance(values, list) or not values:
            raise ValueError(f"series[{index}].values must be a non-empty list")
        name = item.get("name")
        if name is None:
            name = f"Series {index + 1}"
        normalized.append(
            {
                "name": str(name),
                "values": values,
                "color": item.get("color"),
            }
        )
    return normalized


def coerce_numbers(values: list, field: str) -> list[float]:
    numbers = []
    for index, value in enumerate(values):
        try:
            numbers.append(float(value))
        except Exception as exc:
            raise ValueError(f"{field}[{index}] must be numeric: {exc}") from exc
    return numbers


def build_x_positions(spec: dict, series: list[dict]) -> tuple[list, list]:
    x_values = spec.get("x_values")
    if x_values is None:
        length = len(series[0]["values"])
        return list(range(length)), list(range(length))
    if not isinstance(x_values, list) or not x_values:
        raise ValueError("x_values must be a non-empty list when provided")
    if len(x_values) != len(series[0]["values"]):
        raise ValueError("x_values length must match the first series length")

    if all(isinstance(item, (int, float)) for item in x_values):
        return [float(item) for item in x_values], x_values
    return list(range(len(x_values))), [str(item) for item in x_values]


def apply_common_style(fig, ax, spec: dict) -> None:
    fig.patch.set_facecolor("white")
    ax.set_facecolor("white")
    ax.grid(True, axis="y", color="#d9dee7", linewidth=0.8)
    ax.grid(False, axis="x")
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.spines["left"].set_color("#cbd5e1")
    ax.spines["bottom"].set_color("#cbd5e1")
    ax.tick_params(axis="both", colors="#334155")
    title = require_string(spec, "title")
    subtitle = spec.get("subtitle")
    if isinstance(subtitle, str) and subtitle.strip():
        ax.set_title(f"{title}\n{subtitle.strip()}", loc="left", fontsize=22, pad=18)
    else:
        ax.set_title(title, loc="left", fontsize=22, pad=18)
    if isinstance(spec.get("x_label"), str):
        ax.set_xlabel(spec["x_label"], color="#334155")
    if isinstance(spec.get("y_label"), str):
        ax.set_ylabel(spec["y_label"], color="#334155")


def apply_footnotes(fig, spec: dict) -> None:
    footnotes = spec.get("footnotes")
    if isinstance(footnotes, str) and footnotes.strip():
        footnote_text = footnotes.strip()
    elif isinstance(footnotes, list):
        parts = [str(item).strip() for item in footnotes if str(item).strip()]
        footnote_text = "\n".join(parts)
    else:
        footnote_text = ""
    if footnote_text:
        fig.text(0.08, 0.03, footnote_text, ha="left", va="bottom", fontsize=10, color="#475569")


def apply_annotations(ax, spec: dict, series: list[dict], x_positions: list) -> None:
    annotations = spec.get("annotations")
    if not isinstance(annotations, list):
        return
    for item in annotations:
        if not isinstance(item, dict):
            continue
        text = str(item.get("text", "")).strip()
        if not text:
            continue
        if isinstance(item.get("series_index"), int) and isinstance(item.get("point_index"), int):
            series_index = item["series_index"]
            point_index = item["point_index"]
            if 0 <= series_index < len(series) and 0 <= point_index < len(series[series_index]["values"]):
                y_values = coerce_numbers(series[series_index]["values"], f"series[{series_index}].values")
                ax.annotate(
                    text,
                    (x_positions[point_index], y_values[point_index]),
                    textcoords="offset points",
                    xytext=(6, 8),
                    fontsize=10,
                    color="#0f172a",
                )
                continue
        if isinstance(item.get("x"), (int, float)) and isinstance(item.get("y"), (int, float)):
            ax.annotate(
                text,
                (float(item["x"]), float(item["y"])),
                textcoords="offset points",
                xytext=(6, 8),
                fontsize=10,
                color="#0f172a",
            )


def plot_line_like(ax, spec: dict, series: list[dict], palette: list[str], fill: bool) -> None:
    x_positions, x_labels = build_x_positions(spec, series)
    for index, item in enumerate(series):
        y_values = coerce_numbers(item["values"], f"series[{index}].values")
        if len(y_values) != len(x_positions):
            raise ValueError(f"series[{index}] length must match x_values length")
        color = item["color"] if isinstance(item.get("color"), str) else palette[index % len(palette)]
        ax.plot(x_positions, y_values, linewidth=2.5, color=color, label=item["name"])
        if fill:
            ax.fill_between(x_positions, y_values, color=color, alpha=0.18)
    ax.set_xticks(x_positions)
    ax.set_xticklabels(x_labels)
    apply_annotations(ax, spec, series, x_positions)


def plot_scatter(ax, spec: dict, series: list[dict], palette: list[str]) -> None:
    x_positions, x_labels = build_x_positions(spec, series)
    for index, item in enumerate(series):
        y_values = coerce_numbers(item["values"], f"series[{index}].values")
        if len(y_values) != len(x_positions):
            raise ValueError(f"series[{index}] length must match x_values length")
        color = item["color"] if isinstance(item.get("color"), str) else palette[index % len(palette)]
        ax.scatter(x_positions, y_values, s=70, color=color, label=item["name"], alpha=0.88)
    ax.set_xticks(x_positions)
    ax.set_xticklabels(x_labels)
    apply_annotations(ax, spec, series, x_positions)


def plot_bar(ax, spec: dict, series: list[dict], palette: list[str], horizontal: bool) -> None:
    categories_raw = spec.get("x_values")
    if not isinstance(categories_raw, list) or not categories_raw:
        categories_raw = [str(index + 1) for index in range(len(series[0]["values"]))]
    categories = [str(item) for item in categories_raw]
    base = list(range(len(categories)))
    width = 0.8 / max(len(series), 1)

    for index, item in enumerate(series):
        values = coerce_numbers(item["values"], f"series[{index}].values")
        if len(values) != len(categories):
            raise ValueError(f"series[{index}] length must match category count")
        offset = (index - (len(series) - 1) / 2.0) * width
        color = item["color"] if isinstance(item.get("color"), str) else palette[index % len(palette)]
        if horizontal:
            ax.barh([position + offset for position in base], values, height=width, color=color, label=item["name"])
        else:
            ax.bar([position + offset for position in base], values, width=width, color=color, label=item["name"])

    if horizontal:
        ax.set_yticks(base)
        ax.set_yticklabels(categories)
    else:
        ax.set_xticks(base)
        ax.set_xticklabels(categories)


def plot_histogram(ax, spec: dict, series: list[dict], palette: list[str]) -> None:
    bins = spec.get("bins", 10)
    for index, item in enumerate(series):
        values = coerce_numbers(item["values"], f"series[{index}].values")
        color = item["color"] if isinstance(item.get("color"), str) else palette[index % len(palette)]
        ax.hist(values, bins=bins, alpha=0.5, color=color, label=item["name"])


def render(spec: dict) -> Path:
    chart_type = require_string(spec, "chart_type").lower()
    series = require_series(spec)
    palette = spec.get("palette")
    if isinstance(palette, list) and palette:
        colors = [str(item) for item in palette]
    else:
        colors = DEFAULT_PALETTE

    fig, ax = plt.subplots(figsize=(16, 9), dpi=100)
    apply_common_style(fig, ax, spec)

    if chart_type == "line":
        plot_line_like(ax, spec, series, colors, fill=False)
    elif chart_type == "area":
        plot_line_like(ax, spec, series, colors, fill=True)
    elif chart_type == "scatter":
        plot_scatter(ax, spec, series, colors)
    elif chart_type == "bar":
        plot_bar(ax, spec, series, colors, horizontal=False)
    elif chart_type == "horizontal_bar":
        plot_bar(ax, spec, series, colors, horizontal=True)
    elif chart_type == "histogram":
        plot_histogram(ax, spec, series, colors)
    else:
        plt.close(fig)
        raise ValueError(f"unsupported chart_type: {chart_type}")

    if len(series) > 1:
        ax.legend(frameon=False, loc="best")
    apply_footnotes(fig, spec)
    fig.tight_layout(rect=(0.04, 0.06, 0.98, 0.95))

    base_name = sanitize_output_name(str(spec.get("output_name") or spec.get("title") or "chart"))
    path = output_dir() / f"{base_name}-{uuid.uuid4().hex[:8]}.png"
    fig.savefig(path, format="png", bbox_inches="tight")
    plt.close(fig)
    return path.resolve()


def main() -> int:
    try:
        spec = load_spec()
        chart_path = render(spec)
        return emit(
            {
                "success": True,
                "summary": f"Rendered {spec['chart_type']} chart to PNG",
                "artifacts": [
                    {
                        "kind": "image",
                        "path": str(chart_path),
                        "mime": "image/png",
                    }
                ],
                "warnings": [],
            }
        )
    except Exception as exc:
        return failure(str(exc), "图表渲染失败，改为文字说明。")


if __name__ == "__main__":
    sys.exit(main())
