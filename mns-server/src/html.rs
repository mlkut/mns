use mns::Name;
use mns::ZSK_LEN;
use simple_dns::ResourceRecord;

const FAVICON: &str = "https://avatars.githubusercontent.com/u/201356953?s=64";
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
  }}

  body {{
    font-family: var(--sans);
    background: var(--bg);
    color: var(--fg);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 3rem 1.25rem 2rem;
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
    background: radial-gradient(circle, rgba(128,0,0,0.06) 0%, transparent 70%);
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
    background: radial-gradient(circle, rgba(128,0,0,0.04) 0%, transparent 70%);
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
      linear-gradient(rgba(255,255,255,0.015) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255,255,255,0.015) 1px, transparent 1px);
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
    animation: avatarIn 0.8s cubic-bezier(0.16, 1, 0.3, 1) 0.15s both;
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
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0.25s both;
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
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0.35s both;
  }}

  .meta-grid {{
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 1.75rem;
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0.4s both;
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
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0.5s both;
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
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0.6s both;
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
"##,
        accent = ACCENT
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
  }}

  body {{
    font-family: var(--sans);
    background: var(--bg);
    color: var(--fg);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
  }}

  body::before {{
    content: '';
    position: fixed;
    inset: 0;
    background: radial-gradient(circle at 50% 30%, rgba(128,0,0,0.06) 0%, transparent 60%);
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
"##,
        accent = ACCENT
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

pub fn render_html(
    name: &Name,
    zsk: &[u8; ZSK_LEN],
    ns: &str,
    records: &[ResourceRecord],
    _packet_hex: &str,
    timestamp: u64,
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

    format!(
        r#"{head}
<body>

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

{footer}

{particles}
</body>
</html>"#,
    )
}

pub fn render_not_found_page(name: &Name, zsk: Option<&[u8; ZSK_LEN]>, ns: Option<&str>) -> String {
    let name_str = name.to_string();
    let canonical = name.canonical_domain();
    let avatar_svg = name.render_avatar_svg();

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
        (String::new(), "This name is not registered on MNS.")
    };

    let style = main_style();
    let head = page_head(&name_str, &style);
    let particles = particles_script();
    let footer = footer_html();

    format!(
        r#"{head}
<body>

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
    {meta_rows}
  </div>

  <div class="empty-state">
    <p class="empty-text">{empty_text}</p>
  </div>
</div>

{footer}

{particles}
</body>
</html>"#,
    )
}

pub fn render_error(message: &str) -> String {
    let style = error_style();
    let head = page_head("Error", &style);
    format!(
        r#"{head}
<body>
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
