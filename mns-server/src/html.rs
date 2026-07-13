use mns::{Name, ZSK_LEN};
use simple_dns::ResourceRecord;

pub struct Navbar {
    pub sync_block: u64,
    pub sync_time: u64,
    pub network: String,
    pub explorer_url: String,
    pub contract_address: String,
    pub chain_id: u64,
    pub rpc_url: String,
}

pub struct OwnerItemSimple {
    pub name_or_addr: String,
}

pub fn truncate_addr(addr: &str) -> String {
    if addr.len() > 18 {
        format!("{}…{}", &addr[..10], &addr[addr.len() - 8..])
    } else {
        addr.to_string()
    }
}

pub fn format_timestamp(ts: u64) -> String {
    let secs = (ts / 1_000_000) as i64;
    let Ok(dt) = time::OffsetDateTime::from_unix_timestamp(secs) else {
        return ts.to_string();
    };
    let dt = dt.to_offset(time::UtcOffset::UTC);
    dt.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| ts.to_string())
}

fn env() -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();
    env.add_template("base.html", include_str!("../templates/base.html"))
        .unwrap();
    env.add_template("navbar.html", include_str!("../templates/navbar.html"))
        .unwrap();
    env.add_template("footer.html", include_str!("../templates/footer.html"))
        .unwrap();
    env.add_template("home.html", include_str!("../templates/home.html"))
        .unwrap();
    env.add_template("name.html", include_str!("../templates/name.html"))
        .unwrap();
    env.add_template(
        "name_not_found.html",
        include_str!("../templates/name_not_found.html"),
    )
    .unwrap();
    env.add_template("owner.html", include_str!("../templates/owner.html"))
        .unwrap();
    env.add_template("owners.html", include_str!("../templates/owners.html"))
        .unwrap();
    env.add_template("wallet.html", include_str!("../templates/wallet.html"))
        .unwrap();
    env.add_template("error.html", include_str!("../templates/error.html"))
        .unwrap();
    env
}

fn render(env: &minijinja::Environment, template: &str, ctx: minijinja::Value) -> String {
    env.get_template(template).unwrap().render(ctx).unwrap()
}

pub fn render_home_page(nav: &Navbar) -> String {
    let env = env();
    let ctx = minijinja::Value::from_serialize(&serde_json::json!({
        "title": "Mlkut Name System",
        "network": nav.network,
        "block": nav.sync_block,
        "sync_time": nav.sync_time,
        "block_url": format!("{}/block/{}", nav.explorer_url.trim_end_matches('/'), nav.sync_block),
        "contract_url": format!("{}/address/{}?tab=contract", nav.explorer_url.trim_end_matches('/'), nav.contract_address),
    }));
    render(&env, "home.html", ctx)
}

pub fn render_error(message: &str, nav: &Navbar) -> String {
    let env = env();
    let ctx = minijinja::Value::from_serialize(&serde_json::json!({
        "title": "Error",
        "message": message,
        "network": nav.network,
        "block": nav.sync_block,
        "sync_time": nav.sync_time,
        "block_url": format!("{}/block/{}", nav.explorer_url.trim_end_matches('/'), nav.sync_block),
        "contract_url": format!("{}/address/{}?tab=contract", nav.explorer_url.trim_end_matches('/'), nav.contract_address),
    }));
    render(&env, "error.html", ctx)
}

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
        use core::net::{Ipv4Addr, Ipv6Addr};
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

    let editor_config = serde_json::json!({
        "zsk": zsk_hex,
        "name": name_str,
        "canonical": canonical,
        "pubTs": timestamp,
        "pubRecordsJson": records_json,
        "hasZsk": true,
        "ns": ns,
    });

    let env = env();
    let ctx = minijinja::Value::from_serialize(&serde_json::json!({
        "title": name_str,
        "name": name_str,
        "canonical": canonical,
        "avatar_svg": avatar_svg,
        "zsk_full": zsk_full,
        "zsk_display": zsk_display,
        "ns": ns,
        "timestamp": ts,
        "owner_row": owner_row,
        "records_table": records_table,
        "editor_config": editor_config.to_string(),
        "network": nav.network,
        "block": nav.sync_block,
        "sync_time": nav.sync_time,
        "block_url": format!("{}/block/{}", nav.explorer_url.trim_end_matches('/'), nav.sync_block),
        "contract_url": format!("{}/address/{}?tab=contract", nav.explorer_url.trim_end_matches('/'), nav.contract_address),
    }));
    render(&env, "name.html", ctx)
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

    let (meta_rows, empty_text) = if let Some(_zsk) = zsk {
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

    let editor_section = if has_zsk {
        format!(
            r#"<div class="divider" role="separator"></div>
<div class="records-card" id="records-card">
  <div class="records-header">
    <h2>Signed Packet</h2>
    <div style="display:flex;gap:0.5rem">
      <button class="editor-btn" id="ed-create" style="display:none">Create</button>
      <button class="editor-btn secondary" id="ed-edit" style="display:none">Edit</button>
    </div>
  </div>
  <div id="ed-view"><p class="editor-empty">{empty_text}</p></div>
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
        )
    } else {
        String::new()
    };

    let editor_config = if has_zsk {
        serde_json::json!({
            "zsk": zsk_hex,
            "name": name_str,
            "canonical": canonical,
            "pubTs": 0,
            "pubRecordsJson": "[]",
            "hasZsk": true,
            "ns": ns.unwrap_or(""),
        })
    } else {
        serde_json::json!({
            "zsk": "",
            "name": name_str,
            "canonical": canonical,
            "pubTs": 0,
            "pubRecordsJson": "[]",
            "hasZsk": false,
            "ns": ns.unwrap_or(""),
        })
    };

    let env = env();
    let ctx = minijinja::Value::from_serialize(&serde_json::json!({
        "title": name_str,
        "name": name_str,
        "canonical": canonical,
        "avatar_svg": avatar_svg,
        "owner_row": owner_row,
        "meta_rows": meta_rows,
        "has_zsk": has_zsk,
        "editor_section": editor_section,
        "editor_config": editor_config.to_string(),
        "network": nav.network,
        "block": nav.sync_block,
        "sync_time": nav.sync_time,
        "block_url": format!("{}/block/{}", nav.explorer_url.trim_end_matches('/'), nav.sync_block),
        "contract_url": format!("{}/address/{}?tab=contract", nav.explorer_url.trim_end_matches('/'), nav.contract_address),
    }));
    render(&env, "name_not_found.html", ctx)
}

pub fn render_owner_page(address: &str, names: &[Name], nav: &Navbar) -> String {
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

    let env = env();
    let ctx = minijinja::Value::from_serialize(&serde_json::json!({
        "title": "Owner",
        "address": address,
        "truncated": truncate_addr(address),
        "names_html": rows,
        "network": nav.network,
        "block": nav.sync_block,
        "sync_time": nav.sync_time,
        "block_url": format!("{}/block/{}", nav.explorer_url.trim_end_matches('/'), nav.sync_block),
        "contract_url": format!("{}/address/{}?tab=contract", nav.explorer_url.trim_end_matches('/'), nav.contract_address),
    }));
    render(&env, "owner.html", ctx)
}

pub fn render_owners_page(items: &[OwnerItemSimple], nav: &Navbar) -> String {
    let owners_html: String = if items.is_empty() {
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

    let env = env();
    let ctx = minijinja::Value::from_serialize(&serde_json::json!({
        "title": "Owners",
        "owners_html": owners_html,
        "network": nav.network,
        "block": nav.sync_block,
        "sync_time": nav.sync_time,
        "block_url": format!("{}/block/{}", nav.explorer_url.trim_end_matches('/'), nav.sync_block),
        "contract_url": format!("{}/address/{}?tab=contract", nav.explorer_url.trim_end_matches('/'), nav.contract_address),
    }));
    render(&env, "owners.html", ctx)
}

pub fn render_wallet_page(nav: &Navbar) -> String {
    let faucet = if nav.chain_id == 31 {
        "https://faucet.rootstock.io/"
    } else {
        ""
    };

    let wallet_config = serde_json::json!({
        "rpcUrl": nav.rpc_url,
        "chainId": nav.chain_id,
        "faucet": faucet,
    });

    let env = env();
    let ctx = minijinja::Value::from_serialize(&serde_json::json!({
        "title": "Wallet",
        "wallet_config": wallet_config.to_string(),
        "network": nav.network,
        "block": nav.sync_block,
        "sync_time": nav.sync_time,
        "block_url": format!("{}/block/{}", nav.explorer_url.trim_end_matches('/'), nav.sync_block),
        "contract_url": format!("{}/address/{}?tab=contract", nav.explorer_url.trim_end_matches('/'), nav.contract_address),
    }));
    render(&env, "wallet.html", ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nav() -> Navbar {
        Navbar {
            sync_block: 123,
            sync_time: 0,
            network: "testnet".into(),
            explorer_url: "https://explorer.testnet.rootstock.io".into(),
            contract_address: "0xabc".into(),
            chain_id: 31,
            rpc_url: "https://public-node.testnet.rsk.co".into(),
        }
    }

    #[test]
    fn renders_without_stray_placeholders() {
        let html = render_wallet_page(&nav());
        assert!(html.contains("Your Wallet"));
        assert!(html.contains("wallet.js"));
        assert!(!html.contains("{main_style}"));
        assert!(!html.contains("{faucet_js}"));
        assert!(!html.contains("{chain_id}"));
    }

    #[test]
    fn has_new_wallet_lifecycle() {
        let html = render_wallet_page(&nav());
        // JS file is referenced
        assert!(html.contains("wallet.js"));
        // Config is embedded
        assert!(html.contains("MNS_WALLET"));
        assert!(html.contains("rpcUrl"));
        assert!(html.contains("chainId"));
        // localStorage keys (in the JS file, but referenced via wallet.js)
        assert!(html.contains("wallet.js"));
        // No old wallet functions in HTML
        assert!(!html.contains("generate_mnemonic"));
        assert!(!html.contains("validate_mnemonic"));
        assert!(!html.contains("derive_keys("));
    }

    #[test]
    fn renders_home_page() {
        let html = render_home_page(&nav());
        assert!(html.contains("Mlkut Name System"));
        assert!(html.contains("search-form"));
        assert!(html.contains("main.css"));
        assert!(html.contains("home.css"));
    }

    #[test]
    fn renders_error_page() {
        let html = render_error("test error", &nav());
        assert!(html.contains("test error"));
        assert!(html.contains("error.css"));
    }

    #[test]
    fn renders_owners_page() {
        let items = vec![OwnerItemSimple {
            name_or_addr: "0x1234567890abcdef1234567890abcdef12345678".into(),
        }];
        let html = render_owners_page(&items, &nav());
        assert!(html.contains("Owners"));
        assert!(html.contains("0x12345678…12345678"));
    }
}
