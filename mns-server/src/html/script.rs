pub fn particles_script() -> String {
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

pub fn wallet_script(chain_id: u64, rpc_url: &str) -> String {
    let chain_id_hex = format!("0x{:x}", chain_id);
    let chain_name = if chain_id == 30 {
        "Rootstock Mainnet"
    } else {
        "Rootstock Testnet"
    };
    let currency = if chain_id == 30 { "RBTC" } else { "tRBTC" };
    format!(
        r#"<script>
(function() {{
  var el = document.getElementById('wallet-connect');
  if (!el) return;
  var KEY = 'mns-account';

  var CHAIN_ID = '{chain_id_hex}';
  var CHAIN_PARAMS = {{
    chainId: '{chain_id_hex}',
    chainName: '{chain_name}',
    nativeCurrency: {{ name: '{currency}', symbol: '{currency}', decimals: 18 }},
    rpcUrls: ['{rpc_url}']
  }};

  function truncate(addr) {{
    return addr.slice(0, 6) + '…' + addr.slice(-4);
  }}

  function switchChain() {{
    return window.ethereum.request({{ method: 'wallet_switchEthereumChain', params: [{{ chainId: CHAIN_ID }}] }})
      .catch(function(e) {{
        if (e.code === 4001) throw e;
        return window.ethereum.request({{ method: 'wallet_addEthereumChain', params: [CHAIN_PARAMS] }});
      }});
  }}

  function showConnected(account) {{
    try {{ sessionStorage.setItem(KEY, account); }} catch(e) {{}}
    el.innerHTML =
      '<a href="/owner/' + account + '" class="wc-addr" title="' + account + '">' + truncate(account) + '</a>' +
      '<button class="wc-disc" id="wc-disc" aria-label="Disconnect wallet">' +
        '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
          '<path d="M18.36 6.64a9 9 0 1 1-12.73 0"/><line x1="12" y1="2" x2="12" y2="12"/>' +
        '</svg>' +
      '</button>';
    document.getElementById('wc-disc').onclick = function() {{
      try {{ sessionStorage.removeItem(KEY); }} catch(e) {{}}
      window.ethereum.request({{ method: 'wallet_revokePermissions', params: [{{ eth_accounts: {{}} }}] }}).catch(function() {{}});
      showDisconnected();
    }};
  }}

  function showDisconnected() {{
    try {{ sessionStorage.removeItem(KEY); }} catch(e) {{}}
    el.innerHTML =
      typeof window.ethereum === 'undefined'
        ? '<button class="wc-btn" disabled>No Wallet</button>'
        : '<button class="wc-btn" id="wc-btn">Connect</button>';
    var btn = document.getElementById('wc-btn');
    if (btn) {{
      btn.onclick = function() {{
        window.ethereum.request({{ method: 'eth_requestAccounts' }}).then(function(accounts) {{
          if (accounts.length > 0) {{
            return switchChain().then(function() {{ return accounts[0]; }});
          }}
        }}).then(function(account) {{
          if (account) showConnected(account);
        }}).catch(function(e) {{ console.error(e); }});
      }};
    }}
  }}

  var saved;
  try {{ saved = sessionStorage.getItem(KEY); }} catch(e) {{}}
  if (saved) {{
    showConnected(saved);
  }} else {{
    showDisconnected();
  }}

  if (typeof window.ethereum !== 'undefined' && window.ethereum.on) {{
    window.ethereum.on('accountsChanged', function(accounts) {{
      if (accounts.length > 0) showConnected(accounts[0]);
      else showDisconnected();
    }});
  }}
}})();
</script>"#,
        chain_id_hex = chain_id_hex,
        chain_name = chain_name,
        currency = currency,
        rpc_url = rpc_url,
    )
}
