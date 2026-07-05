use mns::Name;
use mns::ZSK_LEN;
use simple_dns::ResourceRecord;

const FAVICON: &str = "/static/favicon.png";
const ACCENT: &str = "#800000";

fn main_style() -> String {
    format!(
        r##"
  *, *::before, *::after {{ margin: 0; padding: 0; box-sizing: border-box; }}

  :root {{
    --bg: #08090d;
    --surface: rgba(255,255,255,0.03);
    --surface-hover: rgba(255,255,255,0.055);
    --border: rgba(255,255,255,0.06);
    --border-focus: rgba(128,0,0,0.3);
    --fg: #e4e8ef;
    --fg-muted: #6b7280;
    --fg-dim: #3a3f4b;
    --accent: {accent};
    --accent-dim: rgba(128,0,0,0.12);
    --accent-glow: rgba(128,0,0,0.15);
    --accent-text: #b05858;
    --mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;
    --sans: 'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
    --radius: 16px;
    --radius-sm: 10px;
    --navbar-bg: rgba(8,9,13,0.85);
    --grid-color: rgba(255,255,255,0.015);
    --glow-1: rgba(128,0,0,0.06);
    --glow-2: rgba(128,0,0,0.04);
  }}

  html[data-theme="light"] {{
    --bg: #f5f5f7;
    --surface: #ffffff;
    --surface-hover: rgba(0,0,0,0.04);
    --border: rgba(0,0,0,0.08);
    --border-focus: rgba(128,0,0,0.25);
    --fg: #1a1a2e;
    --fg-muted: #6b7280;
    --fg-dim: #c0c4cc;
    --accent: {accent};
    --accent-dim: rgba(128,0,0,0.07);
    --accent-glow: rgba(128,0,0,0.08);
    --accent-text: #a04545;
    --navbar-bg: rgba(245,245,247,0.85);
    --grid-color: rgba(0,0,0,0.04);
    --glow-1: rgba(128,0,0,0.03);
    --glow-2: rgba(128,0,0,0.02);
  }}

  body {{
    font-family: var(--sans);
    background: var(--bg);
    color: var(--fg);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 5rem 1.25rem 2rem;
    overflow-x: hidden;
    position: relative;
  }}

  body::before {{
    content: '';
    position: fixed;
    top: -40%;
    left: 50%;
    transform: translateX(-50%);
    width: 700px;
    height: 700px;
    background: radial-gradient(circle, var(--glow-1) 0%, transparent 70%);
    pointer-events: none;
    z-index: 0;
    animation: breathe 8s ease-in-out infinite;
  }}

  body::after {{
    content: '';
    position: fixed;
    bottom: -30%;
    left: 30%;
    width: 500px;
    height: 500px;
    background: radial-gradient(circle, var(--glow-2) 0%, transparent 70%);
    pointer-events: none;
    z-index: 0;
    animation: breathe 10s ease-in-out 3s infinite reverse;
  }}

  @keyframes breathe {{
    0%, 100% {{ opacity: 0.6; transform: translateX(-50%) scale(1); }}
    50% {{ opacity: 1; transform: translateX(-50%) scale(1.1); }}
  }}

  .grid-bg {{
    position: fixed;
    inset: 0;
    background-image:
      linear-gradient(var(--grid-color) 1px, transparent 1px),
      linear-gradient(90deg, var(--grid-color) 1px, transparent 1px);
    background-size: 60px 60px;
    pointer-events: none;
    z-index: 0;
    mask-image: radial-gradient(ellipse 60% 50% at 50% 30%, black 20%, transparent 100%);
    -webkit-mask-image: radial-gradient(ellipse 60% 50% at 50% 30%, black 20%, transparent 100%);
  }}

  .card {{
    position: relative;
    z-index: 1;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    max-width: 560px;
    width: 100%;
    padding: 2.5rem 2.25rem 2rem;
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    box-shadow:
      0 0 0 1px rgba(255,255,255,0.02) inset,
      0 20px 60px -15px rgba(0,0,0,0.5),
      0 0 80px -20px var(--accent-glow);
    animation: cardIn 0.7s cubic-bezier(0.16, 1, 0.3, 1) both;
  }}

  @keyframes cardIn {{
    from {{ opacity: 0; transform: translateY(24px) scale(0.97); }}
    to {{ opacity: 1; transform: translateY(0) scale(1); }}
  }}

  .card::before {{
    content: '';
    position: absolute;
    top: 0; left: 2rem; right: 2rem;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--accent), transparent);
    opacity: 0.4;
    border-radius: 1px;
  }}

  .records-card {{
    margin-top: 1rem;
  }}

  .records-card h2 {{
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: 1rem;
  }}

  .avatar-wrap {{
    display: flex;
    justify-content: center;
    margin-bottom: 1.75rem;
    animation: avatarIn 0.8s cubic-bezier(0.16, 1, 0.3, 1) 0s both;
  }}

  @keyframes avatarIn {{
    from {{ opacity: 0; transform: scale(0.85) rotate(-3deg); }}
    to {{ opacity: 1; transform: scale(1) rotate(0deg); }}
  }}

  .avatar {{
    width: 112px;
    height: 112px;
    color: var(--accent-text);
    background: var(--accent);
    border: 1px solid rgba(128,0,0,0.15);
    border-radius: var(--radius-sm);
    padding: .25rem;
    position: relative;
    transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.4s ease;
  }}

  .avatar:hover {{
    transform: scale(1.05) rotate(1deg);
    box-shadow: 0 0 30px -5px var(--accent-glow);
  }}

  .mns-avatar {{
    width: 100%;
    height: 100%;
    display: block;
    color: var(--fg);
  }}

  .header {{
    text-align: center;
    margin-bottom: 2rem;
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0s both;
  }}

  @keyframes fadeUp {{
    from {{ opacity: 0; transform: translateY(12px); }}
    to {{ opacity: 1; transform: translateY(0); }}
  }}

  h1 {{
    font-size: 1.6rem;
    font-weight: 600;
    letter-spacing: -0.02em;
    line-height: 1.2;
    color: var(--fg);
    margin-bottom: 0.4rem;
  }}

  .canonical {{
    font-family: var(--mono);
    color: var(--accent-text);
    font-size: 0.8rem;
    font-weight: 500;
    opacity: 0.7;
    letter-spacing: 0.01em;
    word-break: break-all;
  }}

  .divider {{
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--border), transparent);
    margin-bottom: 1.5rem;
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0s both;
  }}

  .meta-grid {{
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 1.75rem;
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0s both;
  }}

  .meta-row {{
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.85rem 1rem;
    background: rgba(255,255,255,0.015);
    border: 1px solid rgba(255,255,255,0.04);
    border-radius: var(--radius-sm);
    transition: background 0.25s ease, border-color 0.25s ease;
  }}

  .meta-row:hover {{
    background: var(--surface-hover);
    border-color: rgba(255,255,255,0.07);
  }}

  .meta-row.inline {{
    flex-direction: row;
    align-items: center;
    gap: 1rem;
  }}

  .meta-row.inline .meta-value {{
    text-align: left;
  }}

  .meta-label {{
    color: var(--fg-muted);
    font-size: 0.7rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }}

  .meta-value {{
    font-family: var(--mono);
    font-size: 0.82rem;
    color: var(--fg);
    word-break: break-all;
    line-height: 1.5;
    opacity: 0.85;
  }}

  .meta-value.dim {{
    color: var(--fg-dim);
    font-size: 0.78rem;
  }}

  table {{
    width: 100%;
    border-collapse: collapse;
    font-family: var(--mono);
    font-size: 0.76rem;
  }}

  th {{
    text-align: left;
    color: var(--fg-muted);
    font-size: 0.67rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    padding: 0.5rem 0.5rem 0.5rem 0;
    border-bottom: 1px solid var(--border);
    font-weight: 600;
  }}

  td {{
    padding: 0.5rem 0.5rem 0.5rem 0;
    border-bottom: 1px solid rgba(255,255,255,0.03);
    vertical-align: top;
  }}

  tr:last-child td {{
    border-bottom: none;
  }}

  td.name {{ color: var(--accent-text); }}
  td.type {{ color: #d4a060; }}
  td.rdata {{ color: var(--fg-muted); word-break: break-all; }}
  td.ttl {{ color: var(--fg-dim); }}

  .empty-state {{
    text-align: center;
    padding: 1.75rem 1rem;
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0s both;
  }}

  .empty-text {{
    color: var(--fg-muted);
    font-size: 0.88rem;
    font-weight: 400;
    line-height: 1.5;
  }}

  .footer {{
    position: relative;
    z-index: 1;
    color: var(--fg-dim);
    font-size: 0.72rem;
    font-weight: 500;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    margin-top: 2.5rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0s both;
  }}

  .footer-dot {{
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
    opacity: 0.5;
    animation: pulse-dot 3s ease-in-out infinite;
  }}

  @keyframes pulse-dot {{
    0%, 100% {{ opacity: 0.3; transform: scale(0.9); }}
    50% {{ opacity: 0.7; transform: scale(1.1); }}
  }}

  .particles {{
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 0;
    overflow: hidden;
  }}

  .particle {{
    position: absolute;
    width: 2px;
    height: 2px;
    background: var(--accent);
    border-radius: 50%;
    opacity: 0;
    animation: float-particle linear infinite;
  }}

  @keyframes float-particle {{
    0% {{ opacity: 0; transform: translateY(100vh) scale(0); }}
    10% {{ opacity: 0.4; }}
    90% {{ opacity: 0.4; }}
    100% {{ opacity: 0; transform: translateY(-10vh) scale(1); }}
  }}

  @media (max-width: 480px) {{
    body {{ padding: 2rem 1rem 1.5rem; }}
    .card {{ padding: 2rem 1.5rem 1.5rem; }}
    h1 {{ font-size: 1.35rem; }}
    .avatar {{ width: 96px; height: 96px; padding: 8px; }}
    .meta-row.inline {{ flex-direction: column; align-items: flex-start; gap: 0.35rem; }}
    .meta-row.inline .meta-value {{ text-align: left; }}
  }}

  @media (prefers-reduced-motion: reduce) {{
    *, *::before, *::after {{
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
    }}
  }}

  {navbar_style}
"##,
        accent = ACCENT,
        navbar_style = navbar_style()
    )
}

fn error_style() -> String {
    format!(
        r##"
  *, *::before, *::after {{ margin: 0; padding: 0; box-sizing: border-box; }}

  :root {{
    --bg: #08090d;
    --surface: rgba(255,255,255,0.03);
    --border: rgba(255,255,255,0.06);
    --fg: #e4e8ef;
    --accent: {accent};
    --accent-glow: rgba(128,0,0,0.15);
    --sans: 'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
    --radius: 16px;
    --navbar-bg: rgba(8,9,13,0.85);
    --glow-1: rgba(128,0,0,0.06);
  }}

  html[data-theme="light"] {{
    --bg: #f5f5f7;
    --surface: #ffffff;
    --border: rgba(0,0,0,0.08);
    --fg: #1a1a2e;
    --accent: {accent};
    --accent-glow: rgba(128,0,0,0.08);
    --sans: 'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
    --radius: 16px;
    --navbar-bg: rgba(245,245,247,0.85);
    --glow-1: rgba(128,0,0,0.03);
  }}

  body {{
    font-family: var(--sans);
    background: var(--bg);
    color: var(--fg);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 5rem 2rem 2rem;
  }}

  body::before {{
    content: '';
    position: fixed;
    inset: 0;
    background: radial-gradient(circle at 50% 30%, var(--glow-1) 0%, transparent 60%);
    pointer-events: none;
  }}

  .card {{
    position: relative;
    z-index: 1;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    max-width: 420px;
    width: 100%;
    padding: 2rem 2rem;
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    text-align: center;
    box-shadow: 0 0 80px -20px var(--accent-glow);
  }}

  .card::before {{
    content: '';
    position: absolute;
    top: 0; left: 2rem; right: 2rem;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--accent), transparent);
    opacity: 0.4;
  }}

  .msg {{
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--fg);
  }}

  {navbar_style}
"##,
        accent = ACCENT,
        navbar_style = navbar_style()
    )
}

fn page_head(title: &str, style: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title} — MNS</title>
<link rel="icon" href="{FAVICON}">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
<script>(function(){{var t=localStorage.getItem('mns-theme');if(!t){{t=window.matchMedia('(prefers-color-scheme:light)').matches?'light':'dark'}}document.documentElement.setAttribute('data-theme',t);window._toggleTheme=function(){{var c=document.documentElement.getAttribute('data-theme');var n=c==='light'?'dark':'light';document.documentElement.setAttribute('data-theme',n);localStorage.setItem('mns-theme',n)}}}})()</script>
<style>{style}</style>
</head>"#,
    )
}

fn particles_script() -> String {
    r#"<script>
(function() {
  const container = document.getElementById('particles');
  const count = 12;
  for (let i = 0; i < count; i++) {
    const p = document.createElement('div');
    p.className = 'particle';
    const left = Math.random() * 100;
    const size = 1 + Math.random() * 2;
    const duration = 12 + Math.random() * 18;
    const delay = Math.random() * duration;
    p.style.cssText = `
      left: ${left}%;
      width: ${size}px;
      height: ${size}px;
      animation-duration: ${duration}s;
      animation-delay: -${delay}s;
    `;
    container.appendChild(p);
  }
})();
</script>"#
        .to_string()
}

fn footer_html() -> String {
    r#"<div class="footer">
  <span class="footer-dot"></span>
  MNS Resolver
</div>"#
        .to_string()
}

pub struct Navbar {
    pub sync_block: u64,
    pub network: String,
    pub explorer_url: String,
    pub contract_address: String,
}

fn navbar_html(nav: &Navbar) -> String {
    let block_url = format!(
        "{}/block/{}",
        nav.explorer_url.trim_end_matches('/'),
        nav.sync_block
    );
    let contract_url = format!(
        "{}/address/{}?tab=contract",
        nav.explorer_url.trim_end_matches('/'),
        nav.contract_address
    );
    format!(
        r#"<nav class="navbar">
  <a href="/" class="navbar-logo">
    <img src="/static/mlkut.png" alt="MNS">
  </a>
  <div class="navbar-right">
    <a class="navbar-network" href="{contract_url}" target="_blank" title="View contract on explorer">{network}</a>
    <a class="navbar-block" href="{block_url}" target="_blank" title="view latest indexed block {block} on explorer">
      <span class="liveness-dot"></span>
      <span class="block-number">{block}</span>
    </a>
    <button class="theme-toggle" onclick="_toggleTheme()" aria-label="Switch theme">
      <svg class="theme-icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="5"/>
        <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
      </svg>
      <svg class="theme-icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
      </svg>
    </button>
  </div>
</nav>"#,
        network = nav.network,
        block = nav.sync_block,
        block_url = block_url,
        contract_url = contract_url,
    )
}

fn navbar_style() -> String {
    r##"
  .navbar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.25rem;
    background: var(--navbar-bg);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--border);
    z-index: 100;
  }

  .navbar-logo img {
    height: 26px;
    width: auto;
    display: block;
    opacity: 0.85;
    transition: opacity 0.2s;
  }
  .navbar-logo:hover img { opacity: 1; }

  .navbar-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .navbar-network {
    font-family: var(--mono);
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-muted);
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    text-decoration: none;
    transition: color 0.2s, border-color 0.2s;
  }
  .navbar-network:hover {
    color: var(--fg);
    border-color: var(--accent);
  }

  .navbar-block {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-family: var(--mono);
    font-size: 0.75rem;
    color: var(--fg);
    text-decoration: none;
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    transition: border-color 0.2s, background 0.2s;
  }
  .navbar-block:hover {
    border-color: var(--accent);
    background: var(--surface-hover);
  }

  .liveness-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #22c55e;
    animation: pulse-dot 2s ease-in-out infinite;
  }

  @keyframes pulse-dot {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .block-number {
    font-variant-numeric: tabular-nums;
  }

  .theme-toggle {
    background: none;
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: pointer;
    padding: 5px;
    line-height: 0;
    color: var(--fg-muted);
    transition: color 0.2s, border-color 0.2s;
  }
  .theme-toggle:hover {
    color: var(--fg);
    border-color: var(--accent);
  }
  .theme-toggle svg {
    width: 15px;
    height: 15px;
    display: block;
  }
  html[data-theme="dark"] .theme-icon-sun,
  html[data-theme="light"] .theme-icon-moon {
    display: block;
  }
  html[data-theme="dark"] .theme-icon-moon,
  html[data-theme="light"] .theme-icon-sun {
    display: none;
  }
"##
    .to_string()
}

pub fn render_html(
    name: &Name,
    owner: Option<&[u8; 20]>,
    zsk: &[u8; ZSK_LEN],
    ns: &str,
    records: &[ResourceRecord],
    timestamp: u64,
    nav: &Navbar,
) -> String {
    let name_str = name.to_string();
    let canonical = name.canonical_domain();
    let avatar_svg = name.render_avatar_svg();
    let zsk_hex = hex::encode(zsk);

    let mut rows = String::new();
    for r in records {
        let rtype = format!("{:?}", r.rdata.type_code());
        let rdata_str = format!("{:?}", r.rdata);
        rows.push_str(&format!(
            r#"<tr><td class="name">{}</td><td class="type">{}</td><td class="ttl">{}</td><td class="rdata">{}</td></tr>"#,
            r.name,
            rtype,
            r.ttl,
            rdata_str
        ));
    }

    let ts = if timestamp > 0 {
        format_timestamp(timestamp)
    } else {
        "--".into()
    };

    let owner_row = if let Some(owner) = owner {
        let owner_hex = hex::encode(owner);
        format!(
            r#"<div class="meta-row inline">
      <div class="meta-label">Owner</div>
      <div class="meta-value dim">
        <a href="/owner/0x{owner_hex}" style="color:var(--accent-text);text-decoration:none;">0x{owner_hex}</a>
      </div>
    </div>"#,
        )
    } else {
        String::new()
    };

    let records_section = if records.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div class="card records-card">
  <h2>Resource Records</h2>
  <table>
    <thead><tr><th>Name</th><th>Type</th><th>TTL</th><th>Data</th></tr></thead>
    <tbody>{rows}</tbody>
  </table>
</div>"#,
        )
    };

    let style = main_style();
    let head = page_head(&name_str, &style);
    let particles = particles_script();
    let footer = footer_html();
    let nav_html = navbar_html(nav);

    format!(
        r#"{head}
<body>

{nav_html}

<div class="grid-bg"></div>
<div class="particles" id="particles"></div>

<div class="card">
  <div class="avatar-wrap">
    <div class="avatar">{avatar_svg}</div>
  </div>

  <div class="header">
    <h1>{name_str}</h1>
    <div class="canonical">{canonical}</div>
  </div>

  <div class="divider"></div>

  <div class="meta-grid">
    {owner_row}
    <div class="meta-row inline">
      <div class="meta-label">ZSK</div>
      <div class="meta-value dim">0x{zsk_hex}</div>
    </div>
    <div class="meta-row inline">
      <div class="meta-label">NS</div>
      <div class="meta-value dim">{ns}</div>
    </div>
    <div class="meta-row inline">
      <div class="meta-label">Updated</div>
      <div class="meta-value dim">{ts}</div>
    </div>
  </div>

  <div class="empty-state">
    <p class="empty-text">No signed packets published yet.</p>
  </div>
</div>

{records_section}

<div style="text-align:center;margin-top:0.75rem;z-index:1;position:relative;">
  <a href="/" style="color:var(--fg-muted);font-size:0.75rem;text-decoration:none;">← Home</a>
</div>

{footer}

{particles}
</body>
</html>"#,
    )
}

pub fn render_not_found_page(
    name: &Name,
    owner: Option<&[u8; 20]>,
    zsk: Option<&[u8; ZSK_LEN]>,
    ns: Option<&str>,
    nav: &Navbar,
) -> String {
    let name_str = name.to_string();
    let canonical = name.canonical_domain();
    let avatar_svg = name.render_avatar_svg();

    let owner_row = if let Some(owner) = owner {
        let owner_hex = hex::encode(owner);
        format!(
            r#"<div class="meta-row inline">
      <div class="meta-label">Owner</div>
      <div class="meta-value dim">
        <a href="/owner/0x{owner_hex}" style="color:var(--accent-text);text-decoration:none;">0x{owner_hex}</a>
      </div>
    </div>"#,
        )
    } else {
        String::new()
    };

    let (meta_rows, empty_text) = if let Some(zsk) = zsk {
        let zsk_hex = hex::encode(zsk);
        let ns_val = ns.unwrap_or("");
        let ns_row = if !ns_val.is_empty() {
            format!(
                r#"<div class="meta-row inline">
      <div class="meta-label">NS</div>
      <div class="meta-value dim">{ns_val}</div>
    </div>"#,
            )
        } else {
            String::new()
        };
        (
            format!(
                r#"<div class="meta-row inline">
      <div class="meta-label">ZSK</div>
      <div class="meta-value dim">0x{zsk_hex}</div>
    </div>
    {ns_row}"#,
            ),
            "No signed packets published yet.",
        )
    } else {
        (String::new(), "This name is not registered on MNS yet!.")
    };

    let style = main_style();
    let head = page_head(&name_str, &style);
    let particles = particles_script();
    let footer = footer_html();
    let nav_html = navbar_html(nav);

    format!(
        r#"{head}
<body>

{nav_html}
<div class="grid-bg"></div>
<div class="particles" id="particles"></div>

<div class="card">
  <div class="avatar-wrap">
    <div class="avatar">{avatar_svg}</div>
  </div>

  <div class="header">
    <h1>{name_str}</h1>
    <div class="canonical">{canonical}</div>
  </div>

  <div class="divider"></div>

  <div class="meta-grid">
    {owner_row}
    {meta_rows}
  </div>

  <div class="empty-state">
    <p class="empty-text">{empty_text}</p>
  </div>
</div>

<div style="text-align:center;margin-top:0.75rem;z-index:1;position:relative;">
  <a href="/" style="color:var(--fg-muted);font-size:0.75rem;text-decoration:none;">← Home</a>
</div>

{footer}

{particles}
</body>
</html>"#,
    )
}

pub fn render_home_page(nav: &Navbar) -> String {
    let style = format!(
        r##"{main_style}

  .search-row {{
    display: flex;
    gap: 0.5rem;
    align-items: stretch;
  }}
  .search-row input {{
    flex: 1;
    min-width: 0;
  }}
  .search-row button {{
    flex-shrink: 0;
  }}

  .stats {{
    display: flex;
    gap: 0.5rem;
    justify-content: center;
    margin-top: 1rem;
    margin-bottom: 1.25rem;
  }}

  .stat {{
    padding: 0.5rem 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    text-align: center;
  }}

  .stat-value {{
    font-family: var(--mono);
    font-size: 1rem;
    font-weight: 600;
    color: var(--fg);
  }}

  .stat-label {{
    font-size: 0.65rem;
    color: var(--fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-top: 0.15rem;
  }}

  .history-list {{
    margin-top: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }}
  .history-item {{
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.4rem 0.5rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 0.2s;
    text-decoration: none;
    color: var(--fg);
  }}
  .history-item:hover {{
    background: var(--surface-hover);
  }}
  .history-avatar {{
    width: 28px;
    height: 28px;
    color: var(--accent-text);
    flex-shrink: 0;
    background: var(--fg-muted);
  }}
  .history-name {{
    font-family: var(--mono);
    font-size: 0.8rem;
  }}

  @media (max-width: 480px) {{
    .search-row {{
      flex-direction: column;
    }}
    .search-row button {{
      width: 100%;
    }}
  }}
"##,
        main_style = main_style()
    );
    let head = page_head("Mlkut Name System", &style);
    let particles = particles_script();
    let footer = footer_html();
    let nav_html = navbar_html(nav);

    format!(
        r#"{head}
<body>

{nav_html}

<div class="grid-bg"></div>
<div class="particles" id="particles"></div>

<div class="card" style="text-align:center;">
  <div class="avatar-wrap">
    <img src="/static/mlkut.png" alt="mlkut logo" class="avatar" style="width:128px;height:128px;object-fit:contain;">
  </div>

  <div class="header">
    <h1>Mlkut Name System</h1>
    <div class="canonical" style="font-family:var(--sans);opacity:1;margin-top:1rem;">
      Permissionless name registry for the next millennium.
    </div>
  </div>

  <div class="divider"></div>

  <form id="search-form" onsubmit="event.preventDefault();var q=this.querySelector('input').value.trim();if(q){{if(q.startsWith('0x')){{window.location.href='/owner/'+encodeURIComponent(q);return;}}try{{var h=JSON.parse(localStorage.getItem('mns-history')||'[]');h=h.filter(function(n){{return n!==q}});h.unshift(q);if(h.length>5)h.length=5;localStorage.setItem('mns-history',JSON.stringify(h))}}catch(e){{}}window.location.href='/'+encodeURIComponent(q);}}">
    <div class="search-row">
      <input type="text" id="search-input" placeholder="Search name or 0x address..." style="padding:0.85rem 1.1rem;background:var(--surface);border:1px solid var(--border);border-radius:var(--radius-sm);color:var(--fg);font-family:var(--mono);font-size:0.9rem;outline:none;transition:border-color 0.25s;" onfocus="this.style.borderColor='var(--accent)'" onblur="this.style.borderColor='var(--border)'">
      <button type="submit" style="padding:0.75rem 2rem;background:var(--accent);color:#fff;border:none;border-radius:var(--radius-sm);font-family:var(--sans);font-weight:600;font-size:0.9rem;cursor:pointer;transition:opacity 0.25s;" onmouseover="this.style.opacity=0.85" onmouseout="this.style.opacity=1">Search</button>
    </div>
  </form>

  <div class="stats" id="stats">
    <a href="/owners" style="text-decoration:none;color:inherit;">
    <div class="stat">
      <div class="stat-value" id="stat-owners">—</div>
      <div class="stat-label">Owners</div>
    </div>
    </a>
  </div>

  <div id="history"></div>

  <div style="margin-top:1.5rem;display:flex;gap:1rem;justify-content:center;font-size:0.82rem;">
    <a href="https://mlkut.org" style="color:var(--accent-text);text-decoration:none;font-weight:500;">Read more</a>
    <span style="color:var(--fg-dim);">·</span>
    <a href="https://mlkut.org" style="color:var(--accent-text);text-decoration:none;font-weight:500;">Specs</a>
  </div>
</div>

{footer}

{particles}

<script>
(function() {{
  fetch('/stats').then(function(r){{return r.json()}}).then(function(d){{document.getElementById('stat-owners').textContent=d.total_owners}}).catch(function(){{}});

  var list;
  try {{ list = JSON.parse(localStorage.getItem('mns-history') || '[]'); }} catch(e) {{ list = []; }}
  if (list.length === 0) return;
  var html = '<div class="history-list">';
  for (var i = 0; i < list.length; i++) {{
    var name = list[i];
    html += '<a class="history-item" href="/' + encodeURIComponent(name) + '">' +
      '<img class="history-avatar" src="/avatar/' + encodeURIComponent(name) + '" alt="" style="width:28px;height:28px;border-radius:4px;">' +
      '<span class="history-name">' + name + '</span></a>';
  }}
  html += '</div>';
  document.getElementById('history').innerHTML = html;
}})();
</script>

</body>
</html>"#,
    )
}

pub struct OwnerItemSimple {
    pub name_or_addr: String,
}

pub fn render_owner_page(address: &str, names: &[String], nav: &Navbar) -> String {
    let style = format!(
        r##"{main_style}

  .owner-address {{
    font-family: var(--mono);
    font-size: 0.82rem;
    color: var(--accent-text);
    word-break: break-all;
    text-align: center;
    margin-bottom: 1.5rem;
  }}

  .owner-grid {{
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(115px, 1fr));
    gap: 0.5rem;
  }}

  .owner-item {{
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5rem 0.65rem;
    border-radius: var(--radius-sm);
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
    background: var(--fg);
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
        r#"<div class="empty-section">No names owned.</div>"#.into()
    } else {
        names
            .iter()
            .map(|name| {
                format!(
                    r#"<a class="owner-item" href="/{name}">
                <img class="owner-avatar" src="/avatar/{name}" alt="">
                <span class="owner-name">{name}</span>
              </a>"#,
                    name = name
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r#"{head}
<body>

{nav_html}

<div class="grid-bg"></div>
<div class="particles" id="particles"></div>

<div class="card">
  <div class="header">
    <h1>Owner</h1>
  </div>
  <div class="owner-address">{address}</div>

  <div class="divider"></div>

  <div class="owner-grid">{rows}</div>
</div>

<div style="text-align:center;margin-top:0.75rem;z-index:1;position:relative;">
  <a href="/" style="color:var(--fg-muted);font-size:0.75rem;text-decoration:none;">← Home</a>
</div>

{footer}

{particles}
</body>
</html>"#,
        address = address,
    )
}

pub fn render_owners_page(items: &[OwnerItemSimple], nav: &Navbar) -> String {
    let style = format!(
        r##"{main_style}

  .owner-list {{
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }}

  .owner-row {{
    display: flex;
    align-items: center;
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
        r#"<div class="empty-section">No owners.</div>"#.into()
    } else {
        items
            .iter()
            .map(|item| {
                format!(
                    r#"<a class="owner-row" href="/owner/{addr}">{addr}</a>"#,
                    addr = item.name_or_addr
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r#"{head}
<body>

{nav_html}

<div class="grid-bg"></div>
<div class="particles" id="particles"></div>

<div class="card">
  <div class="header">
    <h1>Owners</h1>
  </div>

  <div class="divider"></div>

  <div class="owner-list">{rows}</div>
</div>

<div style="text-align:center;margin-top:0.75rem;z-index:1;position:relative;">
  <a href="/" style="color:var(--fg-muted);font-size:0.75rem;text-decoration:none;">← Home</a>
</div>

{footer}

{particles}
</body>
</html>"#,
    )
}

pub fn render_error(message: &str, nav: &Navbar) -> String {
    let style = error_style();
    let head = page_head("Error", &style);
    let nav_html = navbar_html(nav);
    format!(
        r#"{head}
<body>
{nav_html}
<div class="card"><div class="msg">{message}</div></div>
</body>
</html>"#,
    )
}

fn format_timestamp(ts: u64) -> String {
    let secs = ts as i64;
    let Ok(dt) = time::OffsetDateTime::from_unix_timestamp(secs) else {
        return ts.to_string();
    };
    let dt = dt.to_offset(time::UtcOffset::UTC);
    dt.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| ts.to_string())
}
