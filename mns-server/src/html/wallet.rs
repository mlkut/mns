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

  .wc-seed-hidden {{ filter: blur(4px); cursor: pointer; transition: filter 0.25s; }}
  .wc-seed-hidden:hover {{ filter: blur(3px); }}
  .wc-seed-hidden.revealed {{ filter: none; }}

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
    let contract_address = &nav.contract_address;
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
    <input type="text" name="username" id="wc-username" value="mns-wallet"
           autocomplete="username" style="display:none" readonly>
    <div class="wc-field">
      <label for="wc-mnemonic">Seed phrase</label>
      <textarea class="wc-input" name="password" id="wc-mnemonic"
                autocomplete="current-password"
                placeholder="Enter your 12-word seed phrase, or generate a new wallet" spellcheck="false"></textarea>
    </div>
    <div class="wc-actions">
      <button type="submit" class="wc-btn-primary" id="wc-unlock">Unlock</button>
      <button type="button" class="wc-btn-ghost" id="wc-generate">Generate New</button>
    </div>
    <p class="wc-msg" id="wc-msg"></p>
  </form>

  <div class="wc-identity" id="wc-identity">
    <div class="wc-warn" id="wc-backup-warn" style="display:none">
      Write down your seed phrase and save it. It is the only way to recover this wallet.
      Your browser has been asked to save it to your password manager.
    </div>

    <div class="wc-key-row">
      <div class="wc-key-label">
        <span>Seed phrase</span>
        <button class="wc-copy" data-copy="mnemonic">copy</button>
      </div>
      <div class="wc-key-value" id="out-mnemonic"></div>
    </div>

    <div class="wc-key-row">
      <div class="wc-key-label">
        <span>Rootstock address</span>
        <button class="wc-copy" data-copy="address">copy</button>
      </div>
      <div class="wc-key-value" id="out-address"></div>
    </div>

    <div class="wc-key-row">
      <div class="wc-key-label">
        <span>RBTC balance</span>
      </div>
      <div class="wc-key-value" id="out-balance">—</div>
    </div>

    <div class="wc-key-row">
      <div class="wc-key-label">
        <span>ZSK public key</span>
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
import init, {{ generate_mnemonic, validate_mnemonic, derive_keys, register as wasm_register, update_batch as wasm_update_batch }} from '/static/mns-wasm/mns_wasm.js';
await init();

const CHAIN_ID = {chain_id};
const RPC_URL = '{rpc_url}';
const CONTRACT = '{contract_address}';
const USERNAME = 'mns-wallet';

let session = null;
let autoUnlockDone = false;

const els = {{
  form: document.getElementById('wc-form'),
  identity: document.getElementById('wc-identity'),
  mnemonic: document.getElementById('wc-mnemonic'),
  msg: document.getElementById('wc-msg'),
  status: document.getElementById('wc-status'),
  statusText: document.getElementById('wc-status-text'),
  outMnemonic: document.getElementById('out-mnemonic'),
  outAddress: document.getElementById('out-address'),
  outBalance: document.getElementById('out-balance'),
  outZsk: document.getElementById('out-zsk'),
  backupWarn: document.getElementById('wc-backup-warn'),
  faucetNote: document.getElementById('wc-faucet-note'),
  generate: document.getElementById('wc-generate'),
  lock: document.getElementById('wc-lock'),
}};

async function fetchBalance(address) {{
  try {{
    var r=await fetch(RPC_URL,{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify({{jsonrpc:'2.0',method:'eth_getBalance',params:[address,'latest'],id:1}})}})
    var d=await r.json()
    if(d.result){{var w=BigInt(d.result);return(Number(w)/1e18).toFixed(6)+' RBTC'}}
    return '—'
  }}catch(e){{return '—'}}
}}

async function rpcCall(method,params) {{
  var r=await fetch(RPC_URL,{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify({{jsonrpc:'2.0',method,params,id:1}})}});
  var d=await r.json();
  if(d.error) throw new Error(d.error.message||JSON.stringify(d.error));
  return d.result;
}}

async function fetchBatches(address) {{
  try{{
    var r=await fetch('/api/batches/'+address);
    if(!r.ok)return;
    var batches=await r.json();
    var container=document.getElementById('wc-batches');
    var list=document.getElementById('wc-batch-list');
    if(!batches.length){{container.classList.remove('show');return}}
    container.classList.add('show');
    list.innerHTML='';
    for(var i=0;i<batches.length;i++){{
      var b=batches[i];
      var div=document.createElement('div');
      div.className='wc-batch';
      var zskShort=b.zsk.length>18?b.zsk.slice(0,10)+'…'+b.zsk.slice(-6):b.zsk;
      div.innerHTML=''
        +'<div class="wc-batch-row">'
        +'  <span class="wc-batch-ordinal">Batch #'+b.ordinal+'</span>'
        +'  <span class="wc-batch-ns">'+b.ns+'</span>'
        +'</div>'
        +'<div class="wc-batch-zsk"><span class="label">ZSK:</span> '+zskShort+'</div>'
        +'<div class="wc-batch-actions">'
        +'  <button class="wc-btn-sm wc-batch-update" data-ordinal="'+b.ordinal+'" data-ns="'+b.ns+'" data-zsk="'+b.zsk+'">Update ZSK</button>'
        +'</div>'
        +'<div class="wc-batch-msg" id="wc-batch-msg-'+b.ordinal+'"></div>';
      list.appendChild(div);
    }}
  }}catch(e){{}}
}}
document.getElementById('wc-batch-list').addEventListener('click',function(e){{
  if(e.target.classList.contains('wc-batch-update')){{
    if(!session)return;
    doUpdateBatch(parseInt(e.target.dataset.ordinal),e.target.dataset.ns,session);
  }}
}});

async function doRegister(ns, data) {{
  var msgEl=document.getElementById('wc-register-msg');
  msgEl.textContent='Preparing…';
  try{{
    var txHash=await wasm_register(RPC_URL,data.private_key_hex,data.zsk_commitment_hex,ns);
    msgEl.innerHTML='Tx sent: <a href="'+('https://explorer.testnet.rootstock.io/tx/'+txHash)+'" target="_blank" rel="noopener">'+txHash.slice(0,14)+'…</a>';
    setTimeout(function(){{fetchBatches(data.address)}},15000);
  }}catch(e){{msgEl.textContent='Error: '+e.message}}
}}

async function doUpdateBatch(ordinal, currentNs, data) {{
  var msgEl=document.getElementById('wc-batch-msg-'+ordinal);
  msgEl.textContent='Preparing…';
  try{{
    var txHash=await wasm_update_batch(RPC_URL,data.private_key_hex,ordinal,data.address,data.zsk_commitment_hex,currentNs);
    msgEl.innerHTML='Tx sent: <a href="'+('https://explorer.testnet.rootstock.io/tx/'+txHash)+'" target="_blank" rel="noopener">'+txHash.slice(0,14)+'…</a>';
    setTimeout(function(){{fetchBatches(data.address)}},15000);
  }}catch(e){{msgEl.textContent='Error: '+e.message}}
}}

function showUnlocked(data, isNew) {{
  session = data;
  els.form.classList.add('hide');
  els.identity.classList.add('show');
  els.status.classList.remove('locked');
  els.status.classList.add('unlocked');
  els.statusText.textContent = 'Unlocked';
  els.outMnemonic.textContent = data.mnemonic;
  els.outMnemonic.classList.add('wc-seed-hidden');
  els.outAddress.textContent = data.address;
  els.outZsk.textContent = '0x' + data.zsk_pub;
  els.backupWarn.style.display = isNew ? 'block' : 'none';
  const faucet = {faucet_js};
  if (faucet) {{
    els.faucetNote.innerHTML = 'Fund this address with test tRBTC from the <a href="' + faucet + '" target="_blank" rel="noopener">faucet</a> to register names.';
  }}
  localStorage.setItem('mns-wallet-addr', data.address);
  localStorage.setItem('mns-wallet-mnemonic', data.mnemonic);
  fetchBalance(data.address).then(function(b){{els.outBalance.textContent=b}});
  fetchBatches(data.address);
}}

function showLocked() {{
  session = null;
  autoUnlockDone = true;
  els.form.classList.remove('hide');
  els.identity.classList.remove('show');
  els.status.classList.remove('unlocked');
  els.status.classList.add('locked');
  els.statusText.textContent = 'Locked';
  els.mnemonic.value = '';
  els.msg.textContent = '';
  els.outBalance.textContent = '—';
  localStorage.removeItem('mns-wallet-addr');
  localStorage.removeItem('mns-wallet-mnemonic');
}}

async function saveCredential(mnemonic) {{
  if (!('PasswordCredential' in window)) return false;
  try {{
    const cred = new window.PasswordCredential({{
      id: USERNAME,
      password: mnemonic,
      name: 'MNS Wallet',
    }});
    await navigator.credentials.store(cred);
    return true;
  }} catch (e) {{
    return false;
  }}
}}

async function unlock(rawMnemonic, isNew) {{
  if (autoUnlockDone) return false;
  const mnemonic = (rawMnemonic || '').trim().replace(/\\s+/g, ' ');
  if (!mnemonic || !validate_mnemonic(mnemonic)) return false;
  autoUnlockDone = true;
  const keys = derive_keys(mnemonic);
  showUnlocked({{
    mnemonic,
    address: keys.address,
    private_key_hex: keys.private_key,
    zsk_pub: keys.zsk_pub,
    zsk_commitment_hex: keys.zsk_commitment,
  }}, isNew === true);
  return true;
}}

async function getSavedCredential(mediation) {{
  if (!navigator.credentials || !('PasswordCredential' in window)) return null;
  try {{
    return await navigator.credentials.get({{ password: true, mediation }});
  }} catch (e) {{
    return null;
  }}
}}

async function attemptAutoUnlock() {{
  var saved = localStorage.getItem('mns-wallet-mnemonic');
  if (saved && await unlock(saved, false)) return;

  const cred = await getSavedCredential('silent');
  if (cred && cred.password && await unlock(cred.password, false)) return;

  const started = Date.now();
  const poll = setInterval(function() {{
    if (autoUnlockDone || Date.now() - started > 3000) {{ clearInterval(poll); return; }}
    if (els.mnemonic.value.trim()) {{ clearInterval(poll); unlock(els.mnemonic.value, false); }}
  }}, 120);

  document.addEventListener('click', function onFirstClick() {{
    document.removeEventListener('click', onFirstClick);
    if (!autoUnlockDone && els.mnemonic.value.trim()) unlock(els.mnemonic.value, false);
  }}, {{ once: true }});
}}

els.form.addEventListener('submit', async function(e) {{
  e.preventDefault();
  let raw = els.mnemonic.value.trim().replace(/\\s+/g, ' ');
  if (!raw) {{
    const cred = await getSavedCredential('optional');
    if (cred && cred.password) {{
      raw = cred.password.trim().replace(/\\s+/g, ' ');
      els.mnemonic.value = raw;
    }}
  }}
  if (!raw) {{ els.msg.textContent = 'Enter a seed phrase or generate one.'; return; }}
  if (!validate_mnemonic(raw)) {{ els.msg.textContent = 'Invalid seed phrase.'; return; }}
  els.msg.textContent = 'Deriving…';
  try {{
    autoUnlockDone = false;
    await unlock(raw, false);
    await saveCredential(raw);
  }} catch (err) {{
    els.msg.textContent = 'Error: ' + err.message;
  }}
}});

els.generate.addEventListener('click', async function() {{
  const mnemonic = generate_mnemonic();
  els.mnemonic.value = mnemonic;
  els.msg.textContent = 'Deriving…';
  try {{
    autoUnlockDone = false;
    await unlock(mnemonic, true);
    await saveCredential(mnemonic);
  }} catch (err) {{
    els.msg.textContent = 'Error: ' + err.message;
  }}
}});

els.lock.addEventListener('click', showLocked);

document.getElementById('wc-register').addEventListener('click', function() {{
  if (!session) return;
  var ns=document.getElementById('wc-ns').value.trim();
  if(!ns){{document.getElementById('wc-register-msg').textContent='Enter a name server';return}}
  doRegister(ns, session);
}});

document.querySelectorAll('.wc-copy').forEach(function(btn) {{
  btn.addEventListener('click', function() {{
    if (!session) return;
    const key = btn.getAttribute('data-copy');
    const val = key === 'mnemonic' ? session.mnemonic
      : key === 'address' ? session.address
      : '0x' + session.zsk_pub;
    if (key === 'mnemonic') {{
      els.outMnemonic.classList.remove('wc-seed-hidden');
      els.outMnemonic.classList.add('revealed');
      setTimeout(function(){{els.outMnemonic.classList.remove('revealed');els.outMnemonic.classList.add('wc-seed-hidden')}}, 3000);
    }}
    navigator.clipboard.writeText(val).then(function() {{
      btn.classList.add('done');
      btn.textContent = 'copied';
      setTimeout(function() {{ btn.classList.remove('done'); btn.textContent = 'copy'; }}, 1200);
    }}).catch(function() {{}});
  }});
}});

els.outMnemonic.addEventListener('click', function() {{
  if (this.classList.contains('revealed')) {{
    this.classList.remove('revealed');
    this.classList.add('wc-seed-hidden');
  }} else {{
    this.classList.remove('wc-seed-hidden');
    this.classList.add('revealed');
    setTimeout(function(el){{el.classList.remove('revealed');el.classList.add('wc-seed-hidden')}}, 5000, this);
  }}
}});

attemptAutoUnlock();
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
        // no unresolved format placeholders left behind
        assert!(!html.contains("{main_style}"));
        assert!(!html.contains("{faucet_js}"));
        assert!(!html.contains("{chain_id}"));
    }

    #[test]
    fn has_cross_browser_auto_unlock() {
        let html = render_wallet_page(&nav());
        // Chromium credential retrieval: silent auto + optional chooser fallback
        assert!(html.contains("getSavedCredential('silent')"));
        assert!(html.contains("getSavedCredential('optional')"));
        // Firefox/Chrome autofill poll
        assert!(html.contains("setInterval"));
        // Safari first-interaction fallback
        assert!(html.contains("onFirstClick"));
        // shared entry point
        assert!(html.contains("attemptAutoUnlock()"));
    }
}
