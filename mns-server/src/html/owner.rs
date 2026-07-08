use super::{
    footer_html, main_style, navbar_html, page_head, particles_script, truncate_addr, Name, Navbar,
    OwnerItemSimple,
};

pub fn render_owner_page(address: &str, names: &[Name], nav: &Navbar) -> String {
    let style = format!(
        r##"{main_style}

  .owner-address {{
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-family: var(--mono);
    font-size: 0.82rem;
    color: var(--accent-text);
    word-break: break-all;
    text-align: center;
    margin-bottom: 1.5rem;
  }}

  .copy-btn {{
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-xs);
    cursor: pointer;
    color: var(--fg-muted);
    padding: 2px 4px;
    font-family: var(--mono);
    font-size: 0.7rem;
    transition: color 0.15s, border-color 0.15s, background 0.15s;
    white-space: nowrap;
  }}
  .copy-btn:hover {{
    color: var(--fg);
    border-color: var(--fg-muted);
    background: var(--surface-hover);
  }}
  .copy-btn.copied {{
    color: var(--accent);
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 10%, transparent);
  }}

  .owner-grid {{
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(115px, 1fr));
    list-style: none;
  }}

  .owner-item {{
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.5rem 0.5rem;
    border-radius: var(--radius-xs);
    text-decoration: none;
    color: var(--fg);
    transition: background 0.2s;
  }}

  .owner-item:hover {{
    background: var(--surface-hover);
  }}

  .owner-avatar {{
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    border-radius: 4px;
    overflow: hidden;
    color: var(--fg);
  }}
  .owner-avatar svg {{
    width: 100%;
    height: 100%;
    display: block;
  }}

  .owner-name {{
    font-family: var(--mono);
    font-size: 0.82rem;
    font-weight: 500;
  }}

  .empty-section {{
    text-align: center;
    padding: 1rem;
    color: var(--fg-muted);
    font-size: 0.82rem;
  }}

  @media (max-width: 480px) {{
    .owner-grid {{
      grid-template-columns: 1fr;
    }}
  }}
"##,
        main_style = main_style()
    );
    let head = page_head("Owner — MNS", &style);
    let particles = particles_script();
    let footer = footer_html();
    let nav_html = navbar_html(nav);
    let rows: String = if names.is_empty() {
        r#"<p class="empty-section">No names owned.</p>"#.into()
    } else {
        let items: String = names
            .iter()
            .map(|name| {
                let svg = name.render_avatar_svg();
                let name_str = name.to_string();
                format!(
                    r#"<li><a class="owner-item" href="/{href}">
                <span class="owner-avatar">{svg}</span>
                <span class="owner-name">{name_str}</span>
              </a></li>"#,
                    href = name_str,
                    name_str = name_str,
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!(r#"<ul class="owner-grid">{items}</ul>"#)
    };

    format!(
        r#"{head}
<body>

{nav_html}

<div class="grid-bg" aria-hidden="true"></div>
<div class="particles" id="particles" aria-hidden="true"></div>

<main>
<section class="card">
  <header class="header">
    <h1>Owner</h1>
  </header>
  <p class="owner-address">
    <span title="{address}">{truncated}</span>
    <button class="copy-btn" id="copy-addr" data-addr="{address}" aria-label="Copy address">
      <svg class="copy-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
      </svg>
      <span class="copy-label">Copy</span>
    </button>
  </p>

  <div class="divider" role="separator"></div>

  {rows}
</section>

<nav class="back-wrap" aria-label="Breadcrumb">
  <a class="back-link" href="/">← Home</a>
</nav>
</main>

{footer}

{particles}
<script>
document.getElementById('copy-addr')?.addEventListener('click', function() {{
  var addr = this.getAttribute('data-addr');
  navigator.clipboard.writeText(addr).then(function() {{
    this.classList.add('copied');
    var orig = this.innerHTML;
    this.innerHTML =
      '<svg class="copy-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>' +
      '<span class="copy-label">Copied!</span>';
    var that = this;
    setTimeout(function() {{ that.innerHTML = orig; that.classList.remove('copied'); }}, 1500);
  }}.bind(this)).catch(function() {{}});
}});
</script>
</body>
</html>"#,
        address = address,
        truncated = truncate_addr(address),
    )
}

pub fn render_owners_page(items: &[OwnerItemSimple], nav: &Navbar) -> String {
    let style = format!(
        r##"{main_style}

  .owner-list {{
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    list-style: none;
  }}

  .owner-row {{
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-sm);
    text-decoration: none;
    color: var(--fg);
    font-family: var(--mono);
    font-size: 0.82rem;
    transition: background 0.2s;
  }}

  .owner-row:hover {{
    background: var(--surface-hover);
  }}

  .empty-section {{
    text-align: center;
    padding: 1rem;
    color: var(--fg-muted);
    font-size: 0.82rem;
  }}
"##,
        main_style = main_style()
    );
    let head = page_head("Owners — MNS", &style);
    let particles = particles_script();
    let footer = footer_html();
    let nav_html = navbar_html(nav);
    let rows: String = if items.is_empty() {
        r#"<p class="empty-section">No owners.</p>"#.into()
    } else {
        let items_html: String = items
            .iter()
            .map(|item| {
                format!(
                    r#"<li><a class="owner-row" href="/owner/{addr}" title="{addr}">{truncated}</a></li>"#,
                    addr = item.name_or_addr,
                    truncated = truncate_addr(&item.name_or_addr)
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!(r#"<ul class="owner-list">{items_html}</ul>"#)
    };

    format!(
        r#"{head}
<body>

{nav_html}

<div class="grid-bg" aria-hidden="true"></div>
<div class="particles" id="particles" aria-hidden="true"></div>

<main>
<section class="card">
  <header class="header">
    <h1>Owners</h1>
  </header>

  <div class="divider" role="separator"></div>

  {rows}
</section>

<nav class="back-wrap" aria-label="Breadcrumb">
  <a class="back-link" href="/">← Home</a>
</nav>
</main>

{footer}

{particles}
</body>
</html>"#,
    )
}
