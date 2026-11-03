/** Maps a scroll offset proportionally between two scroll ranges. */
export function mapScroll(top: number, srcMax: number, dstMax: number): number {
  if (srcMax <= 0 || dstMax <= 0) return 0;
  return Math.min(dstMax, Math.max(0, (top / srcMax) * dstMax));
}
