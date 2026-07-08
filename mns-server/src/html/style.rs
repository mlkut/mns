use super::ACCENT;

pub fn main_style() -> String {
    format!(
        r##"
  *, *::before, *::after {{ margin: 0; padding: 0; box-sizing: border-box; }}

  :root {{
    --white: #e4e8ef;

    --bg: #08090d;
    --surface: rgba(255,255,255,0.03);
    --surface-hover: rgba(255,255,255,0.055);
    --border: rgba(255,255,255,0.06);
    --border-focus: rgba(128,0,0,0.3);
    --fg: var(--white);
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
    --radius-xs: 5px;
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

  main {{
    position: relative;
    z-index: 1;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
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

  .avatar .mns-avatar {{ 
    color: var(--white);
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

  .meta-row.inline dd {{
    text-align: left;
  }}

  dt.meta-label {{
    color: var(--fg-muted);
    font-size: 0.7rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }}

  dd.meta-value {{
    font-family: var(--mono);
    font-size: 0.82rem;
    color: var(--fg);
    word-break: break-all;
    line-height: 1.5;
    opacity: 0.85;
  }}

  dd.meta-value.dim {{
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
    color: var(--fg-muted);
    font-size: 0.88rem;
    font-weight: 400;
    line-height: 1.5;
    animation: fadeUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) 0s both;
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
    .meta-row.inline dd {{ text-align: left; }}
  }}

  @media (prefers-reduced-motion: reduce) {{
    *, *::before, *::after {{
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
    }}
  }}

  .owner-link {{
    color: var(--accent-text);
    text-decoration: none;
  }}

  .back-wrap {{
    text-align: center;
    margin-top: 0.75rem;
    z-index: 1;
    position: relative;
  }}

  .back-link {{
    color: var(--fg-muted);
    font-size: 0.75rem;
    text-decoration: none;
  }}

  .card-center {{
    text-align: center;
  }}

  .home-logo {{
    width: 128px;
    height: 128px;
    object-fit: contain;
  }}

  .home-desc {{
    font-family: var(--sans);
    opacity: 1;
    margin-top: 1rem;
  }}

  .search-input {{
    padding: 0 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--fg);
    font-family: var(--mono);
    font-size: 0.9rem;
    outline: none;
    transition: border-color 0.25s;
  }}
  .search-input:focus {{
    border-color: var(--accent);
  }}

  .search-btn {{
    padding: 0.75rem 2rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--sans);
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: opacity 0.25s;
  }}
  .search-btn:hover {{
    opacity: 0.85;
  }}

  .home-links {{
    margin-top: 1.5rem;
    display: flex;
    gap: 1rem;
    justify-content: center;
    font-size: 0.82rem;
  }}

  .ext-link {{
    color: var(--accent-text);
    text-decoration: none;
    font-weight: 500;
  }}

  .sep-dot {{
    color: var(--fg-dim);
  }}

  .visually-hidden {{
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }}

  {navbar_style}
"##,
        accent = ACCENT,
        navbar_style = navbar_style()
    )
}

pub fn error_style() -> String {
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

  main {{
    position: relative;
    z-index: 1;
    width: 100%;
    display: flex;
    justify-content: center;
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
