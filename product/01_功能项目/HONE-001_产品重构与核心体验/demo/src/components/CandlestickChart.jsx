import { useEffect, useRef } from "react";
import {
  CandlestickSeries,
  ColorType,
  createChart,
  createSeriesMarkers,
  HistogramSeries,
  LineStyle,
} from "lightweight-charts";
import { companyCandles } from "@/data";

export function CandlestickChart({ compact = false }) {
  const containerRef = useRef(null);

  useEffect(() => {
    if (!containerRef.current) return;
    const container = containerRef.current;
    const chart = createChart(container, {
      width: container.clientWidth,
      height: compact ? 260 : 360,
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: "#737373",
        fontFamily: "Geist, system-ui, sans-serif",
        fontSize: compact ? 10 : 11,
      },
      grid: {
        vertLines: { color: "#f1f1f1" },
        horzLines: { color: "#eeeeee" },
      },
      crosshair: {
        vertLine: { color: "#737373", width: 1, style: LineStyle.Dashed, labelBackgroundColor: "#171717" },
        horzLine: { color: "#737373", width: 1, style: LineStyle.Dashed, labelBackgroundColor: "#171717" },
      },
      rightPriceScale: { borderColor: "#e5e5e5", scaleMargins: { top: 0.08, bottom: 0.24 } },
      timeScale: { borderColor: "#e5e5e5", timeVisible: false, rightOffset: 1, barSpacing: compact ? 7 : 12 },
      handleScroll: true,
      handleScale: true,
    });
    const candles = chart.addSeries(CandlestickSeries, {
      upColor: "#ffffff",
      downColor: "#171717",
      borderVisible: true,
      borderUpColor: "#171717",
      borderDownColor: "#171717",
      wickUpColor: "#737373",
      wickDownColor: "#171717",
      priceLineVisible: false,
    });
    candles.setData(companyCandles.map(({ volume, ...item }) => item));
    candles.createPriceLine({
      price: 102.43,
      color: "#737373",
      lineWidth: 1,
      lineStyle: LineStyle.Dashed,
      axisLabelVisible: true,
      title: "我的成本",
    });
    createSeriesMarkers(candles, [
      { time: "2026-06-09", position: "belowBar", color: "#171717", shape: "arrowUp", text: "B" },
      { time: "2026-06-29", position: "belowBar", color: "#171717", shape: "arrowUp", text: "B" },
    ]);
    const volume = chart.addSeries(HistogramSeries, {
      color: "#d4d4d4",
      priceFormat: { type: "volume" },
      priceScaleId: "volume",
      priceLineVisible: false,
      lastValueVisible: false,
    });
    volume.priceScale().applyOptions({ scaleMargins: { top: 0.82, bottom: 0 } });
    volume.setData(companyCandles.map((item) => ({ time: item.time, value: item.volume, color: item.close >= item.open ? "#d4d4d4" : "#737373" })));
    chart.timeScale().fitContent();

    const resizeObserver = new ResizeObserver(() => {
      chart.applyOptions({ width: container.clientWidth, height: compact ? 260 : 360 });
    });
    resizeObserver.observe(container);
    return () => { resizeObserver.disconnect(); chart.remove(); };
  }, [compact]);

  return <div ref={containerRef} className="w-full" aria-label="NVIDIA K 线图，包含成交量、平均成本与历史买入点" />;
}
