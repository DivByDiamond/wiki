"""Attach reviewed photo blocks without changing article prose or source footers."""
import html
import re

MARKERS = ('lead', 'gallery')


def attach(body, lang, slug, data):
    plan = data.get('pages', {}).get(slug)
    if not plan:
        return body
    for marker in MARKERS:
        body = re.sub(r'\n*<!-- wiki-photos:' + marker + r' -->.*?<!-- /wiki-photos:' + marker + r' -->\n*', '\n\n', body, flags=re.S).strip() + '\n'

    def figure(name, eager=False):
        photo = data['photos'][name]
        caption = photo['caption'].get(lang, photo['caption']['en'])
        caption += f" (Minecraft {data['capture']['minecraft']})"
        url = '/static/photos/' + photo['file']
        image = (f'<a href="{url}"><img src="{url}" alt="{html.escape(caption, quote=True)}" '
                 f'width="{photo["width"]}" height="{photo["height"]}" '
                 f'loading="{"eager" if eager else "lazy"}" decoding="async"></a>')
        return image + '\n\n*' + caption + '*'

    def block(kind, content):
        return f'<!-- wiki-photos:{kind} -->\n{content}\n<!-- /wiki-photos:{kind} -->'

    if plan.get('lead'):
        parts = body.strip().split('\n\n')
        position = next((i + 1 for i, part in enumerate(parts)
                         if not part.startswith(('#', '![')) and not re.fullmatch(r'`oc2r:[^`]+`', part)), 1)
        parts.insert(position, block('lead', figure(plan['lead'], eager=True)))
        body = '\n\n'.join(parts) + '\n'
    if plan.get('gallery'):
        heading = {'en': 'In game', 'ru': 'В игре', 'zh': '游戏画面'}.get(lang, 'In game')
        note = {'en': 'Actual Minecraft 26.1.2 screenshots from the OC2R demonstration world.',
                'ru': 'Реальные скриншоты Minecraft 26.1.2 из демонстрационного мира OC2R.',
                'zh': '来自 OC2R 演示世界的真实 Minecraft 26.1.2 截图。'}.get(lang)
        content = f'## {heading}\n\n{note}\n\n' + '\n\n'.join(figure(name) for name in plan['gallery'])
        gallery = block('gallery', content)
        # Generated source provenance remains the last block, not part of a gallery.
        source = re.search(r'\n---\n\n\[[^\n]+\]\(https://github\.com/DivByDiamond/OC2R/blob/[^\n]+\n?$', body)
        if source:
            body = body[:source.start()].rstrip() + '\n\n' + gallery + '\n' + source.group(0)
        else:
            body = body.rstrip() + '\n\n' + gallery + '\n'
    return body
