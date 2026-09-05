import { describe, expect, test } from "bun:test";
import {
  DEFAULT_CAMERA,
  boundCamera,
  boxFaces,
  projectFaces,
  projectPoint,
} from "./data-center-geometry";

describe("data center camera and projection", () => {
  test("a tall rack projects upward while world depth stays consistent for occlusion", () => {
    const base = projectPoint([25, 0, 20], DEFAULT_CAMERA);
    const top = projectPoint([25, 100, 20], DEFAULT_CAMERA);
    expect(top.x).toBe(base.x);
    expect(top.y).toBeLessThan(base.y);
    expect(top.depth).toBeGreaterThan(base.depth);
    const back = projectPoint([25, 0, -120], DEFAULT_CAMERA);
    expect(back.depth).toBeLessThan(base.depth);
  });

  test("zoom scales points around the scene origin without moving the origin", () => {
    const normal = projectPoint([100, 30, 20], DEFAULT_CAMERA);
    const zoomed = projectPoint([100, 30, 20], {
      ...DEFAULT_CAMERA,
      zoom: 1.2,
    });
    expect(zoomed.x - 500).toBeCloseTo((normal.x - 500) * 1.2);
    expect(zoomed.y - 365).toBeCloseTo((normal.y - 365) * 1.2);
    expect(projectPoint([0, 0, 0], { yaw: 66, zoom: 1.35 })).toEqual({
      x: 500,
      y: 365,
      depth: 0,
    });
  });

  test("dragging and zoom controls cannot expose unmodeled faces or create invalid geometry", () => {
    expect(boundCamera({ yaw: -999, zoom: 0 })).toEqual({ yaw: 18, zoom: 0.8 });
    expect(boundCamera({ yaw: 999, zoom: 99 })).toEqual({
      yaw: 66,
      zoom: 1.35,
    });
    expect(boundCamera({ yaw: NaN, zoom: Infinity })).toEqual(DEFAULT_CAMERA);
    for (const yaw of [18, 36, 66]) {
      const projected = projectFaces(
        boxFaces([0, 0, 0], 50, 100, 40, ["front", "side", "top"]),
        { yaw, zoom: 1 },
      );
      expect(projected).toHaveLength(3);
      for (const face of projected)
        expect(face.points).not.toMatch(/NaN|Infinity/);
      expect(projected.map((face) => face.depth)).toEqual(
        projected.map((face) => face.depth).sort((a, b) => a - b),
      );
      expect(projected[2].fill).toBe("top");
    }
  });

  test("the floor never occludes equipment at the back of the room", () => {
    const floor = boxFaces([0, -20, 0], 580, 20, 406, [
      "floor",
      "floor",
      "floor",
    ]).map((face) => ({ ...face, layer: -1 }));
    const cabinet = boxFaces([-230, 0, -94], 38, 70, 50, [
      "cabinet",
      "cabinet",
      "cabinet",
    ]);
    for (const yaw of [18, 36, 66]) {
      const rendered = projectFaces([...cabinet, ...floor], { yaw, zoom: 1 });
      expect(rendered.slice(0, 3).every((face) => face.fill === "floor")).toBe(
        true,
      );
      expect(rendered.slice(3).every((face) => face.fill === "cabinet")).toBe(
        true,
      );
    }
  });
});
