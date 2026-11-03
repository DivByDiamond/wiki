export interface View { scale: number; x: number; y: number }

export const MIN_SCALE = 1;
export const MAX_SCALE = 8;
export const RESET: View = { scale: 1, x: 0, y: 0 };

export function clampScale(s: number): number {
  return Math.min(MAX_SCALE, Math.max(MIN_SCALE, s));
}

/** Zoom by `factor` keeping the point (px, py), relative to the stage centre, fixed on screen. */
export function zoomAt(v: View, factor: number, px: number, py: number): View {
  const scale = clampScale(v.scale * factor);
  if (scale === MIN_SCALE) return RESET;
  const ratio = scale / v.scale;
  return { scale, x: px - (px - v.x) * ratio, y: py - (py - v.y) * ratio };
}

/** Keep the zoomed image from being dragged fully out of the stage. */
export function limitPan(v: View, stageW: number, stageH: number, imgW: number, imgH: number): View {
  const maxX = Math.max(0, (imgW * v.scale - stageW) / 2 + stageW / 4);
  const maxY = Math.max(0, (imgH * v.scale - stageH) / 2 + stageH / 4);
  return { ...v, x: Math.min(maxX, Math.max(-maxX, v.x)), y: Math.min(maxY, Math.max(-maxY, v.y)) };
}
