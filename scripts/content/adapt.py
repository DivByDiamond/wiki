"""Convert the in-game manual's URLs to website routes without changing code blocks."""
import posixpath
import re

LOCALES = {"en_us": "en", "ru_ru": "ru", "zh_cn": "zh"}
SECTIONS = ["start", "hardware", "networking", "robotics", "systems", "scripting", "project"]


def slug_for(path):
    return "start" if path == "index.md" else path.removesuffix(".md")


def section_for(slug):
    if slug.startswith("systems/"):
        return "systems"
    if slug.startswith("reference/") or slug == "scripting":
        return "scripting"
    if slug.startswith("project/"):
        return "project"
    if "network" in slug or "internet" in slug:
        return "networking"
    if slug == "robotics" or slug == "item/robot" or slug.endswith("_module"):
        return "robotics"
    if slug.startswith(("block/", "item/")):
        return "hardware"
    return "start"


def convert(body, relative, lang, known):
    def link(match):
        image, label, dest = match.groups()
        if dest.startswith(("item:", "block:")):
            return f"`{dest.split(':', 1)[1]}`"
        if re.match(r"[a-z]+:", dest) or dest.startswith(("#", "/")):
            return match.group(0)
        path, sep, anchor = dest.partition("#")
        target = posixpath.normpath(posixpath.join(posixpath.dirname(relative), path))
        if target.startswith("../img/") or target.startswith("img/"):
            return f"![{label}](/static/manual/{posixpath.basename(target)})"
        if path.endswith(".md"):
            slug = slug_for(target)
            if (lang, slug) not in known and ("en", slug) not in known:
                raise ValueError(f"Missing manual target: {lang}/{relative}: {dest}")
            return f"[{label}](/{lang}/{slug}{sep}{anchor})"
        raise ValueError(f"Unsupported manual URL: {relative}: {dest}")

    out, fence = [], None
    for line in body.splitlines():
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker:
            run = marker.group(1)
            if fence is None:
                fence = run
            elif run[0] == fence[0] and len(run) >= len(fence):
                fence = None
            out.append(line)
        elif fence:
            out.append(line)
        else:
            out.append(re.sub(r"(!?)\[([^\]]*)\]\(([^)]+)\)", link, line))
    return "\n".join(out).strip() + "\n"
