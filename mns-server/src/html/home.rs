use super::{footer_html, main_style, navbar_html, page_head, particles_script, Navbar};

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

  .content-grid {{
    display: flex;
    gap: 1.25rem;
    margin-top: 1.5rem;
    align-items: stretch;
  }}

  .stats-card {{
    flex: 0 0 200px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    list-style: none;
    overflow: hidden;
  }}

  .stat-row {{
    border-bottom: 1px solid var(--border);
  }}
  .stat-row:last-child {{
    border-bottom: none;
  }}

  .stat-row > a,
  .stat-row > .stat-row-content {{
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    font-size: 0.78rem;
    text-decoration: none;
    color: var(--fg);
    transition: background 0.2s;
  }}
  .stat-row > a:hover,
  .stat-row > .stat-row-content:hover {{
    background: var(--surface-hover);
  }}
  .stat-row .stat-label {{
    color: var(--fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 0.68rem;
  }}
  .stat-row .stat-value {{
    font-family: var(--mono);
    font-weight: 600;
    font-size: 0.82rem;
  }}

  #history {{
    flex: 1;
    min-width: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }}
  #history:empty::before {{
    content: 'No recently viewed names';
    margin: auto;
    padding: 0.5rem;
    color: var(--fg-muted);
    font-size: 0.78rem;
    opacity: 0.4;
  }}

  @media (max-width: 640px) {{
    .content-grid {{
      flex-direction: column;
    }}
    .stats-card {{
      flex: none;
      width: 100%;
    }}
  }}

  .history-list {{
    display: flex;
    flex-direction: column;
    list-style: none;
  }}
  .history-item {{
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.4rem 0.75rem;
    border-radius: 0;
    cursor: pointer;
    transition: background 0.2s, opacity 0.2s;
    text-decoration: none;
    color: var(--fg);
    opacity: 0.5;
  }}
  .history-item:hover,
  .history-item:focus {{
    background: var(--surface-hover);
    opacity: 1;
  }}
  .history-avatar {{
    width: 28px;
    height: 28px;
    color: var(--fg);
    flex-shrink: 0;
    border-radius: 4px;
    overflow: hidden;
    display: inline-block;
  }}
  .history-avatar svg {{
    width: 100%;
    height: 100%;
    display: block;
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

<div class="grid-bg" aria-hidden="true"></div>
<div class="particles" id="particles" aria-hidden="true"></div>

<main>
<section class="card card-center">
  <div class="avatar-wrap">
    <img src="/static/mlkut.png" alt="mlkut logo" class="avatar home-logo">
  </div>

  <header class="header">
    <h1>Mlkut Name System</h1>
    <p class="home-desc">
      Permissionless name registry for the next millennium.
    </p>
  </header>

  <div class="divider" role="separator"></div>

  <form id="search-form" role="search" onsubmit="event.preventDefault();var q=this.querySelector('input').value.trim();if(q){{if(q.startsWith('0x')){{window.location.href='/owner/'+encodeURIComponent(q);return;}}window.location.href='/'+encodeURIComponent(q);}}">
    <div class="search-row">
      <label for="search-input" class="visually-hidden">Search name or owner address</label>
      <input type="text" id="search-input" name="q" class="search-input" placeholder="Search name or owner address...">
      <button type="submit" class="search-btn">Search</button>
    </div>
  </form>

  <div class="content-grid">
    <ul class="stats-card" id="stats" aria-label="Registry statistics">
      <li class="stat-row">
        <div class="stat-row-content" title="Most recently synced Rootstock block">
          <span class="stat-label">Block</span>
          <span class="stat-value"><span id="stat-block">—</span></span>
        </div>
      </li>
      <li class="stat-row">
        <div class="stat-row-content" title="Total registered name slots (batch × 256 + entries)">
          <span class="stat-label">Names</span>
          <span class="stat-value" id="stat-names">—</span>
        </div>
      </li>
      <li class="stat-row">
        <a href="/owners" title="Unique addresses that own one or more names">
          <span class="stat-label">Owners</span>
          <span class="stat-value" id="stat-owners">—</span>
        </a>
      </li>
      <li class="stat-row">
        <div class="stat-row-content" title="Unique zone-signing keys across all registered batches and entries">
          <span class="stat-label">ZSKs</span>
          <span class="stat-value" id="stat-zsks">—</span>
        </div>
      </li>
      <li class="stat-row">
        <div class="stat-row-content" title="Unique name servers across all registered batches and entries">
          <span class="stat-label">NS</span>
          <span class="stat-value" id="stat-ns">—</span>
        </div>
      </li>
      <li class="stat-row">
        <div class="stat-row-content" title="Signed DNS packets published off-chain on this server">
          <span class="stat-label">Packets</span>
          <span class="stat-value" id="stat-packets">—</span>
        </div>
      </li>
    </ul>

    <section id="history" aria-label="Recently viewed names"></section>
  </div>

  <nav class="home-links" aria-label="External resources">
    <a href="https://mlkut.org" class="ext-link">Read more</a>
    <span class="sep-dot" aria-hidden="true">·</span>
    <a href="https://mlkut.org" class="ext-link">Specs</a>
  </nav>
</section>
</main>

{footer}

{particles}

<script>
(function() {{
  fetch('/stats').then(function(r){{return r.json()}}).then(function(d){{
    document.getElementById('stat-owners').textContent=d.total_owners;
    document.getElementById('stat-names').textContent=d.total_names;
    document.getElementById('stat-packets').textContent=d.total_packets;
    document.getElementById('stat-ns').textContent=d.total_ns;
    document.getElementById('stat-zsks').textContent=d.total_zsks;
    document.getElementById('stat-block').textContent=d.last_block;
  }}).catch(function(){{}});

  var list;
  try {{ list = JSON.parse(localStorage.getItem('mns-history') || '[]'); }} catch(e) {{ list = []; }}
  if (list.length === 0) return;
  var html = '<ul class="history-list">';
  var max = Math.min(list.length, 5);
  for (var i = 0; i < max; i++) {{
    var name = list[i];
    html += '<li><a class="history-item" href="/' + encodeURIComponent(name) + '">' +
      '<span class="history-avatar" data-name="' + encodeURIComponent(name) + '"></span>' +
      '<span class="history-name">' + name + '</span></a></li>';
  }}
  html += '</ul>';
  document.getElementById('history').innerHTML = html;
  var avatars = document.querySelectorAll('.history-avatar[data-name]');
  for (var i = 0; i < avatars.length; i++) {{
    (function(el) {{
      fetch('/avatar/' + el.getAttribute('data-name')).then(function(r){{return r.text()}}).then(function(svg){{el.innerHTML=svg;var s=el.querySelector('svg');if(s){{s.style.cssText='width:100%;height:100%;display:block;';s.removeAttribute('width');s.removeAttribute('height')}}}}).catch(function(){{}});
    }})(avatars[i]);
  }}
}})();
</script>

</body>
</html>"#,
    )
}
