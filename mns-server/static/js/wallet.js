import init, {
  init_client,
  generate_wallet,
  derive_wallet_from_hex,
  key_to_mnemonic,
  migrate_from_mnemonic,
  get_balance,
  register as wasm_register,
  update_batch as wasm_update_batch,
} from "/static/mns-wasm/mns_wasm.js";
import {
  storeCredential,
  getCredential,
} from "/static/js/credential.js";
await init();

var cfg = window.MNS_WALLET;
if (cfg) {
  init_client(cfg.rpcUrl);
}

const CHAIN_ID = cfg ? cfg.chainId : 0;

let session = null;

const els = {
  form: document.getElementById("wc-form"),
  identity: document.getElementById("wc-identity"),
  usernameField: document.getElementById("wc-username-field"),
  username: document.getElementById("wc-username"),
  mnemonic: document.getElementById("wc-mnemonic"),
  mnemonicField: document.getElementById("wc-mnemonic-field"),
  msg: document.getElementById("wc-msg"),
  status: document.getElementById("wc-status"),
  statusText: document.getElementById("wc-status-text"),
  outUsername: document.getElementById("out-username"),
  outAddress: document.getElementById("out-address"),
  outBalance: document.getElementById("out-balance"),
  outZsk: document.getElementById("out-zsk"),
  backupWarn: document.getElementById("wc-backup-warn"),
  faucetNote: document.getElementById("wc-faucet-note"),
  generate: document.getElementById("wc-generate"),
  lock: document.getElementById("wc-lock"),
};

// ── localStorage helpers ──

function loadStored() {
  const username = localStorage.getItem("mns-wallet-username");
  const addr = localStorage.getItem("mns-wallet-addr");
  const zsk = localStorage.getItem("mns-wallet-zsk");
  if (username && addr && zsk)
    return { username: username, address: addr, zsk_commitment_hex: zsk };
  return null;
}

function saveStored(data) {
  localStorage.setItem("mns-wallet-username", data.username);
  localStorage.setItem("mns-wallet-addr", data.address);
  localStorage.setItem("mns-wallet-zsk", data.zsk_commitment_hex);
}

function clearStored() {
  localStorage.removeItem("mns-wallet-username");
  localStorage.removeItem("mns-wallet-addr");
  localStorage.removeItem("mns-wallet-zsk");
}

// ── RPC helpers ──

async function fetchBalance(address) {
  try {
    return await get_balance(address);
  } catch {
    return "—";
  }
}

async function fetchBatches(address) {
  try {
    var r = await fetch("/api/batches/" + address);
    if (!r.ok) return;
    var batches = await r.json();
    var container = document.getElementById("wc-batches");
    var list = document.getElementById("wc-batch-list");
    if (!batches.length) {
      container.classList.remove("show");
      return;
    }
    container.classList.add("show");
    list.innerHTML = "";
    for (var i = 0; i < batches.length; i++) {
      var b = batches[i];
      var div = document.createElement("div");
      div.className = "wc-batch";
      var zskShort =
        b.zsk.length > 18
          ? b.zsk.slice(0, 10) + "\u2026" + b.zsk.slice(-6)
          : b.zsk;
      div.innerHTML =
        "" +
        '<div class="wc-batch-row">' +
        '  <span class="wc-batch-ordinal">Batch #' +
        b.ordinal +
        "</span>" +
        '  <span class="wc-batch-ns">' +
        b.ns +
        "</span>" +
        "</div>" +
        '<div class="wc-batch-zsk"><span class="label">ZSK:</span> ' +
        zskShort +
        "</div>" +
        '<div class="wc-batch-actions">' +
        '  <button class="wc-btn-sm wc-batch-update" data-ordinal="' +
        b.ordinal +
        '" data-ns="' +
        b.ns +
        '">Update ZSK</button>' +
        "</div>" +
        '<div class="wc-batch-msg" id="wc-batch-msg-' +
        b.ordinal +
        '"></div>';
      list.appendChild(div);
    }
  } catch {}
}

document
  .getElementById("wc-batch-list")
  .addEventListener("click", function (e) {
    if (e.target.classList.contains("wc-batch-update")) {
      if (!session) return;
      doUpdateBatch(parseInt(e.target.dataset.ordinal), e.target.dataset.ns);
    }
  });

// ── Signing (prompt for credentials on every transaction) ──

async function signTx(fn) {
  var keys = await getCredential("optional");
  if (!keys) return "No credentials found.";
  try {
    var derived = derive_wallet_from_hex(
      keys.rskHex,
      keys.keyType,
      keys.keyHex,
    );
    if (derived[0] !== session.address)
      return "Credential does not match this account.";
    return await fn(keys.rskHex, keys.keyType, keys.keyHex);
  } finally {
    keys = null;
  }
}

async function doRegister(ns) {
  var msgEl = document.getElementById("wc-register-msg");
  msgEl.textContent = "Preparing\u2026";
  try {
    var err = await signTx(async function (rskHex, keyType, keyHex) {
      var txHash = await wasm_register(rskHex, session.zsk_commitment_hex, ns);
      msgEl.innerHTML =
        'Tx sent: <a href="' +
        ("https://explorer.testnet.rootstock.io/tx/" + txHash) +
        '" target="_blank" rel="noopener">' +
        txHash.slice(0, 14) +
        "\u2026</a>";
      setTimeout(function () {
        fetchBatches(session.address);
      }, 15000);
      return null;
    });
    if (err) msgEl.textContent = err;
  } catch (e) {
    msgEl.textContent = "Error: " + e.message;
  }
}

async function doUpdateBatch(ordinal, currentNs) {
  var msgEl = document.getElementById("wc-batch-msg-" + ordinal);
  msgEl.textContent = "Preparing\u2026";
  try {
    var err = await signTx(async function (rskHex, keyType, keyHex) {
      var txHash = await wasm_update_batch(
        rskHex,
        BigInt(ordinal),
        session.address,
        session.zsk_commitment_hex,
        currentNs,
      );
      msgEl.innerHTML =
        'Tx sent: <a href="' +
        ("https://explorer.testnet.rootstock.io/tx/" + txHash) +
        '" target="_blank" rel="noopener">' +
        txHash.slice(0, 14) +
        "\u2026</a>";
      setTimeout(function () {
        fetchBatches(session.address);
      }, 15000);
      return null;
    });
    if (err) msgEl.textContent = err;
  } catch (e) {
    msgEl.textContent = "Error: " + e.message;
  }
}

// ── UI state ──

function showUnlocked(data, isNew) {
  session = {
    username: data.username,
    address: data.address,
    zsk_commitment_hex: data.zsk_commitment_hex,
  };
  els.form.classList.add("hide");
  els.identity.classList.add("show");
  els.status.classList.remove("locked");
  els.status.classList.add("unlocked");
  els.statusText.textContent = "Unlocked";
  els.outUsername.textContent = data.username;
  els.outAddress.textContent = data.address;
  els.outZsk.textContent = data.zsk_commitment_hex;
  els.backupWarn.style.display = isNew ? "block" : "none";
  var faucet = cfg ? cfg.faucet : "";
  if (faucet) {
    els.faucetNote.innerHTML =
      'Fund this address with test tRBTC from the <a href="' +
      faucet +
      '" target="_blank" rel="noopener">faucet</a> to register names.';
  }
  saveStored(data);
  fetchBalance(data.address).then(function (b) {
    els.outBalance.textContent = b;
  });
  fetchBatches(data.address);
}

function showLocked() {
  session = null;
  els.form.classList.remove("hide");
  els.identity.classList.remove("show");
  els.status.classList.remove("unlocked");
  els.status.classList.add("locked");
  els.statusText.textContent = "Locked";
  els.msg.textContent = "";
  els.outBalance.textContent = "\u2014";
  els.mnemonic.value = "";
  els.mnemonicField.style.display = "none";
  els.username.value = "";
  els.usernameField.style.display = "none";
  clearStored();
}

// ── Unlock / Generate flows ──

async function unlockWithCredential() {
  els.msg.textContent = "Unlocking\u2026";
  var keys = await getCredential("optional");
  if (!keys) {
    els.msg.textContent =
      "No credentials found. Generate a new account instead.";
    return;
  }
  const username = keys.id;
  try {
    const result = derive_wallet_from_hex(
      keys.rskHex,
      keys.keyType,
      keys.keyHex,
    );
    showUnlocked(
      {
        username: username,
        address: result[0],
        zsk_commitment_hex: result[1],
      },
      false,
    );
  } catch (e) {
    els.msg.textContent = "Derivation failed: " + e.message;
  }
}

els.form.addEventListener("submit", async function (e) {
  e.preventDefault();

  // Migration: mnemonic visible
  const mnemonic = els.mnemonic.value.trim().replace(/\s+/g, " ");
  if (mnemonic) {
    const username = els.username.value.trim();
    if (!username) {
      els.msg.textContent = "Enter a username for this account.";
      return;
    }
    els.msg.textContent = "Migrating\u2026";
    try {
      const result = migrate_from_mnemonic(mnemonic);
      const rskHex = result[0];
      const keyHex = result[1];
      await storeCredential(username, rskHex, 0, keyHex);
      showUnlocked(
        {
          username: username,
          address: result[2],
          zsk_commitment_hex: result[3],
        },
        true,
      );
      localStorage.removeItem("mns-wallet-mnemonic");
    } catch (err) {
      els.msg.textContent = "Migration failed: " + err.message;
    }
    return;
  }

  // Generate: username field visible (shown by New Account button)
  if (els.usernameField.style.display !== "none") {
    const username = els.username.value.trim();
    if (!username) {
      els.msg.textContent = "Enter a username for this account.";
      return;
    }
    els.msg.textContent = "Generating\u2026";
    try {
      const keys = generate_wallet();
      const rskHex = keys.rsk_privkey;
      const keyHex = keys.zsk_privkey;
      const keyType = keys.zsk_key_type;
      await storeCredential(username, rskHex, keyType, keyHex);
      showUnlocked(
        {
          username: username,
          address: keys.address,
          zsk_commitment_hex: keys.zsk_commitment,
        },
        true,
      );
    } catch (err) {
      els.msg.textContent = "Error: " + err.message;
    }
    return;
  }

  // Unlock: just trigger credential picker
  await unlockWithCredential();
});

els.generate.addEventListener("click", function () {
  // Show username field so user can name this account, then submit to generate
  els.usernameField.style.display = "";
  els.username.focus();
  els.msg.textContent =
    "Enter a name for this account, then click New Account again.";
});

els.lock.addEventListener("click", showLocked);

document.getElementById("wc-register").addEventListener("click", function () {
  if (!session) return;
  var ns = document.getElementById("wc-ns").value.trim();
  if (!ns) {
    document.getElementById("wc-register-msg").textContent =
      "Enter a name server";
    return;
  }
  doRegister(ns);
});

// ── Copy buttons ──

document.querySelectorAll(".wc-copy").forEach(function (btn) {
  btn.addEventListener("click", function () {
    if (!session) return;
    const key = btn.getAttribute("data-copy");
    const val =
      key === "address"
        ? session.address
        : key === "zsk"
          ? session.zsk_commitment_hex
          : "";
    if (!val) return;
    navigator.clipboard
      .writeText(val)
      .then(function () {
        btn.classList.add("done");
        btn.textContent = "copied";
        setTimeout(function () {
          btn.classList.remove("done");
          btn.textContent = "copy";
        }, 1200);
      })
      .catch(function () {});
  });
});

// ── Auto-unlock on page load ──

(function attemptAutoUnlock() {
  // 1. Check for old mnemonic (migration path)
  var oldMnemonic = localStorage.getItem("mns-wallet-mnemonic");
  if (oldMnemonic) {
    els.usernameField.style.display = "";
    els.mnemonicField.style.display = "";
    els.mnemonic.value = oldMnemonic;
    els.msg.textContent =
      "Your old wallet was found. Enter a username to migrate.";
    return;
  }

  // 2. Check localStorage for existing wallet
  var stored = loadStored();
  if (stored) {
    showUnlocked(
      {
        username: stored.username,
        address: stored.address,
        zsk_commitment_hex: stored.zsk_commitment_hex,
      },
      false,
    );
    return;
  }

  // 3. Empty state — just show the form with Unlock / New Account buttons
})();
