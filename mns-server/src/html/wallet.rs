use super::{footer_html, main_style, navbar_html, page_head, particles_script, Navbar};

pub fn render_wallet_page(nav: &Navbar) -> String {
    let style = format!(
        r##"{main_style}

  .wallet-card {{ max-width: 480px; }}

  .wc-status {{
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-family: var(--mono);
    font-size: 0.75rem;
    color: var(--fg-muted);
    margin-bottom: 1.25rem;
  }}
  .wc-status .dot {{
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--fg-dim);
  }}
  .wc-status.unlocked .dot {{ background: #22c55e; }}
  .wc-status.locked .dot {{ background: #d4a060; }}

  .wc-form {{
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }}

  .wc-field {{
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }}
  .wc-field label {{
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-muted);
  }}
  .wc-input {{
    padding: 0.7rem 0.85rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--fg);
    font-family: var(--mono);
    font-size: 0.82rem;
    outline: none;
    width: 100%;
    transition: border-color 0.2s;
  }}
  .wc-input:focus {{ border-color: var(--accent); }}
  textarea.wc-input {{ resize: vertical; min-height: 68px; line-height: 1.5; }}

  .wc-actions {{
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }}
  .wc-btn-primary, .wc-btn-ghost {{
    flex: 1;
    min-width: 120px;
    padding: 0.7rem 1rem;
    border-radius: var(--radius-sm);
    font-family: var(--sans);
    font-weight: 600;
    font-size: 0.82rem;
    cursor: pointer;
    transition: opacity 0.2s, border-color 0.2s, background 0.2s, color 0.2s;
  }}
  .wc-btn-primary {{
    background: var(--accent);
    color: #fff;
    border: none;
  }}
  .wc-btn-primary:hover {{ opacity: 0.88; }}
  .wc-btn-ghost {{
    background: none;
    color: var(--fg-muted);
    border: 1px solid var(--border);
  }}
  .wc-btn-ghost:hover {{ color: var(--fg); border-color: var(--accent); }}

  .wc-identity {{
    display: none;
    flex-direction: column;
    gap: 0.85rem;
  }}
  .wc-identity.show {{ display: flex; }}
  .wc-form.hide {{ display: none; }}

  .wc-key-row {{
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding: 0.75rem 0.9rem;
    background: rgba(255,255,255,0.015);
    border: 1px solid rgba(255,255,255,0.05);
    border-radius: var(--radius-sm);
  }}
  .wc-key-label {{
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.66rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-muted);
  }}
  .wc-key-value {{
    font-family: var(--mono);
    font-size: 0.76rem;
    color: var(--fg);
    word-break: break-all;
    line-height: 1.5;
  }}
  .wc-copy {{
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-xs);
    color: var(--fg-muted);
    cursor: pointer;
    font-family: var(--mono);
    font-size: 0.62rem;
    padding: 1px 6px;
    transition: color 0.15s, border-color 0.15s;
  }}
  .wc-copy:hover {{ color: var(--fg); border-color: var(--accent); }}
  .wc-copy.done {{ color: var(--accent); border-color: var(--accent); }}

  .wc-note {{
    font-size: 0.72rem;
    color: var(--fg-muted);
    line-height: 1.5;
    text-align: center;
  }}
  .wc-note a {{ color: var(--accent-text); }}

  .wc-warn {{
    font-size: 0.72rem;
    color: #d4a060;
    line-height: 1.5;
    padding: 0.6rem 0.8rem;
    background: rgba(212,160,96,0.08);
    border: 1px solid rgba(212,160,96,0.2);
    border-radius: var(--radius-sm);
  }}

  .wc-msg {{
    font-size: 0.72rem;
    text-align: center;
    color: var(--accent-text);
    min-height: 1rem;
  }}

  .wc-btn-sm {{
    padding: 0.35rem 0.7rem;
    border-radius: var(--radius-sm);
    font-family: var(--sans);
    font-weight: 600;
    font-size: 0.72rem;
    cursor: pointer;
    border: 1px solid var(--border);
    background: none;
    color: var(--fg-muted);
    transition: color 0.15s, border-color 0.15s;
  }}
  .wc-btn-sm:hover {{ color: var(--fg); border-color: var(--accent); }}
  .wc-btn-sm:disabled {{ opacity: 0.4; cursor: default; }}

  .wc-batches {{ display: none; flex-direction: column; gap: 0.6rem; }}
  .wc-batches.show {{ display: flex; }}
  .wc-batches-title {{
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-muted);
    margin-bottom: 0.15rem;
  }}
  .wc-batch {{
    padding: 0.7rem 0.9rem;
    background: rgba(255,255,255,0.015);
    border: 1px solid rgba(255,255,255,0.05);
    border-radius: var(--radius-sm);
  }}
  .wc-batch-row {{
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    flex-wrap: wrap;
  }}
  .wc-batch-ordinal {{
    font-family: var(--mono);
    font-size: 0.76rem;
    color: var(--fg);
  }}
  .wc-batch-ns {{
    font-family: var(--mono);
    font-size: 0.68rem;
    color: var(--fg-muted);
  }}
  .wc-batch-zsk {{
    font-family: var(--mono);
    font-size: 0.66rem;
    color: var(--fg-dim);
    word-break: break-all;
    width: 100%;
    margin-top: 0.3rem;
  }}
  .wc-batch-zsk .label {{ color: var(--fg-muted); }}
  .wc-batch-actions {{ margin-top: 0.45rem; }}
  .wc-batch-msg {{
    font-size: 0.68rem;
    color: var(--accent-text);
    margin-top: 0.3rem;
    min-height: 0.9rem;
    word-break: break-all;
  }}
"##,
        main_style = main_style()
    );
    let head = page_head("Wallet — MNS", &style);
    let particles = particles_script();
    let footer = footer_html();
    let nav_html = navbar_html(nav);
    let chain_id = nav.chain_id;
    let rpc_url = &nav.rpc_url;
    let faucet = if chain_id == 31 {
        "https://faucet.rootstock.io/"
    } else {
        ""
    };

    format!(
        r#"{head}
<body>

{nav_html}

<div class="grid-bg" aria-hidden="true"></div>
<div class="particles" id="particles" aria-hidden="true"></div>

<main>
<section class="card wallet-card">
  <header class="header">
    <h1>Your Wallet</h1>
  </header>

  <div class="wc-status locked" id="wc-status">
    <span class="dot"></span>
    <span id="wc-status-text">Locked</span>
  </div>

  <form class="wc-form" id="wc-form" autocomplete="on">
    <div class="wc-field" id="wc-username-field" style="display:none">
      <label for="wc-username">Account name</label>
      <input type="text" class="wc-input" id="wc-username" value=""
             autocomplete="off" placeholder="e.g. personal, work, testing">
    </div>

    <div class="wc-field" id="wc-mnemonic-field" style="display:none">
      <label for="wc-mnemonic">Old seed phrase (migration)</label>
      <textarea class="wc-input" name="password" id="wc-mnemonic"
                autocomplete="current-password"
                placeholder="Enter your old 12-word seed phrase" spellcheck="false"></textarea>
    </div>

    <div class="wc-actions">
      <button type="submit" class="wc-btn-primary" id="wc-unlock">Unlock</button>
      <button type="button" class="wc-btn-ghost" id="wc-generate">New Account</button>
    </div>
    <p class="wc-msg" id="wc-msg"></p>
  </form>

  <div class="wc-identity" id="wc-identity">
    <div class="wc-warn" id="wc-backup-warn" style="display:none">
      Your keys have been saved to your browser's password manager.
      They will be requested when you sign a transaction.
    </div>

    <div class="wc-key-row">
      <div class="wc-key-label"><span>Account</span></div>
      <div class="wc-key-value" id="out-username"></div>
    </div>

    <div class="wc-key-row">
      <div class="wc-key-label">
        <span>Rootstock address</span>
        <button class="wc-copy" data-copy="address">copy</button>
      </div>
      <div class="wc-key-value" id="out-address"></div>
    </div>

    <div class="wc-key-row">
      <div class="wc-key-label"><span>RBTC balance</span></div>
      <div class="wc-key-value" id="out-balance">—</div>
    </div>

    <div class="wc-key-row">
      <div class="wc-key-label">
        <span>ZSK</span>
        <button class="wc-copy" data-copy="zsk">copy</button>
      </div>
      <div class="wc-key-value" id="out-zsk"></div>
    </div>

    <div class="divider" role="separator" style="margin:0.25rem 0 0.75rem"></div>

    <div class="wc-field">
      <label for="wc-ns">Name server (NS) for new batch</label>
      <div class="wc-actions">
        <input type="text" class="wc-input" id="wc-ns" placeholder="ns1.example.com" spellcheck="false" style="flex:1;min-width:0">
        <button type="button" class="wc-btn-primary" id="wc-register">Register</button>
      </div>
    </div>
    <p class="wc-register-msg" id="wc-register-msg" style="font-size:0.72rem;text-align:center;color:var(--accent-text);min-height:1rem"></p>

    <div class="divider" role="separator" style="margin:0.25rem 0 0.75rem"></div>

    <div class="wc-batches" id="wc-batches">
      <div class="wc-batches-title">Your Batches</div>
      <div id="wc-batch-list"></div>
    </div>

    <p class="wc-note" id="wc-faucet-note"></p>

    <div class="wc-actions">
      <button type="button" class="wc-btn-ghost" id="wc-lock">Lock</button>
    </div>
  </div>

  <p class="wc-note" style="margin-top:1rem">
    Keys are derived in your browser and never sent to the server.
  </p>
</section>

<nav class="back-wrap" aria-label="Breadcrumb">
  <a class="back-link" href="/">← Home</a>
</nav>
</main>

{footer}

{particles}

<script type="module">
import init, {{ init_client, generate_wallet, derive_wallet_from_hex, key_to_mnemonic, migrate_from_mnemonic, get_balance, register as wasm_register, update_batch as wasm_update_batch }} from '/static/mns-wasm/mns_wasm.js';
await init();
init_client('{rpc_url}');

const CHAIN_ID = {chain_id};

let session = null;

const els = {{
  form: document.getElementById('wc-form'),
  identity: document.getElementById('wc-identity'),
  usernameField: document.getElementById('wc-username-field'),
  username: document.getElementById('wc-username'),
  mnemonic: document.getElementById('wc-mnemonic'),
  mnemonicField: document.getElementById('wc-mnemonic-field'),
  msg: document.getElementById('wc-msg'),
  status: document.getElementById('wc-status'),
  statusText: document.getElementById('wc-status-text'),
  outUsername: document.getElementById('out-username'),
  outAddress: document.getElementById('out-address'),
  outBalance: document.getElementById('out-balance'),
  outZsk: document.getElementById('out-zsk'),
  backupWarn: document.getElementById('wc-backup-warn'),
  faucetNote: document.getElementById('wc-faucet-note'),
  generate: document.getElementById('wc-generate'),
  lock: document.getElementById('wc-lock'),
}};

// ── localStorage helpers ──

function loadStored() {{
  const username = localStorage.getItem('mns-wallet-username');
  const addr = localStorage.getItem('mns-wallet-addr');
  const zsk = localStorage.getItem('mns-wallet-zsk');
  if (username && addr && zsk) return {{ username, address: addr, zsk_commitment_hex: zsk }};
  return null;
}}

function saveStored(data) {{
  localStorage.setItem('mns-wallet-username', data.username);
  localStorage.setItem('mns-wallet-addr', data.address);
  localStorage.setItem('mns-wallet-zsk', data.zsk_commitment_hex);
}}

function clearStored() {{
  localStorage.removeItem('mns-wallet-username');
  localStorage.removeItem('mns-wallet-addr');
  localStorage.removeItem('mns-wallet-zsk');
}}

// ── Credential helpers ──

async function storeCredential(username, rskHex, edHex) {{
  if (!('PasswordCredential' in window)) return false;
  try {{
    const cred = new window.PasswordCredential({{
      id: username,
      password: rskHex + '\\n' + edHex,
      name: 'MNS Wallet — ' + username,
    }});
    await navigator.credentials.store(cred);
    return true;
  }} catch {{ return false; }}
}}

async function getCredential(mediation) {{
  if (!navigator.credentials || !('PasswordCredential' in window)) return null;
  try {{
    return await navigator.credentials.get({{
      password: true,
      mediation: mediation || 'optional',
    }});
  }} catch {{ return null; }}
}}

function parseCredentialPassword(password) {{
  if (!password) return null;
  const parts = password.split('\\n');
  if (parts.length !== 2 || !parts[0] || !parts[1]) return null;
  return {{ rskHex: parts[0], edHex: parts[1] }};
}}

// ── RPC helpers ──

async function fetchBalance(address) {{
  try {{ return await get_balance(address); }} catch {{ return '—'; }}
}}

async function fetchBatches(address) {{
  try {{
    var r = await fetch('/api/batches/' + address);
    if (!r.ok) return;
    var batches = await r.json();
    var container = document.getElementById('wc-batches');
    var list = document.getElementById('wc-batch-list');
    if (!batches.length) {{ container.classList.remove('show'); return; }}
    container.classList.add('show');
    list.innerHTML = '';
    for (var i = 0; i < batches.length; i++) {{
      var b = batches[i];
      var div = document.createElement('div');
      div.className = 'wc-batch';
      var zskShort = b.zsk.length > 18 ? b.zsk.slice(0, 10) + '…' + b.zsk.slice(-6) : b.zsk;
      div.innerHTML = ''
        + '<div class="wc-batch-row">'
        + '  <span class="wc-batch-ordinal">Batch #' + b.ordinal + '</span>'
        + '  <span class="wc-batch-ns">' + b.ns + '</span>'
        + '</div>'
        + '<div class="wc-batch-zsk"><span class="label">ZSK:</span> ' + zskShort + '</div>'
        + '<div class="wc-batch-actions">'
        + '  <button class="wc-btn-sm wc-batch-update" data-ordinal="' + b.ordinal + '" data-ns="' + b.ns + '">Update ZSK</button>'
        + '</div>'
        + '<div class="wc-batch-msg" id="wc-batch-msg-' + b.ordinal + '"></div>';
      list.appendChild(div);
    }}
  }} catch {{}}
}}

document.getElementById('wc-batch-list').addEventListener('click', function(e) {{
  if (e.target.classList.contains('wc-batch-update')) {{
    if (!session) return;
    doUpdateBatch(parseInt(e.target.dataset.ordinal), e.target.dataset.ns);
  }}
}});

// ── Signing (prompt for credentials on every transaction) ──

async function signTx(fn) {{
  var cred = await getCredential('optional');
  if (!cred || !cred.password) return 'No credentials found.';
  var keys = parseCredentialPassword(cred.password);
  if (!keys) return 'Invalid stored credentials.';
  try {{
    var derived = derive_wallet_from_hex(keys.rskHex, keys.edHex);
    if (derived[0] !== session.address) return 'Credential does not match this account.';
    return await fn(keys.rskHex, keys.edHex);
  }} finally {{
    keys = null;
  }}
}}

async function doRegister(ns) {{
  var msgEl = document.getElementById('wc-register-msg');
  msgEl.textContent = 'Preparing…';
  try {{
    var err = await signTx(async function(rskHex, edHex) {{
      var txHash = await wasm_register(rskHex, session.zsk_commitment_hex, ns);
      msgEl.innerHTML = 'Tx sent: <a href="' + ('https://explorer.testnet.rootstock.io/tx/' + txHash) + '" target="_blank" rel="noopener">' + txHash.slice(0, 14) + '…</a>';
      setTimeout(function() {{ fetchBatches(session.address); }}, 15000);
      return null;
    }});
    if (err) msgEl.textContent = err;
  }} catch (e) {{ msgEl.textContent = 'Error: ' + e.message; }}
}}

async function doUpdateBatch(ordinal, currentNs) {{
  var msgEl = document.getElementById('wc-batch-msg-' + ordinal);
  msgEl.textContent = 'Preparing…';
  try {{
    var err = await signTx(async function(rskHex, edHex) {{
      var txHash = await wasm_update_batch(rskHex, BigInt(ordinal), session.address, session.zsk_commitment_hex, currentNs);
      msgEl.innerHTML = 'Tx sent: <a href="' + ('https://explorer.testnet.rootstock.io/tx/' + txHash) + '" target="_blank" rel="noopener">' + txHash.slice(0, 14) + '…</a>';
      setTimeout(function() {{ fetchBatches(session.address); }}, 15000);
      return null;
    }});
    if (err) msgEl.textContent = err;
  }} catch (e) {{ msgEl.textContent = 'Error: ' + e.message; }}
}}

// ── UI state ──

function showUnlocked(data, isNew) {{
  session = {{ username: data.username, address: data.address, zsk_commitment_hex: data.zsk_commitment_hex }};
  els.form.classList.add('hide');
  els.identity.classList.add('show');
  els.status.classList.remove('locked');
  els.status.classList.add('unlocked');
  els.statusText.textContent = 'Unlocked';
  els.outUsername.textContent = data.username;
  els.outAddress.textContent = data.address;
  els.outZsk.textContent = data.zsk_commitment_hex;
  els.backupWarn.style.display = isNew ? 'block' : 'none';
  const faucet = {faucet_js};
  if (faucet) {{
    els.faucetNote.innerHTML = 'Fund this address with test tRBTC from the <a href="' + faucet + '" target="_blank" rel="noopener">faucet</a> to register names.';
  }}
  saveStored(data);
  fetchBalance(data.address).then(function(b) {{ els.outBalance.textContent = b; }});
  fetchBatches(data.address);
}}

function showLocked() {{
  session = null;
  els.form.classList.remove('hide');
  els.identity.classList.remove('show');
  els.status.classList.remove('unlocked');
  els.status.classList.add('locked');
  els.statusText.textContent = 'Locked';
  els.msg.textContent = '';
  els.outBalance.textContent = '—';
  els.mnemonic.value = '';
  els.mnemonicField.style.display = 'none';
  els.username.value = '';
  els.usernameField.style.display = 'none';
  clearStored();
}}

// ── Unlock / Generate flows ──

async function unlockWithCredential() {{
  els.msg.textContent = 'Unlocking…';
  var cred = await getCredential('optional');
  if (!cred || !cred.password) {{
    els.msg.textContent = 'No credentials found. Generate a new account instead.';
    return;
  }}
  const keys = parseCredentialPassword(cred.password);
  if (!keys) {{ els.msg.textContent = 'Invalid stored credentials.'; return; }}
  const username = cred.id || 'wallet';
  try {{
    const result = derive_wallet_from_hex(keys.rskHex, keys.edHex);
    showUnlocked({{
      username: username,
      address: result[0],
      zsk_commitment_hex: result[1],
    }}, false);
  }} catch (e) {{
    els.msg.textContent = 'Derivation failed: ' + e.message;
  }}
}}

els.form.addEventListener('submit', async function(e) {{
  e.preventDefault();

  // Migration: mnemonic visible
  const mnemonic = els.mnemonic.value.trim().replace(/\\s+/g, ' ');
  if (mnemonic) {{
    const username = els.username.value.trim();
    if (!username) {{ els.msg.textContent = 'Enter a username for this account.'; return; }}
    els.msg.textContent = 'Migrating…';
    try {{
      const result = migrate_from_mnemonic(mnemonic);
      const rskHex = result[0];
      const edHex = result[1];
      await storeCredential(username, rskHex, edHex);
      showUnlocked({{
        username: username,
        address: result[2],
        zsk_commitment_hex: result[3],
      }}, true);
      localStorage.removeItem('mns-wallet-mnemonic');
    }} catch (err) {{
      els.msg.textContent = 'Migration failed: ' + err.message;
    }}
    return;
  }}

  // Generate: username field visible (shown by New Account button)
  if (els.usernameField.style.display !== 'none') {{
    const username = els.username.value.trim();
    if (!username) {{ els.msg.textContent = 'Enter a username for this account.'; return; }}
    els.msg.textContent = 'Generating…';
    try {{
      const keys = generate_wallet();
      const rskHex = keys.rsk_privkey;
      const edHex = keys.ed_privkey;
      await storeCredential(username, rskHex, edHex);
      showUnlocked({{
        username: username,
        address: keys.address,
        zsk_commitment_hex: keys.zsk_commitment,
      }}, true);
    }} catch (err) {{
      els.msg.textContent = 'Error: ' + err.message;
    }}
    return;
  }}

  // Unlock: just trigger credential picker
  await unlockWithCredential();
}});

els.generate.addEventListener('click', function() {{
  // Show username field so user can name this account, then submit to generate
  els.usernameField.style.display = '';
  els.username.focus();
  els.msg.textContent = 'Enter a name for this account, then click New Account again.';
}});

els.lock.addEventListener('click', showLocked);

document.getElementById('wc-register').addEventListener('click', function() {{
  if (!session) return;
  var ns = document.getElementById('wc-ns').value.trim();
  if (!ns) {{ document.getElementById('wc-register-msg').textContent = 'Enter a name server'; return; }}
  doRegister(ns);
}});

// ── Copy buttons ──

document.querySelectorAll('.wc-copy').forEach(function(btn) {{
  btn.addEventListener('click', function() {{
    if (!session) return;
    const key = btn.getAttribute('data-copy');
    const val = key === 'address' ? session.address
      : key === 'zsk' ? session.zsk_commitment_hex
      : '';
    if (!val) return;
    navigator.clipboard.writeText(val).then(function() {{
      btn.classList.add('done');
      btn.textContent = 'copied';
      setTimeout(function() {{ btn.classList.remove('done'); btn.textContent = 'copy'; }}, 1200);
    }}).catch(function() {{}});
  }});
}});

// ── Auto-unlock on page load ──

(function attemptAutoUnlock() {{
  // 1. Check for old mnemonic (migration path)
  var oldMnemonic = localStorage.getItem('mns-wallet-mnemonic');
  if (oldMnemonic) {{
    els.usernameField.style.display = '';
    els.mnemonicField.style.display = '';
    els.mnemonic.value = oldMnemonic;
    els.msg.textContent = 'Your old wallet was found. Enter a username to migrate.';
    return;
  }}

  // 2. Check localStorage for existing wallet
  var stored = loadStored();
  if (stored) {{
    showUnlocked({{
      username: stored.username,
      address: stored.address,
      zsk_commitment_hex: stored.zsk_commitment_hex,
    }}, false);
    return;
  }}

  // 3. Empty state — just show the form with Unlock / New Account buttons
}})();
</script>

</body>
</html>"#,
        faucet_js = if faucet.is_empty() {
            "''".to_string()
        } else {
            format!("'{faucet}'")
        },
        chain_id = chain_id,
        rpc_url = rpc_url,
    )
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
        assert!(html.contains("mns_wasm.js"));
        assert!(!html.contains("{main_style}"));
        assert!(!html.contains("{faucet_js}"));
        assert!(!html.contains("{chain_id}"));
    }

    #[test]
    fn has_new_wallet_lifecycle() {
        let html = render_wallet_page(&nav());
        // New WASM imports
        assert!(html.contains("generate_wallet"));
        assert!(html.contains("derive_wallet_from_hex"));
        assert!(html.contains("migrate_from_mnemonic"));
        // localStorage keys
        assert!(html.contains("mns-wallet-username"));
        assert!(html.contains("mns-wallet-addr"));
        assert!(html.contains("mns-wallet-zsk"));
        // Old mnemonic key referenced for migration
        assert!(html.contains("mns-wallet-mnemonic"));
        // Credential helpers
        assert!(html.contains("storeCredential"));
        assert!(html.contains("getCredential"));
        // Auto-unlock
        assert!(html.contains("attemptAutoUnlock"));
        // No old wallet functions
        assert!(!html.contains("generate_mnemonic"));
        assert!(!html.contains("validate_mnemonic"));
        assert!(!html.contains("derive_keys("));
    }
}
