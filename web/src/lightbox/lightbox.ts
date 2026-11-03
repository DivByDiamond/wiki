import { limitPan, RESET, zoomAt, type View } from "./zoom.ts";

const LABELS: Record<string, Record<string, string>> = {
  en: { close: "Close", zoomIn: "Zoom in", zoomOut: "Zoom out", reset: "Reset zoom", prev: "Previous image", next: "Next image" },
  ru: { close: "Закрыть", zoomIn: "Приблизить", zoomOut: "Отдалить", reset: "Сбросить масштаб", prev: "Предыдущее фото", next: "Следующее фото" },
  zh: { close: "关闭", zoomIn: "放大", zoomOut: "缩小", reset: "重置缩放", prev: "上一张", next: "下一张" },
};

const SELECTOR = ".prose img";

function label(key: string): string {
  const lang = (document.documentElement.lang || "en").slice(0, 2);
  return (LABELS[lang] ?? LABELS.en)[key] ?? LABELS.en[key];
}

function button(cls: string, key: string, text: string): HTMLButtonElement {
  const b = document.createElement("button");
  b.type = "button";
  b.className = `lightbox-btn ${cls}`;
  b.textContent = text;
  b.title = label(key);
  b.setAttribute("aria-label", label(key));
  return b;
}

export function mountLightbox(): void {
  const images = () => Array.from(document.querySelectorAll<HTMLImageElement>(SELECTOR));
  if (!images().length) return;

  let overlay: HTMLDivElement | null = null;
  let img: HTMLImageElement;
  let stage: HTMLDivElement;
  let caption: HTMLDivElement;
  let list: HTMLImageElement[] = [];
  let index = 0;
  let view: View = RESET;
  let opener: HTMLElement | null = null;

  const render = () => {
    img.style.transform = `translate(${view.x}px, ${view.y}px) scale(${view.scale})`;
    stage.classList.toggle("zoomed", view.scale > 1);
  };
  const setView = (v: View) => {
    const r = stage.getBoundingClientRect();
    view = limitPan(v, r.width, r.height, img.clientWidth, img.clientHeight);
    render();
  };
  const zoomCentre = (f: number) => setView(zoomAt(view, f, 0, 0));

  const show = (i: number) => {
    index = (i + list.length) % list.length;
    const src = list[index];
    img.src = src.currentSrc || src.src;
    img.alt = src.alt;
    const text = src.alt || src.closest("figure")?.querySelector("figcaption")?.textContent || "";
    caption.textContent = text ? `${text} (${index + 1}/${list.length})` : `${index + 1}/${list.length}`;
    view = RESET;
    render();
  };

  const close = () => {
    overlay?.remove();
    overlay = null;
    document.documentElement.classList.remove("lightbox-open");
    document.removeEventListener("keydown", onKey, true);
    opener?.focus?.();
  };

  const onKey = (e: KeyboardEvent) => {
    const map: Record<string, () => void> = {
      Escape: close,
      "+": () => zoomCentre(1.4), "=": () => zoomCentre(1.4),
      "-": () => zoomCentre(1 / 1.4), "0": () => setView(RESET),
      ArrowLeft: () => list.length > 1 && show(index - 1),
      ArrowRight: () => list.length > 1 && show(index + 1),
    };
    const fn = map[e.key];
    if (!fn) return;
    e.preventDefault();
    e.stopPropagation();
    fn();
  };

  function open(target: HTMLImageElement): void {
    list = images();
    opener = target;
    overlay = document.createElement("div");
    overlay.className = "lightbox";
    overlay.setAttribute("role", "dialog");
    overlay.setAttribute("aria-modal", "true");
    stage = document.createElement("div");
    stage.className = "lightbox-stage";
    img = document.createElement("img");
    img.className = "lightbox-img";
    img.draggable = false;
    stage.append(img);
    caption = document.createElement("div");
    caption.className = "lightbox-caption";

    const bar = document.createElement("div");
    bar.className = "lightbox-bar";
    const close_ = button("close", "close", "✕");
    const plus = button("", "zoomIn", "+");
    const minus = button("", "zoomOut", "−");
    const reset = button("", "reset", "1:1");
    bar.append(minus, reset, plus, close_);
    overlay.append(stage, bar, caption);
    if (list.length > 1) {
      const prev = button("prev", "prev", "‹");
      const next = button("next", "next", "›");
      prev.onclick = () => show(index - 1);
      next.onclick = () => show(index + 1);
      overlay.append(prev, next);
    }
    close_.onclick = close;
    plus.onclick = () => zoomCentre(1.4);
    minus.onclick = () => zoomCentre(1 / 1.4);
    reset.onclick = () => setView(RESET);

    // Click or tap on the image toggles zoom at that point; click on the empty backdrop closes.
    stage.addEventListener("click", (e) => {
      if (e.target !== img) { if (view.scale === 1) close(); return; }
      const r = stage.getBoundingClientRect();
      const px = e.clientX - r.left - r.width / 2;
      const py = e.clientY - r.top - r.height / 2;
      setView(view.scale > 1 ? RESET : zoomAt(view, 2.5, px, py));
    });
    stage.addEventListener("wheel", (e) => {
      e.preventDefault();
      const r = stage.getBoundingClientRect();
      setView(zoomAt(view, e.deltaY < 0 ? 1.15 : 1 / 1.15, e.clientX - r.left - r.width / 2, e.clientY - r.top - r.height / 2));
    }, { passive: false });

    // Pointer drag (pan) and two-finger pinch.
    const pointers = new Map<number, { x: number; y: number }>();
    let pinch = 0;
    let moved = false;
    stage.addEventListener("pointerdown", (e) => {
      stage.setPointerCapture(e.pointerId);
      pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
      moved = false;
      pinch = pointers.size === 2 ? dist(pointers) : 0;
    });
    stage.addEventListener("pointermove", (e) => {
      const p = pointers.get(e.pointerId);
      if (!p) return;
      const dx = e.clientX - p.x;
      const dy = e.clientY - p.y;
      p.x = e.clientX;
      p.y = e.clientY;
      if (pointers.size === 2) {
        const d = dist(pointers);
        if (pinch) {
          const r = stage.getBoundingClientRect();
          const [a, b] = [...pointers.values()];
          setView(zoomAt(view, d / pinch, (a.x + b.x) / 2 - r.left - r.width / 2, (a.y + b.y) / 2 - r.top - r.height / 2));
        }
        pinch = d;
        moved = true;
      } else if (view.scale > 1) {
        moved = true;
        setView({ ...view, x: view.x + dx, y: view.y + dy });
      }
    });
    const up = (e: PointerEvent) => { pointers.delete(e.pointerId); pinch = 0; };
    stage.addEventListener("pointerup", up);
    stage.addEventListener("pointercancel", up);
    stage.addEventListener("click", (e) => { if (moved) e.stopPropagation(); }, true);

    document.body.append(overlay);
    document.documentElement.classList.add("lightbox-open");
    document.addEventListener("keydown", onKey, true);
    show(list.indexOf(target));
    close_.focus();
  }

  document.addEventListener("click", (e) => {
    const t = e.target;
    if (!(t instanceof HTMLImageElement) || !t.matches(SELECTOR) || overlay) return;
    if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey) return;
    e.preventDefault();
    open(t);
  });
}

function dist(p: Map<number, { x: number; y: number }>): number {
  const [a, b] = [...p.values()];
  return Math.hypot(a.x - b.x, a.y - b.y);
}
