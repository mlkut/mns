use super::{
    footer_html, format_timestamp, main_style, navbar_html, page_head, particles_script,
    truncate_addr, Navbar, Name, ResourceRecord, ZSK_LEN,
};

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

    let records_section = if records.is_empty() {
        String::new()
    } else {
        format!(
            r#"<section class="card records-card" aria-labelledby="records-heading">
  <h2 id="records-heading">Resource Records</h2>
  <table>
    <thead><tr><th scope="col">Name</th><th scope="col">Type</th><th scope="col">TTL</th><th scope="col">Data</th></tr></thead>
    <tbody>{rows}</tbody>
  </table>
</section>"#,
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
      <dd class="meta-value dim">0x{zsk_hex}</dd>
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

  <p class="empty-state">No signed packets published yet.</p>
</article>

{records_section}

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

    let (meta_rows, empty_text) = if let Some(zsk) = zsk {
        let zsk_hex = hex::encode(zsk);
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
      <dd class="meta-value dim">0x{zsk_hex}</dd>
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
    let history_script = if owner.is_some() {
        format!(
            r#"<script>try{{var h=JSON.parse(localStorage.getItem('mns-history')||'[]');h=h.filter(function(n){{return n!=='{name_str}'}});h.unshift('{name_str}');if(h.length>4)h.length=4;localStorage.setItem('mns-history',JSON.stringify(h))}}catch(e){{}}</script>"#,
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

  <p class="empty-state">{empty_text}</p>
</article>

<nav class="back-wrap" aria-label="Breadcrumb">
  <a class="back-link" href="/">← Home</a>
</nav>
</main>

{footer}

{particles}
{history_script}
</body>
</html>"#,
    )
}
