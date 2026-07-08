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
import {{ generateMnemonic, validateMnemonic, mnemonicToSeed }} from 'https://esm.sh/@scure/bip39@1.3.0';
import {{ wordlist }} from 'https://esm.sh/@scure/bip39@1.3.0/wordlists/english';
import {{ HDKey }} from 'https://esm.sh/@scure/bip32@1.4.0';
import {{ secp256k1 }} from 'https://esm.sh/@noble/curves@1.4.0/secp256k1';
import {{ ed25519 }} from 'https://esm.sh/@noble/curves@1.4.0/ed25519';
import {{ keccak_256 }} from 'https://esm.sh/@noble/hashes@1.4.0/sha3';
import {{ sha256 }} from 'https://esm.sh/@noble/hashes@1.4.0/sha256';
import {{ bytesToHex, concatBytes, utf8ToBytes }} from 'https://esm.sh/@noble/hashes@1.4.0/utils';

const CHAIN_ID = {chain_id};
const RPC_URL = '{rpc_url}';
const RSK_COIN = CHAIN_ID === 31 ? 37310 : 137;
const RSK_PATH = `m/44'/${{RSK_COIN}}'/0'/0/0`;
const ZSK_DOMAIN = 'mns-zsk';
const USERNAME = 'mns-wallet';

// in-memory session keys (also persisted to localStorage for page-reload)
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

function toChecksumRsk(addrHex, chainId) {{
  // RSKIP-60: checksum uses chainId prefix
  const addr = addrHex.toLowerCase().replace('0x', '');
  const prefix = chainId ? chainId.toString() + '0x' : '';
  const hash = bytesToHex(keccak_256(utf8ToBytes(prefix + addr)));
  let out = '0x';
  for (let i = 0; i < addr.length; i++) {{
    out += parseInt(hash[i], 16) >= 8 ? addr[i].toUpperCase() : addr[i];
  }}
  return out;
}}

async function deriveFromMnemonic(mnemonic) {{
  const seed = await mnemonicToSeed(mnemonic);
  // Rootstock (secp256k1) via BIP44
  const hd = HDKey.fromMasterSeed(seed);
  const child = hd.derive(RSK_PATH);
  const pub = secp256k1.getPublicKey(child.privateKey, false); // 65 bytes uncompressed
  const addrBytes = keccak_256(pub.slice(1)).slice(-20);
  const address = toChecksumRsk(bytesToHex(addrBytes), CHAIN_ID);
  // ZSK (ed25519) from seed + domain
  const zskSeed = sha256(concatBytes(new Uint8Array(seed), utf8ToBytes(ZSK_DOMAIN)));
  const zskPub = ed25519.getPublicKey(zskSeed);
  return {{
    mnemonic,
    address,
    rskPrivKey: child.privateKey,
    zskSeed,
    zskPub: '0x' + bytesToHex(zskPub),
  }};
}}

async function fetchBalance(address) {{
  try {{
    var r=await fetch(RPC_URL,{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify({{jsonrpc:'2.0',method:'eth_getBalance',params:[address,'latest'],id:1}})}})
    var d=await r.json()
    if(d.result){{var w=BigInt(d.result);return(Number(w)/1e18).toFixed(6)+' RBTC'}}
    return '—'
  }}catch(e){{return '—'}}
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
  els.outZsk.textContent = data.zskPub;
  els.backupWarn.style.display = isNew ? 'block' : 'none';
  const faucet = {faucet_js};
  if (faucet) {{
    els.faucetNote.innerHTML = 'Fund this address with test tRBTC from the <a href="' + faucet + '" target="_blank" rel="noopener">faucet</a> to register names.';
  }}
  localStorage.setItem('mns-wallet-addr', data.address);
  localStorage.setItem('mns-wallet-mnemonic', data.mnemonic);
  fetchBalance(data.address).then(function(b){{els.outBalance.textContent=b}})
}}

function showLocked() {{
  session = null;
  autoUnlockDone = true; // explicit lock: don't let autofill silently re-unlock
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

// Save to browser credential store if supported (Chromium). Otherwise the
// <form> submit lets the native password manager offer to save.
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

// Shared unlock path used by manual submit, credential API, and autofill.
async function unlock(rawMnemonic, isNew) {{
  if (autoUnlockDone) return false;
  const mnemonic = (rawMnemonic || '').trim().replace(/\s+/g, ' ');
  if (!mnemonic || !validateMnemonic(mnemonic, wordlist)) return false;
  autoUnlockDone = true;
  const data = await deriveFromMnemonic(mnemonic);
  showUnlocked(data, isNew === true);
  return true;
}}

// Retrieve a saved credential from the Credential Management API (Chromium).
// `mediation` = 'silent' returns a credential only if the user previously
// granted access (no UI); 'optional' shows the account chooser once.
async function getSavedCredential(mediation) {{
  if (!navigator.credentials || !('PasswordCredential' in window)) return null;
  try {{
    return await navigator.credentials.get({{ password: true, mediation }});
  }} catch (e) {{
    return null;
  }}
}}

// Auto-unlock strategy that works across all browsers:
//  1. Chromium: silent PasswordCredential retrieval (instant, no user gesture).
//     Returns null until the user has picked the credential once — in that
//     case we fall through to native autofill rather than nagging with a chooser.
//  2. Chrome/Firefox: browser autofills the <textarea autocomplete> at load —
//     poll briefly until a value appears.
//  3. Safari: only fills after the user's first interaction (Touch/Face ID),
//     so also re-check on the first click.
async function attemptAutoUnlock() {{
  // 0. Restore from localStorage (persistent across page loads).
  var saved = localStorage.getItem('mns-wallet-mnemonic');
  if (saved && await unlock(saved, false)) return;

  // 1. Silent Credential Management API (Chromium, only if already granted).
  const cred = await getSavedCredential('silent');
  if (cred && cred.password && await unlock(cred.password, false)) return;

  // 2. Poll for browser form autofill (Chrome/Firefox fill on DOM ready).
  const started = Date.now();
  const poll = setInterval(function() {{
    if (autoUnlockDone || Date.now() - started > 3000) {{ clearInterval(poll); return; }}
    if (els.mnemonic.value.trim()) {{ clearInterval(poll); unlock(els.mnemonic.value, false); }}
  }}, 120);

  // 3. Safari fills on first user interaction — re-check then.
  document.addEventListener('click', function onFirstClick() {{
    document.removeEventListener('click', onFirstClick);
    if (!autoUnlockDone && els.mnemonic.value.trim()) unlock(els.mnemonic.value, false);
  }}, {{ once: true }});
}}


els.form.addEventListener('submit', async function(e) {{
  e.preventDefault();
  let raw = els.mnemonic.value.trim().replace(/\s+/g, ' ');
  // Empty field + user gesture: offer the saved-credential chooser (Chromium).
  // After the user picks once, future silent auto-unlocks will succeed.
  if (!raw) {{
    const cred = await getSavedCredential('optional');
    if (cred && cred.password) {{
      raw = cred.password.trim().replace(/\s+/g, ' ');
      els.mnemonic.value = raw;
    }}
  }}
  if (!raw) {{ els.msg.textContent = 'Enter a seed phrase or generate one.'; return; }}
  if (!validateMnemonic(raw, wordlist)) {{ els.msg.textContent = 'Invalid seed phrase.'; return; }}
  els.msg.textContent = 'Deriving…';
  try {{
    autoUnlockDone = false; // allow manual submit to take over
    await unlock(raw, false);
    await saveCredential(raw);
  }} catch (err) {{
    els.msg.textContent = 'Error: ' + err.message;
  }}
}});

els.generate.addEventListener('click', async function() {{
  const mnemonic = generateMnemonic(wordlist, 128); // 12 words
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

document.querySelectorAll('.wc-copy').forEach(function(btn) {{
  btn.addEventListener('click', function() {{
    if (!session) return;
    const key = btn.getAttribute('data-copy');
    const val = key === 'mnemonic' ? session.mnemonic
      : key === 'address' ? session.address
      : session.zskPub;
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
        assert!(html.contains("37310")); // testnet coin type (chain_id=31 in nav)
        assert!(html.contains("mns-zsk"));
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
