export function isTextTarget(t: EventTarget | null): boolean {
  if (!(t instanceof HTMLElement)) return false;
  return t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement
    || t instanceof HTMLSelectElement || t.isContentEditable;
}

export function onSlash(handler: () => void): void {
  document.addEventListener("keydown", (e) => {
    if (e.key !== "/" || e.ctrlKey || e.metaKey || e.altKey || isTextTarget(e.target)) return;
    e.preventDefault();
    handler();
  });
}

export function onEscape(handler: () => void): void {
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") handler();
  });
}
