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
const CONTRACT = '{contract_address}';
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

// ── ZSK commitment (SHA256(0x00 || pubkey)) ──
function zskCommitment(pubKey) {{
  var input = new Uint8Array(1 + pubKey.length);
  input[0] = 0x00;
  input.set(pubKey, 1);
  return sha256(input);
}}

function parseHex(h){{h=h.startsWith('0x')?h.slice(2):h;if(h.length%2)h='0'+h;var b=new Uint8Array(h.length/2);for(var i=0;i<b.length;i++)b[i]=parseInt(h.substr(i*2,2),16);return b}}

// ── RLP encoding helpers ──
function rlpEncodeBytes(bytes) {{
  if(bytes.length===1&&bytes[0]<0x80) return bytes;
  if(bytes.length<=55) return concatBytes(new Uint8Array([0x80+bytes.length]),bytes);
  var lenBytes=[],v=bytes.length;
  while(v>0){{lenBytes.unshift(v&0xff);v>>=8}}
  return concatBytes(new Uint8Array([0xb7+lenBytes.length]),new Uint8Array(lenBytes),bytes);
}}
function rlpEncodeList(items) {{
  var payload=new Uint8Array(0);
  for(var i=0;i<items.length;i++) payload=concatBytes(payload,items[i]);
  if(payload.length<=55) return concatBytes(new Uint8Array([0xc0+payload.length]),payload);
  var lenBytes=[],v=payload.length;
  while(v>0){{lenBytes.unshift(v&0xff);v>>=8}}
  return concatBytes(new Uint8Array([0xf7+lenBytes.length]),new Uint8Array(lenBytes),payload);
}}
function encodeU128(val) {{
  if(val===0) return rlpEncodeBytes(new Uint8Array(0));
  if(typeof val==='string') val=BigInt(val);
  else val=BigInt(val);
  var bytes=[];
  while(val>0n){{bytes.unshift(Number(val&0xffn));val>>=8n}}
  return rlpEncodeBytes(new Uint8Array(bytes));
}}
function padLeft32(bytes) {{
  if(bytes.length>=32) return bytes.slice(-32);
  var padded=new Uint8Array(32);
  padded.set(bytes,32-bytes.length);
  return padded;
}}

// ── Fetch nonce + gas price via RPC ──
async function rpcCall(method,params) {{
  var r=await fetch(RPC_URL,{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify({{jsonrpc:'2.0',method,params,id:1}})}});
  var d=await r.json();
  if(d.error) throw new Error(d.error.message||JSON.stringify(d.error));
  return d.result;
}}
async function fetchNonce(addr) {{
  var hex=await rpcCall('eth_getTransactionCount',[addr,'latest']);
  return BigInt(hex);
}}
async function fetchGasPrice() {{
  var hex=await rpcCall('eth_gasPrice',[]);
  return BigInt(hex);
}}

// ── Sign calldata, build EIP-155 tx, broadcast via RPC ──
async function signAndBroadcast(calldata, data) {{
  var nonce=await fetchNonce(data.address);
  var gasPrice=await fetchGasPrice();
  var gasLimit=300000;
  var to=parseHex(CONTRACT.replace('0x',''));
  var value=new Uint8Array(0);
  var unsignedTx=rlpEncodeList([
    encodeU128(nonce),encodeU128(gasPrice),encodeU128(gasLimit),
    rlpEncodeBytes(to),rlpEncodeBytes(value),rlpEncodeBytes(calldata),
    encodeU128(CHAIN_ID),rlpEncodeBytes(new Uint8Array(0)),
    rlpEncodeBytes(new Uint8Array(0))
  ]);
  var txHash=keccak_256(unsignedTx);
  var sig=secp256k1.sign(new Uint8Array(txHash),data.rskPrivKey);
  var raw64=sig.toCompactRawBytes();
  var rBytes=raw64.slice(0,32),sBytes=raw64.slice(32);
  var v=sig.recovery+35+CHAIN_ID*2;
  var signedTx=rlpEncodeList([
    encodeU128(nonce),encodeU128(gasPrice),encodeU128(gasLimit),
    rlpEncodeBytes(to),rlpEncodeBytes(value),rlpEncodeBytes(calldata),
    encodeU128(v),rlpEncodeBytes(rBytes),rlpEncodeBytes(sBytes)
  ]);
  return await rpcCall('eth_sendRawTransaction',['0x'+bytesToHex(signedTx)]);
}}

// ── ABI encoding for register(bytes32,string) ──
var REGISTER_SELECTOR=parseHex('cf2d31fb');
function encodeRegister(zskHex, ns) {{
  var zskBytes=padLeft32(parseHex(zskHex.replace('0x','')));
  var nsOffset=padLeft32(new Uint8Array([0x60]));
  var nsUtf8=utf8ToBytes(ns);
  var nsLen=padLeft32(new Uint8Array([nsUtf8.length&0xff]));
  var nsPadded=new Uint8Array(Math.ceil(nsUtf8.length/32)*32||32);
  nsPadded.set(nsUtf8);
  return concatBytes(REGISTER_SELECTOR,zskBytes,nsOffset,nsLen,nsPadded);
}}

// ── ABI encoding for updateBatch(uint64,address,bytes32,string) ──
var UPDATE_BATCH_SELECTOR=parseHex('697c209b');
function encodeUpdateBatch(ordinal, ownerHex, zskHex, ns) {{
  var ordinalBytes=padLeft32(new Uint8Array([
    (ordinal>>>24)&0xff,(ordinal>>>16)&0xff,(ordinal>>>8)&0xff,ordinal&0xff
  ]));
  var ownerBytes=padLeft32(parseHex(ownerHex.replace('0x','')));
  var zskBytes=padLeft32(parseHex(zskHex.replace('0x','')));
  var nsOffset=padLeft32(new Uint8Array([0x80]));
  var nsUtf8=utf8ToBytes(ns);
  var nsLen=padLeft32(new Uint8Array([nsUtf8.length&0xff]));
  var nsPadded=new Uint8Array(Math.ceil(nsUtf8.length/32)*32||32);
  nsPadded.set(nsUtf8);
  return concatBytes(UPDATE_BATCH_SELECTOR,ordinalBytes,ownerBytes,zskBytes,nsOffset,nsLen,nsPadded);
}}

// ── Register a new batch ──
async function doRegister(ns, data) {{
  var msgEl=document.getElementById('wc-register-msg');
  msgEl.textContent='Preparing…';
  try{{
    var zskCommit=zskCommitment(parseHex(data.zskPub.slice(2)));
    var calldata=encodeRegister('0x'+bytesToHex(zskCommit),ns);
    var txHash=await signAndBroadcast(calldata,data);
    msgEl.innerHTML='Tx sent: <a href="'+('https://explorer.testnet.rootstock.io/tx/'+txHash)+'" target="_blank" rel="noopener">'+txHash.slice(0,14)+'…</a>';
    setTimeout(function(){{fetchBatches(data.address)}},15000);
  }}catch(e){{msgEl.textContent='Error: '+e.message}}
}}

// ── Update batch ZSK ──
async function doUpdateBatch(ordinal, currentNs, currentZsk, data) {{
  var msgEl=document.getElementById('wc-batch-msg-'+ordinal);
  msgEl.textContent='Preparing…';
  try{{
    var zskCommit=zskCommitment(parseHex(data.zskPub.slice(2)));
    var calldata=encodeUpdateBatch(ordinal,data.address,'0x'+bytesToHex(zskCommit),currentNs);
    var txHash=await signAndBroadcast(calldata,data);
    msgEl.innerHTML='Tx sent: <a href="'+('https://explorer.testnet.rootstock.io/tx/'+txHash)+'" target="_blank" rel="noopener">'+txHash.slice(0,14)+'…</a>';
    setTimeout(function(){{fetchBatches(data.address)}},15000);
  }}catch(e){{msgEl.textContent='Error: '+e.message}}
}}

// ── Fetch and render batches ──
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
    doUpdateBatch(parseInt(e.target.dataset.ordinal),e.target.dataset.ns,e.target.dataset.zsk,session);
  }}
}});

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
  fetchBalance(data.address).then(function(b){{els.outBalance.textContent=b}});
  fetchBatches(data.address);
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
