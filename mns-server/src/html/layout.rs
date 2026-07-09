use super::{Navbar, FAVICON};

pub fn page_head(title: &str, style: &str) -> String {
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

pub fn footer_html() -> String {
    r#"<footer class="footer">
  <span class="footer-dot" aria-hidden="true"></span>
  MNS Resolver
</footer>"#
        .to_string()
}

pub fn navbar_html(nav: &Navbar) -> String {
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
        r#"<div style="text-align: center;background-color: #ffca00;width: 100%;position: fixed;top: 0;z-index: 101;color: black;font-size: 0.85rem;line-height: 1.3rem;"> WIP — breaking changes expected!</div>
<nav class="navbar" aria-label="Primary">
  <a href="/" class="navbar-logo" aria-label="MNS home">
    <img src="/static/mlkut.png" alt="MNS">
  </a>
  <div class="navbar-right">
    <a class="navbar-btn" href="{contract_url}" target="_blank" rel="noopener noreferrer" title="View contract on explorer">{network}</a>
    <a class="navbar-btn" href="{block_url}" target="_blank" rel="noopener noreferrer" title="view latest indexed block {block} on explorer">
      <span class="liveness-dot" aria-hidden="true"></span>
      <span class="block-number">{block}</span>
      <span class="block-ago" data-ago="{sync_time}"></span>
    </a>
    <a class="navbar-btn" href="/wallet" id="nav-wallet" title="Your MNS wallet">Wallet</a>
    <button type="button" class="theme-toggle" onclick="_toggleTheme()" aria-label="Switch theme">
      <svg class="theme-icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="5"/>
        <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
      </svg>
      <svg class="theme-icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
      </svg>
    </button>
  </div>
</nav>
<script>
(function(){{
window._ago=function(t){{if(!t)return'';var s=Math.floor((Date.now()/1e3-t));if(s<60)return s+'s';if(s<3600)return Math.floor(s/60)+'m';if(s<86400)return Math.floor(s/3600)+'h';return Math.floor(s/86400)+'d'}}
function tick(){{document.querySelectorAll('[data-ago]').forEach(function(el){{var t=el.textContent=window._ago(Number(el.getAttribute('data-ago')));if(t)el.style.display='inline'}})}}
tick();setInterval(tick,3e4);
var addr=localStorage.getItem('mns-wallet-addr');
if(addr){{var el=document.getElementById('nav-wallet');el.textContent=addr.slice(0,6)+'…'+addr.slice(-4);el.title=addr}}
}})()
</script>"#,
        network = nav.network,
        block = nav.sync_block,
        sync_time = nav.sync_time,
        block_url = block_url,
        contract_url = contract_url,
    )
}
