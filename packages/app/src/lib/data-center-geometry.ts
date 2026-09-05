/** A small orthographic 3D scene: world coordinates remain independent of viewport pixels. */
export type Point3 = readonly [number, number, number];
export type Camera = { yaw: number; zoom: number };
export const DEFAULT_CAMERA: Camera = { yaw: 36, zoom: 1 };

export function boundCamera(camera: Camera): Camera {
  return {
    yaw: Math.max(
      18,
      Math.min(
        66,
        Number.isFinite(camera.yaw) ? camera.yaw : DEFAULT_CAMERA.yaw,
      ),
    ),
    zoom: Math.max(
      0.8,
      Math.min(1.35, Number.isFinite(camera.zoom) ? camera.zoom : 1),
    ),
  };
}

export function projectPoint([x, y, z]: Point3, camera: Camera) {
  const { yaw, zoom } = boundCamera(camera);
  const radians = (yaw * Math.PI) / 180;
  const horizontal = x * Math.cos(radians) - z * Math.sin(radians);
  const depth = x * Math.sin(radians) + z * Math.cos(radians);
  return {
    x: 500 + horizontal * zoom,
    y: 365 + (depth * 0.52 - y * 0.854) * zoom,
    depth: depth * 0.854 + y * 0.52,
  };
}

export type SceneFace = {
  points: Point3[];
  fill: string;
  zone?: string;
  layer?: number;
};

/** Only these three faces can face the camera throughout its bounded orbit. */
export function boxFaces(
  center: Point3,
  width: number,
  height: number,
  depth: number,
  colors: readonly [string, string, string],
  zone?: string,
): SceneFace[] {
  const [x, y, z] = center;
  const l = x - width / 2,
    r = x + width / 2,
    b = z - depth / 2,
    f = z + depth / 2,
    t = y + height;
  return [
    {
      points: [
        [l, y, f],
        [r, y, f],
        [r, t, f],
        [l, t, f],
      ],
      fill: colors[0],
      zone,
    },
    {
      points: [
        [r, y, b],
        [r, y, f],
        [r, t, f],
        [r, t, b],
      ],
      fill: colors[1],
      zone,
    },
    {
      points: [
        [l, t, b],
        [r, t, b],
        [r, t, f],
        [l, t, f],
      ],
      fill: colors[2],
      zone,
    },
  ];
}

export function projectFaces(faces: SceneFace[], camera: Camera) {
  return faces
    .map((face, order) => {
      const points = face.points.map((point) => projectPoint(point, camera));
      return {
        ...face,
        order,
        points: points.map((point) => `${point.x},${point.y}`).join(" "),
        depth:
          points.reduce((sum, point) => sum + point.depth, 0) / points.length,
      };
    })
    .sort(
      (a, b) =>
        (a.layer ?? 0) - (b.layer ?? 0) ||
        a.depth - b.depth ||
        a.order - b.order,
    );
}
