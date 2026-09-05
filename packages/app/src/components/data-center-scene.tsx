import { For, Index, createMemo, createSignal, onCleanup } from "solid-js";
import {
  DATA_CENTER_ZONES,
  type DataCenterZoneId,
} from "@/lib/data-center-model";
import {
  DEFAULT_CAMERA,
  boundCamera,
  boxFaces,
  projectFaces,
  projectPoint,
  type Point3,
  type SceneFace,
} from "@/lib/data-center-geometry";

const ANCHORS: Record<
  DataCenterZoneId,
  { point: Point3; offset: [number, number] }
> = {
  chip: { point: [30, 104, 72], offset: [-10, 105] },
  storage: { point: [184, 97, -93], offset: [76, -54] },
  optical: { point: [-73, 69, 156], offset: [-94, 55] },
  power: { point: [-213, 70, 8], offset: [-105, -27] },
  cooling: { point: [226, 51, 80], offset: [115, 48] },
  software: { point: [-73, 174, -104], offset: [-60, -58] },
};

function makeScene(): SceneFace[] {
  const faces: SceneFace[] = [];
  const box = (
    p: Point3,
    w: number,
    h: number,
    d: number,
    c: readonly [string, string, string],
    zone?: DataCenterZoneId,
  ) => faces.push(...boxFaces(p, w, h, d, c, zone));
  box([0, -21, 0], 580, 18, 406, ["#a8b7bb", "#8d9da4", "#e2ebed"]);
  box([0, -3, 0], 570, 3, 396, ["#d7e2e4", "#c4d1d5", "#f2f6f6"]);
  // Raised floor tiles are world-space geometry, so they rotate with the room.
  for (let x = -270; x <= 270; x += 30)
    box([x, 0, 0], 0.65, 0.1, 390, ["#d5dfe1", "#d5dfe1", "#d5dfe1"]);
  for (let z = -180; z <= 180; z += 30)
    box([0, 0, z], 560, 0.1, 0.65, ["#d5dfe1", "#d5dfe1", "#d5dfe1"]);
  // A room-wide floor cannot be sorted by its centroid against small equipment.
  // It is always below the facility; sort its own tiles before any equipment.
  faces.forEach((face) => {
    face.layer = -1;
  });
  const rack = (
    x: number,
    z: number,
    zone: DataCenterZoneId,
    tint: string,
    height = 92,
  ) => {
    box([x, 2, z], 47, height, 44, ["#263740", "#354a54", "#607580"], zone);
    box(
      [x, 8, z + 22.5],
      38,
      height - 13,
      0.8,
      ["#14262f", "#253b45", "#415965"],
      zone,
    );
    for (let y = 17; y < height; y += 12) {
      box(
        [x, y, z + 23.2],
        31,
        7,
        0.8,
        ["#3e5560", "#3e5560", "#748a91"],
        zone,
      );
      box([x - 11, y + 2, z + 24], 2.6, 2.2, 0.6, [tint, tint, tint], zone);
      box(
        [x + 4, y + 2, z + 24],
        14,
        1,
        0.6,
        ["#172e39", "#172e39", "#172e39"],
        zone,
      );
    }
    box([x, height + 2, z], 37, 1, 33, [tint, tint, tint], zone);
    box([x, height + 3, z], 27, 1, 23, ["#4c626c", "#4c626c", "#4c626c"], zone);
  };
  for (const z of [-18, 70])
    for (const x of [-48, 14, 76]) rack(x, z, "chip", "#e79562");
  for (const x of [155, 216]) rack(x, -111, "storage", "#a89bd5", 88);
  for (const x of [-112, -52]) rack(x, 154, "optical", "#6bc7ba", 58);
  // Busway above the compute racks.
  box([12, 110, 20], 184, 5, 7, ["#daae7c", "#b58c5f", "#edc69b"], "chip");
  for (const x of [-65, 91])
    box([x, 3, 20], 3, 107, 3, ["#85989e", "#a3b0b4", "#c5d0d3"], "chip");
  // Electrical switchgear and backup supply.
  for (const x of [-233, -185]) {
    box([x, 2, 4], 38, 69, 58, ["#e4d4b7", "#c8b696", "#f7e9d0"], "power");
    box([x, 17, 34], 29, 46, 1, ["#baab90", "#baab90", "#e4d4b7"], "power");
    box([x, 49, 35], 13, 8, 1, ["#344a50", "#344a50", "#344a50"], "power");
    box([x + 10, 33, 35], 2, 9, 1, ["#68777a", "#68777a", "#68777a"], "power");
  }
  box([-212, 2, -94], 96, 37, 50, ["#cbbd9e", "#b3a17b", "#ece1c6"], "power");
  for (let x = -249; x < -175; x += 8)
    box([x, 10, -68], 3, 21, 1, ["#8c8776", "#8c8776", "#8c8776"], "power");
  // Cooling units with inset fan grids and a supply/return pipe pair.
  for (const z of [34, 109]) {
    box([228, 2, z], 56, 43, 58, ["#a1c5d0", "#7fa7b5", "#d6e9ec"], "cooling");
    for (const dx of [-12, 12])
      for (const dz of [-13, 13]) {
        box(
          [228 + dx, 45, z + dz],
          19,
          1,
          19,
          ["#7196a1", "#7196a1", "#456e7e"],
          "cooling",
        );
        box(
          [228 + dx, 46, z + dz],
          12,
          1,
          3,
          ["#bddee4", "#bddee4", "#bddee4"],
          "cooling",
        );
        box(
          [228 + dx, 46, z + dz],
          3,
          1,
          12,
          ["#bddee4", "#bddee4", "#bddee4"],
          "cooling",
        );
      }
  }
  for (const z of [125, 134]) {
    box([136, 8, z], 145, 5, 4, ["#70acbd", "#6096a7", "#a6d6df"], "cooling");
    box(
      [68, 8, z - 18],
      4,
      5,
      40,
      ["#70acbd", "#6096a7", "#a6d6df"],
      "cooling",
    );
  }
  // Software is deliberately a floating logical layer, not a physical appliance.
  box(
    [-78, 164, -113],
    150,
    8,
    75,
    ["#8baaca", "#708fae", "#d3e3f0"],
    "software",
  );
  for (const x of [-121, -78, -35]) {
    box(
      [x, 173, -113],
      30,
      5,
      45,
      ["#8fadc7", "#8fadc7", "#f2f7fb"],
      "software",
    );
    for (const z of [-125, -114, -103])
      box(
        [x, 179, z],
        20,
        0.5,
        3,
        ["#87a9c6", "#87a9c6", "#87a9c6"],
        "software",
      );
  }
  return faces;
}

const SCENE = makeScene();
const PATHS: { zone: DataCenterZoneId; points: Point3[]; color: string }[] = [
  {
    zone: "power",
    points: [
      [-192, 3, 43],
      [-150, 3, 43],
      [-150, 3, 105],
      [-12, 3, 105],
      [-12, 3, 70],
    ],
    color: "#c59547",
  },
  {
    zone: "optical",
    points: [
      [-80, 3, 150],
      [-80, 3, 118],
      [109, 3, 118],
      [109, 3, -65],
      [160, 3, -65],
    ],
    color: "#429d8d",
  },
  {
    zone: "storage",
    points: [
      [178, 3, -82],
      [126, 3, -82],
      [126, 3, 20],
      [76, 3, 20],
    ],
    color: "#9f85bd",
  },
];

export function DataCenterScene(props: {
  selected: DataCenterZoneId | null;
  onSelect: (id: DataCenterZoneId, trigger: HTMLButtonElement) => void;
}) {
  const [camera, setCamera] = createSignal({ ...DEFAULT_CAMERA });
  const [dragging, setDragging] = createSignal(false);
  const projected = createMemo(() => projectFaces(SCENE, camera()));
  const position = (point: Point3) => projectPoint(point, camera());
  const labelPosition = (id: DataCenterZoneId) => {
    const anchor = position(ANCHORS[id].point);
    return {
      x: Math.max(165, Math.min(835, anchor.x + ANCHORS[id].offset[0])),
      y: Math.max(85, Math.min(560, anchor.y + ANCHORS[id].offset[1])),
    };
  };
  let drag: { id: number; x: number; yaw: number } | undefined;
  let frame: number | undefined;
  const rotate = (amount: number) =>
    setCamera((current) =>
      boundCamera({ ...current, yaw: current.yaw + amount }),
    );
  const zoom = (amount: number) =>
    setCamera((current) =>
      boundCamera({ ...current, zoom: current.zoom + amount }),
    );
  const finishDrag = () => {
    drag = undefined;
    setDragging(false);
    if (frame !== undefined) cancelAnimationFrame(frame);
    frame = undefined;
  };
  onCleanup(finishDrag);

  return (
    <div class="dc-scene-wrap">
      <div class="dc-scene-topline">
        <span>
          <i /> 交互式产业地图
        </span>
        <span>AI INFRASTRUCTURE</span>
      </div>
      <div class="dc-scene" classList={{ "is-dragging": dragging() }}>
        <svg
          viewBox="0 0 1000 630"
          role="img"
          aria-label="可旋转的 3D 数据中心：供电、算力机柜、光互联、存储、冷却及上方的软件平台层"
          class="dc-scene-svg"
          onPointerDown={(event) => {
            if (event.button !== 0) return;
            drag = { id: event.pointerId, x: event.clientX, yaw: camera().yaw };
            event.currentTarget.setPointerCapture(event.pointerId);
            setDragging(true);
          }}
          onPointerMove={(event) => {
            if (!drag || drag.id !== event.pointerId) return;
            const yaw = drag.yaw + (event.clientX - drag.x) * 0.16;
            if (frame !== undefined) cancelAnimationFrame(frame);
            frame = requestAnimationFrame(() => {
              setCamera((current) => boundCamera({ ...current, yaw }));
              frame = undefined;
            });
          }}
          onPointerUp={finishDrag}
          onPointerCancel={finishDrag}
          onLostPointerCapture={finishDrag}
        >
          <defs>
            <radialGradient id="dc-ground">
              <stop offset="0%" stop-color="#7b919c" stop-opacity=".24" />
              <stop offset="100%" stop-color="#7b919c" stop-opacity="0" />
            </radialGradient>
          </defs>
          <ellipse cx="500" cy="413" rx="378" ry="162" fill="url(#dc-ground)" />
          <Index each={projected()}>
            {(face) => (
              <polygon
                points={face().points}
                fill={face().fill}
                stroke={face().fill}
                stroke-width=".4"
                opacity={
                  props.selected && face().zone && props.selected !== face().zone
                    ? 0.53
                    : 1
                }
              />
            )}
          </Index>
          <For each={PATHS}>
            {(path) => (
              <polyline
                class="dc-flow"
                classList={{ "is-active": props.selected === path.zone }}
                points={path.points
                  .map((p) => {
                    const t = position(p);
                    return `${t.x},${t.y}`;
                  })
                  .join(" ")}
                fill="none"
                stroke={path.color}
                stroke-width="2"
                stroke-dasharray="5 5"
                opacity={
                  props.selected && props.selected !== path.zone ? 0.25 : 0.8
                }
              />
            )}
          </For>
          <For each={[-120, -35]}>
            {(x) => {
              const top = () => position([x, 164, -113]);
              const bottom = () => position([x, 2, -113]);
              return (
                <line
                  x1={top().x}
                  y1={top().y}
                  x2={bottom().x}
                  y2={bottom().y}
                  stroke="#94adc3"
                  stroke-dasharray="3 6"
                  opacity=".5"
                />
              );
            }}
          </For>
          <For each={DATA_CENTER_ZONES}>
            {(zone) => {
              const anchor = () => position(ANCHORS[zone.id].point);
              return (
                <g>
                  <line
                    x1={anchor().x}
                    y1={anchor().y}
                    x2={labelPosition(zone.id).x}
                    y2={labelPosition(zone.id).y}
                    stroke={zone.color}
                    stroke-width="1.4"
                    opacity=".7"
                  />
                  <circle
                    cx={anchor().x}
                    cy={anchor().y}
                    r="4"
                    fill={zone.color}
                    stroke="white"
                    stroke-width="2"
                  />
                </g>
              );
            }}
          </For>
        </svg>
        <For each={DATA_CENTER_ZONES}>
          {(zone) => {
            const label = () => labelPosition(zone.id);
            return (
              <button
                type="button"
                class="dc-hotspot"
                classList={{ "is-selected": props.selected === zone.id }}
                style={{
                  left: `${label().x / 10}%`,
                  top: `${label().y / 6.3}%`,
                  "--zone-color": zone.color,
                }}
                aria-label={`查看${zone.title}`}
                aria-haspopup="dialog"
                aria-expanded={props.selected === zone.id}
                onClick={(event) =>
                  props.onSelect(zone.id, event.currentTarget)
                }
              >
                <span>{zone.shortLabel}</span>
                <strong>{zone.title}</strong>
                <b>↗</b>
              </button>
            );
          }}
        </For>
      </div>
      <div class="dc-scene-controls">
        <span class="dc-gesture-hint">左右拖动旋转 · 点击标签探索</span>
        <div class="dc-control-buttons" role="group" aria-label="模型视角">
          <button
            type="button"
            aria-label="向左旋转"
            disabled={camera().yaw <= 18}
            onClick={() => rotate(-8)}
          >
            ↶
          </button>
          <button
            type="button"
            aria-label="向右旋转"
            disabled={camera().yaw >= 66}
            onClick={() => rotate(8)}
          >
            ↷
          </button>
          <span />
          <button
            type="button"
            aria-label="缩小模型"
            disabled={camera().zoom <= 0.8}
            onClick={() => zoom(-0.1)}
          >
            −
          </button>
          <output aria-label="模型缩放">
            {Math.round(camera().zoom * 100)}%
          </output>
          <button
            type="button"
            aria-label="放大模型"
            disabled={camera().zoom >= 1.35}
            onClick={() => zoom(0.1)}
          >
            +
          </button>
          <span />
          <button
            type="button"
            class="dc-reset"
            onClick={() => setCamera({ ...DEFAULT_CAMERA })}
          >
            复位
          </button>
        </div>
      </div>
    </div>
  );
}
