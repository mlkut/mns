use core::net::{Ipv4Addr, Ipv6Addr};

use super::{
    footer_html, format_timestamp, main_style, navbar_html, page_head, particles_script,
    truncate_addr, Name, Navbar, ResourceRecord, ZSK_LEN,
};

fn svcb_fields(svcb: &simple_dns::rdata::SVCB) -> String {
    let alpn: Vec<String> = svcb
        .iter_params()
        .filter_map(|p| match p {
            simple_dns::rdata::SVCParam::Alpn(ids) => {
                Some(ids.iter().map(|s| s.to_string()).collect::<Vec<_>>())
            }
            _ => None,
        })
        .flatten()
        .collect();
    let port = svcb
        .iter_params()
        .filter_map(|p| match p {
            simple_dns::rdata::SVCParam::Port(port) => Some(*port),
            _ => None,
        })
        .next();
    let mut s = format!(
        "\"priority\":{},\"target\":\"{}\"",
        svcb.priority, svcb.target
    );
    if !alpn.is_empty() {
        s.push_str(&format!(",\"alpn\":\"{}\"", alpn.join(",")));
    }
    if let Some(port) = port {
        s.push_str(&format!(",\"port\":{port}"));
    }
    s
}

fn records_to_json(records: &[ResourceRecord]) -> String {
    let mut items = Vec::new();
    for r in records {
        let rtype = format!("{:?}", r.rdata.type_code());
        let ttl = r.ttl;
        let name = r.name.to_string();
        let fields = match &r.rdata {
            simple_dns::rdata::RData::A(a) => {
                let octets = a.address.to_be_bytes();
                format!(
                    "\"address\":\"{}.{}.{}.{}\"",
                    octets[0], octets[1], octets[2], octets[3]
                )
            }
            simple_dns::rdata::RData::AAAA(aaaa) => {
                let octets = aaaa.address.to_be_bytes();
                format!(
                    "\"address\":\"{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}\"",
                    octets[0], octets[1], octets[2], octets[3],
                    octets[4], octets[5], octets[6], octets[7],
                    octets[8], octets[9], octets[10], octets[11],
                    octets[12], octets[13], octets[14], octets[15]
                )
            }
            simple_dns::rdata::RData::NS(ns) => {
                format!("\"target\":\"{}\"", ns.0)
            }
            simple_dns::rdata::RData::CNAME(cname) => {
                format!("\"target\":\"{}\"", cname.0)
            }
            simple_dns::rdata::RData::MX(mx) => {
                format!(
                    "\"preference\":{},\"exchange\":\"{}\"",
                    mx.preference, mx.exchange
                )
            }
            simple_dns::rdata::RData::TXT(txt) => {
                let s = txt
                    .iter_raw()
                    .filter_map(|(raw, _)| std::str::from_utf8(raw).ok())
                    .collect::<Vec<_>>()
                    .join(" ");
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"txt\":\"{escaped}\"")
            }
            simple_dns::rdata::RData::SRV(srv) => {
                format!(
                    "\"priority\":{},\"weight\":{},\"port\":{},\"target\":\"{}\"",
                    srv.priority, srv.weight, srv.port, srv.target
                )
            }
            simple_dns::rdata::RData::HTTPS(https) => svcb_fields(&https.0),
            simple_dns::rdata::RData::SVCB(svcb) => svcb_fields(svcb),
            simple_dns::rdata::RData::SOA(soa) => {
                format!(
                    "\"mname\":\"{}\",\"rname\":\"{}\",\"serial\":{},\"refresh\":{},\"retry\":{},\"expire\":{},\"minimum\":{}",
                    soa.mname, soa.rname, soa.serial, soa.refresh, soa.retry, soa.expire, soa.minimum
                )
            }
            simple_dns::rdata::RData::PTR(ptr) => {
                format!("\"target\":\"{}\"", ptr.0)
            }
            _ => String::new(),
        };
        items.push(format!(
            "{{\"type\":\"{rtype}\",\"ttl\":{ttl},\"name\":\"{name}\"{comma}{fields}}}",
            comma = if fields.is_empty() { "" } else { "," }
        ));
    }
    format!("[{}]", items.join(","))
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
    let zsk_full = format!("0x{zsk_hex}");
    let zsk_display = truncate_addr(&zsk_full);

    let mut rows = String::new();
    for r in records {
        let rtype = format!("{:?}", r.rdata.type_code());
        let name_s = r.name.to_string();
        let display_name = if name_s == canonical {
            "@".to_string()
        } else if let Some(rel) = name_s.strip_suffix(&format!(".{canonical}")) {
            rel.to_string()
        } else {
            name_s
        };
        let rdata_str = match &r.rdata {
            simple_dns::rdata::RData::A(a) => Ipv4Addr::from(a.address).to_string(),
            simple_dns::rdata::RData::AAAA(aaaa) => Ipv6Addr::from(aaaa.address).to_string(),
            simple_dns::rdata::RData::NS(ns) => ns.0.to_string(),
            simple_dns::rdata::RData::CNAME(cname) => cname.0.to_string(),
            simple_dns::rdata::RData::MX(mx) => format!("{} {}", mx.preference, mx.exchange),
            simple_dns::rdata::RData::TXT(txt) => txt
                .iter_raw()
                .filter_map(|(raw, _)| std::str::from_utf8(raw).ok())
                .collect::<Vec<_>>()
                .join(" "),
            simple_dns::rdata::RData::SRV(srv) => {
                format!(
                    "{} {} {} {}",
                    srv.priority, srv.weight, srv.port, srv.target
                )
            }
            simple_dns::rdata::RData::PTR(ptr) => ptr.0.to_string(),
            _ => format!("{:?}", r.rdata),
        };
        rows.push_str(&format!(
            r#"<tr><td class="name">{}</td><td class="type">{}</td><td class="ttl">{}</td><td class="rdata">{}</td></tr>"#,
            display_name, rtype, r.ttl, rdata_str
        ));
    }

    let ts = if timestamp > 0 {
        format_timestamp(timestamp)
    } else {
        "--".into()
    };

    let owner_row = if let Some(owner) = owner {
        let owner_hex = hex::encode(owner);
        let full = format!("0x{owner_hex}");
        let truncated = truncate_addr(&full);
        format!(
            r#"<div class="meta-row inline">
      <dt class="meta-label">Owner</dt>
      <dd class="meta-value dim">
        <a class="owner-link" href="/owner/0x{owner_hex}" title="{full}">{truncated}</a>
      </dd>
    </div>"#,
        )
    } else {
        String::new()
    };

    let records_json = records_to_json(records);

    let records_table = if records.is_empty() {
        String::new()
    } else {
        format!(
            r#"<table>
    <thead><tr><th scope="col">Name</th><th scope="col">Type</th><th scope="col">TTL</th><th scope="col">Data</th></tr></thead>
    <tbody>{rows}</tbody>
  </table>"#,
        )
    };

    let style = format!(
        r#"{main_style}

  .records-header {{
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }}

  .records-header h2 {{
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }}

  .editor-btn {{
    padding: 0.4rem 0.8rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: var(--radius-xs);
    font-family: var(--mono);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.2s;
  }}
  .editor-btn:hover {{ opacity: 0.85; }}
  .editor-btn.secondary {{
    background: transparent;
    border: 1px solid var(--border);
    color: var(--fg-muted);
  }}
  .editor-btn.secondary:hover {{
    border-color: var(--fg-muted);
    color: var(--fg);
  }}
  .editor-btn:disabled {{
    opacity: 0.4;
    cursor: not-allowed;
  }}

  .editor-empty {{
    text-align: center;
    padding: 1rem;
    color: var(--fg-muted);
    font-size: 0.82rem;
  }}

  .editor-form {{ display: none; }}
  .editor-form.show {{ display: block; }}

  .record-row {{
    display: flex;
    gap: 0.5rem;
    align-items: flex-start;
    padding: 0.6rem 0;
    border-bottom: 1px solid var(--border);
  }}
  .record-row:last-child {{ border-bottom: none; }}

  .record-row select,
  .record-row input,
  .record-row textarea {{
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-xs);
    color: var(--fg);
    font-family: var(--mono);
    font-size: 0.75rem;
    padding: 0.35rem 0.5rem;
    outline: none;
    transition: border-color 0.2s;
  }}
  .record-row select:focus,
  .record-row input:focus,
  .record-row textarea:focus {{
    border-color: var(--accent);
  }}

  .record-row select {{
    width: 80px;
    flex-shrink: 0;
  }}

  .record-row input.name-input {{
    width: 100px;
    flex-shrink: 0;
  }}

  .record-row input.ttl-input {{
    width: 50px;
    flex-shrink: 0;
  }}

  .record-row .record-fields {{
    display: flex;
    gap: 0.4rem;
    flex: 1;
    align-items: center;
    flex-wrap: wrap;
  }}

  .record-row .record-fields input {{
    flex: 1;
    min-width: 80px;
  }}

  .record-row .record-fields textarea {{
    width: 100%;
    min-height: 2.5rem;
    resize: vertical;
  }}

  .record-row .rec-label {{
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--fg-dim);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }}

  .record-row .rec-delete {{
    background: none;
    border: none;
    color: var(--fg-dim);
    cursor: pointer;
    padding: 0.2rem;
    font-size: 0.9rem;
    line-height: 1;
    transition: color 0.2s;
    flex-shrink: 0;
  }}
  .record-row .rec-delete:hover {{ color: #ef4444; }}

  .editor-actions {{
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }}

  .editor-status {{
    font-family: var(--mono);
    font-size: 0.75rem;
    color: var(--fg-muted);
    margin-top: 0.5rem;
  }}
  .editor-status.error {{ color: #ef4444; }}
  .editor-status.ok {{ color: #22c55e; }}

  @media (max-width: 480px) {{
    .record-row {{ flex-wrap: wrap; }}
    .record-row select {{ width: 100%; }}
    .record-row input.name-input {{ width: 60px; }}
    .record-row input.ttl-input {{ width: 40px; }}
  }}
"#,
        main_style = main_style()
    );

    let head = page_head(&name_str, &style);
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
<article class="card">
  <div class="avatar-wrap">
    <div class="avatar">{avatar_svg}</div>
  </div>

  <header class="header">
    <h1>{name_str}</h1>
    <p class="canonical">{canonical}</p>
  </header>

  <div class="divider" role="separator"></div>

  <dl class="meta-grid">
    {owner_row}
    <div class="meta-row inline">
      <dt class="meta-label">ZSK</dt>
      <dd class="meta-value dim" title="{zsk_full}">{zsk_display}</dd>
    </div>
    <div class="meta-row inline">
      <dt class="meta-label">NS</dt>
      <dd class="meta-value dim">{ns}</dd>
    </div>
    <div class="meta-row inline">
      <dt class="meta-label">Updated</dt>
      <dd class="meta-value dim">{ts}</dd>
    </div>
  </dl>

  <div class="divider" role="separator"></div>

  <div class="records-card" id="records-card">
    <div class="records-header">
      <h2>Signed Packet</h2>
      <div style="display:flex;gap:0.5rem">
        <button class="editor-btn" id="ed-create" style="display:none">Create</button>
        <button class="editor-btn secondary" id="ed-edit" style="display:none">Edit</button>
      </div>
    </div>
    <div id="ed-view">{records_table}</div>
    <div class="editor-form" id="ed-form">
      <div id="ed-records"></div>
      <button class="editor-btn secondary" id="ed-add" style="margin-top:0.5rem">+ Add Record</button>
      <div class="editor-actions">
        <button class="editor-btn" id="ed-publish">Publish</button>
        <button class="editor-btn secondary" id="ed-cancel">Cancel</button>
      </div>
      <div class="editor-status" id="ed-status"></div>
    </div>
  </div>
</article>

<nav class="back-wrap" aria-label="Breadcrumb">
  <a class="back-link" href="/">← Home</a>
</nav>
</main>

{footer}

{particles}
<script type="module">
import init, {{ create_signed_packet, derive_wallet_from_hex }} from '/static/mns-wasm/mns_wasm.js';
await init();

(function() {{
  var ZSK = '{zsk_hex}';
  var NAME = '{name_str}';
  var CANONICAL = '{canonical}';
  var PUB_TS = {timestamp};
  var PUB_RECORDS_JSON = '{records_json}';
  var DRAFT_KEY = 'mns-draft-' + NAME;

  var records = [];
  var edView = document.getElementById('ed-view');
  var edForm = document.getElementById('ed-form');
  var edRecords = document.getElementById('ed-records');
  var edStatus = document.getElementById('ed-status');
  var edEdit = document.getElementById('ed-edit');
  var edCreate = document.getElementById('ed-create');
  var edAdd = document.getElementById('ed-add');
  var edPublish = document.getElementById('ed-publish');
  var edCancel = document.getElementById('ed-cancel');

  function hasWallet() {{
    try {{ return !!localStorage.getItem('mns-wallet-zsk'); }} catch(e) {{ return false; }}
  }}

  function loadDraft() {{
    try {{
      var raw = localStorage.getItem(DRAFT_KEY);
      if (!raw) return null;
      var d = JSON.parse(raw);
      if (d.baseTimestamp !== PUB_TS) {{
        localStorage.removeItem(DRAFT_KEY);
        return null;
      }}
      return d;
    }} catch(e) {{ return null; }}
  }}

  function saveDraft() {{
    try {{
      localStorage.setItem(DRAFT_KEY, JSON.stringify({{
        records: records,
        baseTimestamp: PUB_TS
      }}));
    }} catch(e) {{}}
  }}

  function clearDraft() {{
    try {{ localStorage.removeItem(DRAFT_KEY); }} catch(e) {{}}
  }}

  function showButtons(create, edit) {{
    edCreate.style.display = create ? '' : 'none';
    edEdit.style.display = edit ? '' : 'none';
  }}

  function showForm() {{
    edRecords.innerHTML = '';
    records.forEach(function(r, i) {{ edRecords.appendChild(makeRow(r, i)); }});
    edView.style.display = 'none';
    edForm.classList.add('show');
  }}

  function hideForm() {{
    edForm.classList.remove('show');
    edView.style.display = '';
  }}

  function startEditing(recs) {{
    records = JSON.parse(JSON.stringify(recs));
    var suffix = '.' + CANONICAL;
    records.forEach(function(r) {{
      if (r.name && r.name !== '@') {{
        if (r.name === CANONICAL) r.name = '';
        else if (r.name.endsWith(suffix)) r.name = r.name.slice(0, -suffix.length);
      }}
    }});
    saveDraft();
    showButtons(false, false);
    showForm();
  }}

  function formatRdata(r) {{
    switch(r.type) {{
      case 'A': case 'AAAA': return r.address || '';
      case 'NS': case 'CNAME': case 'PTR': return r.target || '';
      case 'MX': return (r.preference||10) + ' ' + (r.exchange||'');
      case 'TXT': return r.txt || '';
      case 'SRV': return (r.priority||0) + ' ' + (r.weight||0) + ' ' + (r.port||0) + ' ' + (r.target||'');
      case 'HTTPS': case 'SVCB':
        var s = (r.priority||0) + ' ' + (r.target||'.');
        if (r.alpn) s += ' alpn=' + r.alpn;
        if (r.port) s += ' port=' + r.port;
        return s;
      case 'SOA': return (r.mname||'') + ' ' + (r.rname||'') + ' ' + (r.serial||0);
      default: return '';
    }}
  }}

  function esc(s) {{ var el = document.createElement('span'); el.textContent = s; return el.innerHTML; }}

  function makeRow(r, idx) {{
    var row = document.createElement('div');
    row.className = 'record-row';
    row.dataset.idx = idx;

    var sel = document.createElement('select');
    ['A','AAAA','NS','CNAME','MX','TXT','SRV','HTTPS','SVCB','SOA','PTR'].forEach(function(t) {{
      var opt = document.createElement('option');
      opt.value = t; opt.textContent = t;
      if (t === r.type) opt.selected = true;
      sel.appendChild(opt);
    }});
    sel.onchange = function() {{ r.type = this.value; rebuildRow(row, r, idx); }};

    var nameInput = document.createElement('input');
    nameInput.className = 'name-input';
    nameInput.placeholder = '@';
    nameInput.value = r.name || '';
    nameInput.oninput = function() {{ r.name = this.value; saveDraft(); }};

    var ttlInput = document.createElement('input');
    ttlInput.className = 'ttl-input';
    ttlInput.type = 'number';
    ttlInput.placeholder = 'TTL';
    ttlInput.value = r.ttl || 300;
    ttlInput.oninput = function() {{ r.ttl = parseInt(this.value) || 300; saveDraft(); }};

    var fields = document.createElement('div');
    fields.className = 'record-fields';

    var del = document.createElement('button');
    del.className = 'rec-delete';
    del.textContent = '\u00d7';
    del.onclick = function() {{
      records.splice(idx, 1);
      saveDraft();
      rebuildAll();
    }};

    row.appendChild(sel);
    row.appendChild(nameInput);
    row.appendChild(ttlInput);
    row.appendChild(fields);
    row.appendChild(del);

    buildFields(fields, r);
    return row;
  }}

  function rebuildRow(row, r, idx) {{
    var fields = row.querySelector('.record-fields');
    fields.innerHTML = '';
    buildFields(fields, r);
  }}

  function rebuildAll() {{ showForm(); }}

  function buildFields(container, r) {{
    switch(r.type) {{
      case 'A':
        addInput(container, 'Address', r.address || '', function(v) {{ r.address = v; saveDraft(); }});
        break;
      case 'AAAA':
        addInput(container, 'Address', r.address || '', function(v) {{ r.address = v; saveDraft(); }});
        break;
      case 'NS': case 'CNAME': case 'PTR':
        addInput(container, 'Target', r.target || '', function(v) {{ r.target = v; saveDraft(); }});
        break;
      case 'MX':
        addInput(container, 'Pref', r.preference || 10, function(v) {{ r.preference = parseInt(v)||10; saveDraft(); }}, 60);
        addInput(container, 'Exchange', r.exchange || '', function(v) {{ r.exchange = v; saveDraft(); }});
        break;
      case 'TXT':
        var ta = document.createElement('textarea');
        ta.placeholder = 'Text content';
        ta.value = r.txt || '';
        ta.oninput = function() {{ r.txt = this.value; saveDraft(); }};
        container.appendChild(ta);
        break;
      case 'SRV':
        addInput(container, 'Pri', r.priority || 10, function(v) {{ r.priority = parseInt(v)||10; saveDraft(); }}, 50);
        addInput(container, 'Wt', r.weight || 0, function(v) {{ r.weight = parseInt(v)||0; saveDraft(); }}, 50);
        addInput(container, 'Port', r.port || 443, function(v) {{ r.port = parseInt(v)||443; saveDraft(); }}, 60);
        addInput(container, 'Target', r.target || '', function(v) {{ r.target = v; saveDraft(); }});
        break;
      case 'HTTPS': case 'SVCB':
        addInput(container, 'Pri', r.priority || 1, function(v) {{ r.priority = parseInt(v)||1; saveDraft(); }}, 50);
        addInput(container, 'Target', r.target || '.', function(v) {{ r.target = v; saveDraft(); }});
        addInput(container, 'ALPN', r.alpn || 'h2,h3', function(v) {{ r.alpn = v; saveDraft(); }});
        addInput(container, 'Port', r.port || '', function(v) {{ r.port = v ? parseInt(v) : undefined; saveDraft(); }}, 60);
        break;
      case 'SOA':
        addInput(container, 'MNAME', r.mname || '', function(v) {{ r.mname = v; saveDraft(); }});
        addInput(container, 'RNAME', r.rname || '', function(v) {{ r.rname = v; saveDraft(); }});
        addInput(container, 'Serial', r.serial || 2024010101, function(v) {{ r.serial = parseInt(v)||0; saveDraft(); }}, 100);
        addInput(container, 'Refresh', r.refresh || 3600, function(v) {{ r.refresh = parseInt(v)||0; saveDraft(); }}, 70);
        addInput(container, 'Retry', r.retry || 600, function(v) {{ r.retry = parseInt(v)||0; saveDraft(); }}, 60);
        addInput(container, 'Expire', r.expire || 604800, function(v) {{ r.expire = parseInt(v)||0; saveDraft(); }}, 80);
        addInput(container, 'Min', r.minimum || 86400, function(v) {{ r.minimum = parseInt(v)||0; saveDraft(); }}, 70);
        break;
    }}
  }}

  function addInput(container, label, value, onChange, width) {{
    var lbl = document.createElement('span');
    lbl.className = 'rec-label';
    lbl.textContent = label;
    container.appendChild(lbl);
    var inp = document.createElement('input');
    inp.value = value || '';
    if (width) inp.style.width = width + 'px';
    inp.oninput = function() {{ onChange(this.value); }};
    container.appendChild(inp);
  }}

  edEdit.onclick = function() {{
    var pubRecords = [];
    try {{ pubRecords = JSON.parse(PUB_RECORDS_JSON); }} catch(e) {{}}
    startEditing(pubRecords);
  }};

  edCancel.onclick = function() {{
    clearDraft();
    hideForm();
    records = [];
    if (PUB_TS > 0) showButtons(false, true);
    else showButtons(true, false);
  }};

  edAdd.onclick = function() {{
    records.push({{ type: 'A', ttl: 300, name: '', address: '' }});
    saveDraft();
    rebuildAll();
  }};

  edPublish.onclick = async function() {{
    if (!hasWallet()) {{
      edStatus.textContent = 'No wallet found. Go to /wallet first.';
      edStatus.className = 'editor-status error';
      return;
    }}
    edStatus.textContent = 'Checking key...';
    edStatus.className = 'editor-status';
    edPublish.disabled = true;
    try {{
      var walletZsk = localStorage.getItem('mns-wallet-zsk');
      if (!walletZsk) {{
        edStatus.textContent = 'Wallet not found. Go to /wallet first.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      if (walletZsk !== ZSK) {{
        edStatus.textContent = 'Key does not match this name' + "'" + 's ZSK.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      var validRecords = records.filter(function(r) {{
        if (!r.type) return false;
        switch(r.type) {{
          case 'A': case 'AAAA': return !!r.address;
          case 'NS': case 'CNAME': case 'PTR': return !!r.target;
          case 'MX': return !!r.exchange;
          case 'TXT': return !!r.txt;
          case 'SRV': return !!r.target;
          case 'HTTPS': case 'SVCB': return true;
          case 'SOA': return !!r.mname;
          default: return true;
        }}
      }});

      if (validRecords.length === 0) {{
        edStatus.textContent = 'Add at least one record.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      var cred = null;
      try {{
        cred = await navigator.credentials.get({{
          password: true,
          mediation: 'optional'
        }});
      }} catch(e) {{}}

      if (!cred || !cred.password) {{
        edStatus.textContent = 'No credentials found. Unlock your wallet first.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      var parts = cred.password.split('\\n');
      if (parts.length !== 2) {{
        edStatus.textContent = 'Invalid credentials.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      var rskHex = parts[0];
      var keyType, keyHex;
      if (parts[1].length === 64) {{
        keyType = 0;
        keyHex = parts[1];
      }} else {{
        keyType = parseInt(parts[1].substring(0, 2), 16);
        keyHex = parts[1].substring(2);
      }}

      var derived = derive_wallet_from_hex(rskHex, keyType, keyHex);
      if (derived[1] !== ZSK) {{
        edStatus.textContent = 'Key does not match this name' + "'" + 's ZSK.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      edStatus.textContent = 'Signing...';
      var b64 = create_signed_packet(keyType, keyHex, NAME, JSON.stringify(validRecords));

      var bytes = Uint8Array.from(atob(b64), function(c) {{ return c.charCodeAt(0); }});
      var resp = await fetch('/' + NAME, {{
        method: 'PUT',
        headers: {{ 'Content-Type': 'application/mns.mlkut.org#SignedPacket' }},
        body: bytes
      }});

      if (resp.ok) {{
        clearDraft();
        edStatus.textContent = 'Published!';
        edStatus.className = 'editor-status ok';
        setTimeout(function() {{ window.location.reload(); }}, 1000);
      }} else {{
        var msg = await resp.text();
        edStatus.textContent = 'Error: ' + msg;
        edStatus.className = 'editor-status error';
      }}
    }} catch(e) {{
      edStatus.textContent = 'Error: ' + e.message;
      edStatus.className = 'editor-status error';
    }}
    edPublish.disabled = false;
  }};

  // Init
  if (hasWallet()) {{
    try {{
      var walletZsk = localStorage.getItem('mns-wallet-zsk');
      if (walletZsk === ZSK) {{
        var draft = loadDraft();
        if (draft) {{
          startEditing(draft.records);
        }} else if (PUB_TS > 0) {{
          showButtons(false, true);
        }} else {{
          showButtons(true, false);
        }}
      }}
    }} catch(e) {{}}
  }}
}})();
</script>
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
        let full = format!("0x{owner_hex}");
        let truncated = truncate_addr(&full);
        format!(
            r#"<div class="meta-row inline">
      <dt class="meta-label">Owner</dt>
      <dd class="meta-value dim">
        <a class="owner-link" href="/owner/0x{owner_hex}" title="{full}">{truncated}</a>
      </dd>
    </div>"#,
        )
    } else {
        String::new()
    };

    let zsk_hex = zsk.map(hex::encode).unwrap_or_default();
    let zsk_full = format!("0x{zsk_hex}");
    let zsk_display = truncate_addr(&zsk_full);
    let has_zsk = zsk.is_some();

    let (meta_rows, empty_text) = if let Some(zsk) = zsk {
        let ns_val = ns.unwrap_or("");
        let ns_row = if !ns_val.is_empty() {
            format!(
                r#"<div class="meta-row inline">
      <dt class="meta-label">NS</dt>
      <dd class="meta-value dim">{ns_val}</dd>
    </div>"#,
            )
        } else {
            String::new()
        };
        (
            format!(
                r#"<div class="meta-row inline">
      <dt class="meta-label">ZSK</dt>
      <dd class="meta-value dim" title="{zsk_full}">{zsk_display}</dd>
    </div>
    {ns_row}"#,
            ),
            "No signed packets published yet.",
        )
    } else {
        (String::new(), "This name is not registered on MNS yet!.")
    };

    let style = format!(
        r#"{main_style}

  .records-header {{
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }}

  .editor-btn {{
    padding: 0.4rem 0.8rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: var(--radius-xs);
    font-family: var(--mono);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.2s;
  }}
  .editor-btn:hover {{ opacity: 0.85; }}
  .editor-btn.secondary {{
    background: transparent;
    border: 1px solid var(--border);
    color: var(--fg-muted);
  }}
  .editor-btn.secondary:hover {{
    border-color: var(--fg-muted);
    color: var(--fg);
  }}
  .editor-btn:disabled {{
    opacity: 0.4;
    cursor: not-allowed;
  }}

  .editor-empty {{
    text-align: center;
    padding: 1rem;
    color: var(--fg-muted);
    font-size: 0.82rem;
  }}

  .editor-form {{ display: none; }}
  .editor-form.show {{ display: block; }}

  .record-row {{
    display: flex;
    gap: 0.5rem;
    align-items: flex-start;
    padding: 0.6rem 0;
    border-bottom: 1px solid var(--border);
  }}
  .record-row:last-child {{ border-bottom: none; }}

  .record-row select,
  .record-row input,
  .record-row textarea {{
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-xs);
    color: var(--fg);
    font-family: var(--mono);
    font-size: 0.75rem;
    padding: 0.35rem 0.5rem;
    outline: none;
    transition: border-color 0.2s;
  }}
  .record-row select:focus,
  .record-row input:focus,
  .record-row textarea:focus {{
    border-color: var(--accent);
  }}

  .record-row select {{
    width: 80px;
    flex-shrink: 0;
  }}

  .record-row input.name-input {{
    width: 100px;
    flex-shrink: 0;
  }}

  .record-row input.ttl-input {{
    width: 50px;
    flex-shrink: 0;
  }}

  .record-row .record-fields {{
    display: flex;
    gap: 0.4rem;
    flex: 1;
    align-items: center;
    flex-wrap: wrap;
  }}

  .record-row .record-fields input {{
    flex: 1;
    min-width: 80px;
  }}

  .record-row .record-fields textarea {{
    width: 100%;
    min-height: 2.5rem;
    resize: vertical;
  }}

  .record-row .rec-label {{
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--fg-dim);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }}

  .record-row .rec-delete {{
    background: none;
    border: none;
    color: var(--fg-dim);
    cursor: pointer;
    padding: 0.2rem;
    font-size: 0.9rem;
    line-height: 1;
    transition: color 0.2s;
    flex-shrink: 0;
  }}
  .record-row .rec-delete:hover {{ color: #ef4444; }}

  .editor-actions {{
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }}

  .editor-status {{
    font-family: var(--mono);
    font-size: 0.75rem;
    color: var(--fg-muted);
    margin-top: 0.5rem;
  }}
  .editor-status.error {{ color: #ef4444; }}
  .editor-status.ok {{ color: #22c55e; }}

  @media (max-width: 480px) {{
    .record-row {{ flex-wrap: wrap; }}
    .record-row select {{ width: 100%; }}
    .record-row input.name-input {{ width: 60px; }}
    .record-row input.ttl-input {{ width: 40px; }}
  }}
"#,
        main_style = main_style()
    );

    let head = page_head(&name_str, &style);
    let particles = particles_script();
    let footer = footer_html();
    let nav_html = navbar_html(nav);
    let history_script = format!(
        r#"<script>try{{var h=JSON.parse(localStorage.getItem('mns-history')||'[]');h=h.filter(function(n){{return n!=='{name_str}'}});h.unshift('{name_str}');if(h.length>5)h.length=5;localStorage.setItem('mns-history',JSON.stringify(h))}}catch(e){{}}</script>"#,
    );

    let editor_section = if has_zsk {
        r#"<div class="divider" role="separator"></div>
<div class="records-card" id="records-card">
  <div class="records-header">
    <h2>Signed Packet</h2>
    <div style="display:flex;gap:0.5rem">
      <button class="editor-btn" id="ed-create" style="display:none">Create</button>
      <button class="editor-btn secondary" id="ed-edit" style="display:none">Edit</button>
    </div>
  </div>
  <div id="ed-view"><p class="editor-empty">No signed packets published yet.</p></div>
  <div class="editor-form" id="ed-form">
    <div id="ed-records"></div>
    <button class="editor-btn secondary" id="ed-add" style="margin-top:0.5rem">+ Add Record</button>
    <div class="editor-actions">
      <button class="editor-btn" id="ed-publish">Publish</button>
      <button class="editor-btn secondary" id="ed-cancel">Cancel</button>
    </div>
    <div class="editor-status" id="ed-status"></div>
  </div>
</div>"#
    } else {
        ""
    };

    let editor_script = if has_zsk {
        format!(
            r#"<script type="module">
import init, {{ create_signed_packet, derive_wallet_from_hex }} from '/static/mns-wasm/mns_wasm.js';
await init();

(function() {{
  var ZSK = '{zsk_hex}';
  var NAME = '{name_str}';
  var CANONICAL = '{canonical}';
  var PUB_TS = 0;
  var PUB_RECORDS_JSON = '[]';
  var DRAFT_KEY = 'mns-draft-' + NAME;

  var records = [];
  var edView = document.getElementById('ed-view');
  var edForm = document.getElementById('ed-form');
  var edRecords = document.getElementById('ed-records');
  var edStatus = document.getElementById('ed-status');
  var edEdit = document.getElementById('ed-edit');
  var edCreate = document.getElementById('ed-create');
  var edAdd = document.getElementById('ed-add');
  var edPublish = document.getElementById('ed-publish');
  var edCancel = document.getElementById('ed-cancel');

  function hasWallet() {{
    try {{ return !!localStorage.getItem('mns-wallet-zsk'); }} catch(e) {{ return false; }}
  }}

  function loadDraft() {{
    try {{
      var raw = localStorage.getItem(DRAFT_KEY);
      if (!raw) return null;
      var d = JSON.parse(raw);
      if (d.baseTimestamp !== PUB_TS) {{
        localStorage.removeItem(DRAFT_KEY);
        return null;
      }}
      return d;
    }} catch(e) {{ return null; }}
  }}

  function saveDraft() {{
    try {{
      localStorage.setItem(DRAFT_KEY, JSON.stringify({{
        records: records,
        baseTimestamp: PUB_TS
      }}));
    }} catch(e) {{}}
  }}

  function clearDraft() {{
    try {{ localStorage.removeItem(DRAFT_KEY); }} catch(e) {{}}
  }}

  function showButtons(create, edit) {{
    edCreate.style.display = create ? '' : 'none';
    edEdit.style.display = edit ? '' : 'none';
  }}

  function showForm() {{
    edRecords.innerHTML = '';
    records.forEach(function(r, i) {{ edRecords.appendChild(makeRow(r, i)); }});
    edView.style.display = 'none';
    edForm.classList.add('show');
  }}

  function hideForm() {{
    edForm.classList.remove('show');
    edView.style.display = '';
  }}

  function startEditing(recs) {{
    records = JSON.parse(JSON.stringify(recs));
    var suffix = '.' + CANONICAL;
    records.forEach(function(r) {{
      if (r.name && r.name !== '@') {{
        if (r.name === CANONICAL) r.name = '';
        else if (r.name.endsWith(suffix)) r.name = r.name.slice(0, -suffix.length);
      }}
    }});
    saveDraft();
    showButtons(false, false);
    showForm();
  }}

  function formatRdata(r) {{
    switch(r.type) {{
      case 'A': case 'AAAA': return r.address || '';
      case 'NS': case 'CNAME': case 'PTR': return r.target || '';
      case 'MX': return (r.preference||10) + ' ' + (r.exchange||'');
      case 'TXT': return r.txt || '';
      case 'SRV': return (r.priority||0) + ' ' + (r.weight||0) + ' ' + (r.port||0) + ' ' + (r.target||'');
      case 'HTTPS': case 'SVCB':
        var s = (r.priority||0) + ' ' + (r.target||'.');
        if (r.alpn) s += ' alpn=' + r.alpn;
        if (r.port) s += ' port=' + r.port;
        return s;
      case 'SOA': return (r.mname||'') + ' ' + (r.rname||'') + ' ' + (r.serial||0);
      default: return '';
    }}
  }}

  function esc(s) {{ var el = document.createElement('span'); el.textContent = s; return el.innerHTML; }}

  function makeRow(r, idx) {{
    var row = document.createElement('div');
    row.className = 'record-row';
    row.dataset.idx = idx;

    var sel = document.createElement('select');
    ['A','AAAA','NS','CNAME','MX','TXT','SRV','HTTPS','SVCB','SOA','PTR'].forEach(function(t) {{
      var opt = document.createElement('option');
      opt.value = t; opt.textContent = t;
      if (t === r.type) opt.selected = true;
      sel.appendChild(opt);
    }});
    sel.onchange = function() {{ r.type = this.value; rebuildRow(row, r, idx); }};

    var nameInput = document.createElement('input');
    nameInput.className = 'name-input';
    nameInput.placeholder = '@';
    nameInput.value = r.name || '';
    nameInput.oninput = function() {{ r.name = this.value; saveDraft(); }};

    var ttlInput = document.createElement('input');
    ttlInput.className = 'ttl-input';
    ttlInput.type = 'number';
    ttlInput.placeholder = 'TTL';
    ttlInput.value = r.ttl || 300;
    ttlInput.oninput = function() {{ r.ttl = parseInt(this.value) || 300; saveDraft(); }};

    var fields = document.createElement('div');
    fields.className = 'record-fields';

    var del = document.createElement('button');
    del.className = 'rec-delete';
    del.textContent = '\u00d7';
    del.onclick = function() {{
      records.splice(idx, 1);
      saveDraft();
      rebuildAll();
    }};

    row.appendChild(sel);
    row.appendChild(nameInput);
    row.appendChild(ttlInput);
    row.appendChild(fields);
    row.appendChild(del);

    buildFields(fields, r);
    return row;
  }}

  function rebuildRow(row, r, idx) {{
    var fields = row.querySelector('.record-fields');
    fields.innerHTML = '';
    buildFields(fields, r);
  }}

  function rebuildAll() {{ showForm(); }}

  function buildFields(container, r) {{
    switch(r.type) {{
      case 'A':
        addInput(container, 'Address', r.address || '', function(v) {{ r.address = v; saveDraft(); }});
        break;
      case 'AAAA':
        addInput(container, 'Address', r.address || '', function(v) {{ r.address = v; saveDraft(); }});
        break;
      case 'NS': case 'CNAME': case 'PTR':
        addInput(container, 'Target', r.target || '', function(v) {{ r.target = v; saveDraft(); }});
        break;
      case 'MX':
        addInput(container, 'Pref', r.preference || 10, function(v) {{ r.preference = parseInt(v)||10; saveDraft(); }}, 60);
        addInput(container, 'Exchange', r.exchange || '', function(v) {{ r.exchange = v; saveDraft(); }});
        break;
      case 'TXT':
        var ta = document.createElement('textarea');
        ta.placeholder = 'Text content';
        ta.value = r.txt || '';
        ta.oninput = function() {{ r.txt = this.value; saveDraft(); }};
        container.appendChild(ta);
        break;
      case 'SRV':
        addInput(container, 'Pri', r.priority || 10, function(v) {{ r.priority = parseInt(v)||10; saveDraft(); }}, 50);
        addInput(container, 'Wt', r.weight || 0, function(v) {{ r.weight = parseInt(v)||0; saveDraft(); }}, 50);
        addInput(container, 'Port', r.port || 443, function(v) {{ r.port = parseInt(v)||443; saveDraft(); }}, 60);
        addInput(container, 'Target', r.target || '', function(v) {{ r.target = v; saveDraft(); }});
        break;
      case 'HTTPS': case 'SVCB':
        addInput(container, 'Pri', r.priority || 1, function(v) {{ r.priority = parseInt(v)||1; saveDraft(); }}, 50);
        addInput(container, 'Target', r.target || '.', function(v) {{ r.target = v; saveDraft(); }});
        addInput(container, 'ALPN', r.alpn || 'h2,h3', function(v) {{ r.alpn = v; saveDraft(); }});
        addInput(container, 'Port', r.port || '', function(v) {{ r.port = v ? parseInt(v) : undefined; saveDraft(); }}, 60);
        break;
      case 'SOA':
        addInput(container, 'MNAME', r.mname || '', function(v) {{ r.mname = v; saveDraft(); }});
        addInput(container, 'RNAME', r.rname || '', function(v) {{ r.rname = v; saveDraft(); }});
        addInput(container, 'Serial', r.serial || 2024010101, function(v) {{ r.serial = parseInt(v)||0; saveDraft(); }}, 100);
        addInput(container, 'Refresh', r.refresh || 3600, function(v) {{ r.refresh = parseInt(v)||0; saveDraft(); }}, 70);
        addInput(container, 'Retry', r.retry || 600, function(v) {{ r.retry = parseInt(v)||0; saveDraft(); }}, 60);
        addInput(container, 'Expire', r.expire || 604800, function(v) {{ r.expire = parseInt(v)||0; saveDraft(); }}, 80);
        addInput(container, 'Min', r.minimum || 86400, function(v) {{ r.minimum = parseInt(v)||0; saveDraft(); }}, 70);
        break;
    }}
  }}

  function addInput(container, label, value, onChange, width) {{
    var lbl = document.createElement('span');
    lbl.className = 'rec-label';
    lbl.textContent = label;
    container.appendChild(lbl);
    var inp = document.createElement('input');
    inp.value = value || '';
    if (width) inp.style.width = width + 'px';
    inp.oninput = function() {{ onChange(this.value); }};
    container.appendChild(inp);
  }}

  edCreate.onclick = function() {{
    var draft = loadDraft();
    var recs = draft ? draft.records : [];
    startEditing(recs);
  }};

  edCancel.onclick = function() {{
    clearDraft();
    hideForm();
    records = [];
    showButtons(true, false);
  }};

  edAdd.onclick = function() {{
    records.push({{ type: 'A', ttl: 300, name: '', address: '' }});
    saveDraft();
    rebuildAll();
  }};

  edPublish.onclick = async function() {{
    if (!hasWallet()) {{
      edStatus.textContent = 'No wallet found. Go to /wallet first.';
      edStatus.className = 'editor-status error';
      return;
    }}
    edStatus.textContent = 'Checking key...';
    edStatus.className = 'editor-status';
    edPublish.disabled = true;
    try {{
      var walletZsk = localStorage.getItem('mns-wallet-zsk');
      if (!walletZsk) {{
        edStatus.textContent = 'Wallet not found. Go to /wallet first.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      if (walletZsk !== ZSK) {{
        edStatus.textContent = 'Key does not match this name' + "'" + 's ZSK.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      var validRecords = records.filter(function(r) {{
        if (!r.type) return false;
        switch(r.type) {{
          case 'A': case 'AAAA': return !!r.address;
          case 'NS': case 'CNAME': case 'PTR': return !!r.target;
          case 'MX': return !!r.exchange;
          case 'TXT': return !!r.txt;
          case 'SRV': return !!r.target;
          case 'HTTPS': case 'SVCB': return true;
          case 'SOA': return !!r.mname;
          default: return true;
        }}
      }});

      if (validRecords.length === 0) {{
        edStatus.textContent = 'Add at least one record.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      var cred = null;
      try {{
        cred = await navigator.credentials.get({{
          password: true,
          mediation: 'optional'
        }});
      }} catch(e) {{}}

      if (!cred || !cred.password) {{
        edStatus.textContent = 'No credentials found. Unlock your wallet first.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      var parts = cred.password.split('\\n');
      if (parts.length !== 2) {{
        edStatus.textContent = 'Invalid credentials.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      var rskHex = parts[0];
      var keyType, keyHex;
      if (parts[1].length === 64) {{
        keyType = 0;
        keyHex = parts[1];
      }} else {{
        keyType = parseInt(parts[1].substring(0, 2), 16);
        keyHex = parts[1].substring(2);
      }}

      var derived = derive_wallet_from_hex(rskHex, keyType, keyHex);
      if (derived[1] !== ZSK) {{
        edStatus.textContent = 'Key does not match this name' + "'" + 's ZSK.';
        edStatus.className = 'editor-status error';
        edPublish.disabled = false;
        return;
      }}

      edStatus.textContent = 'Signing...';
      var b64 = create_signed_packet(keyType, keyHex, NAME, JSON.stringify(validRecords));

      var bytes = Uint8Array.from(atob(b64), function(c) {{ return c.charCodeAt(0); }});
      var resp = await fetch('/' + NAME, {{
        method: 'PUT',
        headers: {{ 'Content-Type': 'application/mns.mlkut.org#SignedPacket' }},
        body: bytes
      }});

      if (resp.ok) {{
        clearDraft();
        edStatus.textContent = 'Published!';
        edStatus.className = 'editor-status ok';
        setTimeout(function() {{ window.location.reload(); }}, 1000);
      }} else {{
        var msg = await resp.text();
        edStatus.textContent = 'Error: ' + msg;
        edStatus.className = 'editor-status error';
      }}
    }} catch(e) {{
      edStatus.textContent = 'Error: ' + e.message;
      edStatus.className = 'editor-status error';
    }}
    edPublish.disabled = false;
  }};

  if (hasWallet()) {{
    try {{
      var walletZsk = localStorage.getItem('mns-wallet-zsk');
      if (walletZsk === ZSK) {{
        var draft = loadDraft();
        if (draft) {{
          startEditing(draft.records);
        }} else {{
          showButtons(true, false);
        }}
      }}
    }} catch(e) {{}}
  }}
}})();
</script>"#
        )
    } else {
        String::new()
    };

    format!(
        r#"{head}
<body>

{nav_html}
<div class="grid-bg" aria-hidden="true"></div>
<div class="particles" id="particles" aria-hidden="true"></div>

<main>
<article class="card">
  <div class="avatar-wrap">
    <div class="avatar">{avatar_svg}</div>
  </div>

  <header class="header">
    <h1>{name_str}</h1>
    <p class="canonical">{canonical}</p>
  </header>

  <div class="divider" role="separator"></div>

  <dl class="meta-grid">
    {owner_row}
    {meta_rows}
  </dl>

  {editor_section}
</article>

<nav class="back-wrap" aria-label="Breadcrumb">
  <a class="back-link" href="/">← Home</a>
</nav>
</main>

{footer}

{particles}
{history_script}
{editor_script}
</body>
</html>"#,
    )
}
