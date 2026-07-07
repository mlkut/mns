use super::{error_style, navbar_html, page_head, wallet_script, Navbar};

pub fn render_error(message: &str, nav: &Navbar) -> String {
    let style = error_style();
    let head = page_head("Error", &style);
    let wallet_script = wallet_script(nav.chain_id, &nav.rpc_url);
    let nav_html = navbar_html(nav);
    format!(
        r#"{head}
<body>
{nav_html}
<main>
<section class="card" role="alert"><p class="msg">{message}</p></section>
</main>
{wallet_script}
</body>
</html>"#,
    )
}
