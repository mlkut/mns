use mns::{Name, ZSK_LEN};
use simple_dns::ResourceRecord;

const FAVICON: &str = "/static/favicon.png";
const ACCENT: &str = "#800000";

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
    let secs = ts as i64;
    let Ok(dt) = time::OffsetDateTime::from_unix_timestamp(secs) else {
        return ts.to_string();
    };
    let dt = dt.to_offset(time::UtcOffset::UTC);
    dt.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| ts.to_string())
}

mod error;
mod home;
mod layout;
mod name;
mod owner;
mod script;
mod style;
mod wallet;

pub use error::render_error;
pub use home::render_home_page;
pub use layout::{footer_html, navbar_html, page_head};
pub use name::{render_html, render_not_found_page};
pub use owner::{render_owner_page, render_owners_page};
pub use script::particles_script;
pub use style::{error_style, main_style};
pub use wallet::render_wallet_page;
