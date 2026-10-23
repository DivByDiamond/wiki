/** Native details retain keyboard and no-JavaScript support. */
export function mountHeader(): void {
  const menus = Array.from(document.querySelectorAll<HTMLDetailsElement>("[data-header-menu]"));
  const close = (except?: HTMLDetailsElement) => {
    for (const menu of menus) if (menu !== except) menu.open = false;
  };
  for (const menu of menus) {
    menu.addEventListener("toggle", () => { if (menu.open) close(menu); });
  }
  document.addEventListener("click", (event) => {
    if (!(event.target instanceof Node)) return;
    const containing = menus.find((menu) => menu.contains(event.target as Node));
    close(containing);
  });
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    const active = menus.find((menu) => menu.open);
    if (!active) return;
    close();
    active.querySelector<HTMLElement>("summary")?.focus();
  });
}
