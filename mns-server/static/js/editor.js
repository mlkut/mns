import init, {
  create_signed_packet,
  derive_wallet_from_hex,
} from "/static/mns-wasm/mns_wasm.js";
import { getCredential } from "/static/js/credential.js";
await init();

(function () {
  var cfg = window.MNS_EDITOR;
  if (!cfg) return;

  var ZSK = cfg.zsk;
  var NAME = cfg.name;
  var CANONICAL = cfg.canonical;
  var PUB_TS = cfg.pubTs;
  var PUB_RECORDS_JSON = cfg.pubRecordsJson;
  var HAS_ZSK = cfg.hasZsk;
  var NS = cfg.ns || "";
  var DRAFT_KEY = "mns-draft-" + NAME;

  var records = [];
  var edView = document.getElementById("ed-view");
  var edForm = document.getElementById("ed-form");
  var edRecords = document.getElementById("ed-records");
  var edStatus = document.getElementById("ed-status");
  var edEdit = document.getElementById("ed-edit");
  var edCreate = document.getElementById("ed-create");
  var edAdd = document.getElementById("ed-add");
  var edPublish = document.getElementById("ed-publish");
  var edCancel = document.getElementById("ed-cancel");

  function hasWallet() {
    try {
      return !!localStorage.getItem("mns-wallet-zsk");
    } catch (e) {
      return false;
    }
  }

  function loadDraft() {
    try {
      var raw = localStorage.getItem(DRAFT_KEY);
      if (!raw) return null;
      var d = JSON.parse(raw);
      if (d.baseTimestamp !== PUB_TS) {
        localStorage.removeItem(DRAFT_KEY);
        return null;
      }
      return d;
    } catch (e) {
      return null;
    }
  }

  function saveDraft() {
    try {
      localStorage.setItem(
        DRAFT_KEY,
        JSON.stringify({
          records: records,
          baseTimestamp: PUB_TS,
        }),
      );
    } catch (e) {}
  }

  function clearDraft() {
    try {
      localStorage.removeItem(DRAFT_KEY);
    } catch (e) {}
  }

  function showButtons(create, edit) {
    if (edCreate) edCreate.style.display = create ? "" : "none";
    if (edEdit) edEdit.style.display = edit ? "" : "none";
  }

  function showForm() {
    edRecords.innerHTML = "";
    records.forEach(function (r, i) {
      edRecords.appendChild(makeRow(r, i));
    });
    edView.style.display = "none";
    edForm.classList.add("show");
  }

  function hideForm() {
    edForm.classList.remove("show");
    edView.style.display = "";
  }

  function startEditing(recs) {
    records = JSON.parse(JSON.stringify(recs));
    var suffix = "." + CANONICAL;
    records.forEach(function (r) {
      if (r.name && r.name !== "@") {
        if (r.name === CANONICAL) r.name = "";
        else if (r.name.endsWith(suffix))
          r.name = r.name.slice(0, -suffix.length);
      }
    });
    saveDraft();
    showButtons(false, false);
    showForm();
  }

  function formatRdata(r) {
    switch (r.type) {
      case "A":
      case "AAAA":
        return r.address || "";
      case "NS":
      case "CNAME":
      case "PTR":
        return r.target || "";
      case "MX":
        return (r.preference || 10) + " " + (r.exchange || "");
      case "TXT":
        return r.txt || "";
      case "SRV":
        return (
          (r.priority || 0) +
          " " +
          (r.weight || 0) +
          " " +
          (r.port || 0) +
          " " +
          (r.target || "")
        );
      case "HTTPS":
      case "SVCB":
        var s = (r.priority || 0) + " " + (r.target || ".");
        if (r.alpn) s += " alpn=" + r.alpn;
        if (r.port) s += " port=" + r.port;
        return s;
      case "SOA":
        return (r.mname || "") + " " + (r.rname || "") + " " + (r.serial || 0);
      default:
        return "";
    }
  }

  function esc(s) {
    var el = document.createElement("span");
    el.textContent = s;
    return el.innerHTML;
  }

  function makeRow(r, idx) {
    var row = document.createElement("div");
    row.className = "record-row";
    row.dataset.idx = idx;

    var sel = document.createElement("select");
    [
      "A",
      "AAAA",
      "NS",
      "CNAME",
      "MX",
      "TXT",
      "SRV",
      "HTTPS",
      "SVCB",
      "SOA",
      "PTR",
    ].forEach(function (t) {
      var opt = document.createElement("option");
      opt.value = t;
      opt.textContent = t;
      if (t === r.type) opt.selected = true;
      sel.appendChild(opt);
    });
    sel.onchange = function () {
      r.type = this.value;
      rebuildRow(row, r, idx);
    };

    var nameInput = document.createElement("input");
    nameInput.className = "name-input";
    nameInput.placeholder = "@";
    nameInput.value = r.name || "";
    nameInput.oninput = function () {
      r.name = this.value;
      saveDraft();
    };

    var ttlInput = document.createElement("input");
    ttlInput.className = "ttl-input";
    ttlInput.type = "number";
    ttlInput.placeholder = "TTL";
    ttlInput.value = r.ttl || 300;
    ttlInput.oninput = function () {
      r.ttl = parseInt(this.value) || 300;
      saveDraft();
    };

    var fields = document.createElement("div");
    fields.className = "record-fields";

    var del = document.createElement("button");
    del.className = "rec-delete";
    del.textContent = "\u00d7";
    del.onclick = function () {
      records.splice(idx, 1);
      saveDraft();
      rebuildAll();
    };

    row.appendChild(sel);
    row.appendChild(nameInput);
    row.appendChild(ttlInput);
    row.appendChild(fields);
    row.appendChild(del);

    buildFields(fields, r);
    return row;
  }

  function rebuildRow(row, r, idx) {
    var fields = row.querySelector(".record-fields");
    fields.innerHTML = "";
    buildFields(fields, r);
  }

  function rebuildAll() {
    showForm();
  }

  function buildFields(container, r) {
    switch (r.type) {
      case "A":
        addInput(container, "Address", r.address || "", function (v) {
          r.address = v;
          saveDraft();
        });
        break;
      case "AAAA":
        addInput(container, "Address", r.address || "", function (v) {
          r.address = v;
          saveDraft();
        });
        break;
      case "NS":
      case "CNAME":
      case "PTR":
        addInput(container, "Target", r.target || "", function (v) {
          r.target = v;
          saveDraft();
        });
        break;
      case "MX":
        addInput(
          container,
          "Pref",
          r.preference || 10,
          function (v) {
            r.preference = parseInt(v) || 10;
            saveDraft();
          },
          60,
        );
        addInput(container, "Exchange", r.exchange || "", function (v) {
          r.exchange = v;
          saveDraft();
        });
        break;
      case "TXT":
        var ta = document.createElement("textarea");
        ta.placeholder = "Text content";
        ta.value = r.txt || "";
        ta.oninput = function () {
          r.txt = this.value;
          saveDraft();
        };
        container.appendChild(ta);
        break;
      case "SRV":
        addInput(
          container,
          "Pri",
          r.priority || 10,
          function (v) {
            r.priority = parseInt(v) || 10;
            saveDraft();
          },
          50,
        );
        addInput(
          container,
          "Wt",
          r.weight || 0,
          function (v) {
            r.weight = parseInt(v) || 0;
            saveDraft();
          },
          50,
        );
        addInput(
          container,
          "Port",
          r.port || 443,
          function (v) {
            r.port = parseInt(v) || 443;
            saveDraft();
          },
          60,
        );
        addInput(container, "Target", r.target || "", function (v) {
          r.target = v;
          saveDraft();
        });
        break;
      case "HTTPS":
      case "SVCB":
        addInput(
          container,
          "Pri",
          r.priority || 1,
          function (v) {
            r.priority = parseInt(v) || 1;
            saveDraft();
          },
          50,
        );
        addInput(container, "Target", r.target || ".", function (v) {
          r.target = v;
          saveDraft();
        });
        addInput(container, "ALPN", r.alpn || "h2,h3", function (v) {
          r.alpn = v;
          saveDraft();
        });
        addInput(
          container,
          "Port",
          r.port || "",
          function (v) {
            r.port = v ? parseInt(v) : undefined;
            saveDraft();
          },
          60,
        );
        break;
      case "SOA":
        addInput(container, "MNAME", r.mname || "", function (v) {
          r.mname = v;
          saveDraft();
        });
        addInput(container, "RNAME", r.rname || "", function (v) {
          r.rname = v;
          saveDraft();
        });
        addInput(
          container,
          "Serial",
          r.serial || 2024010101,
          function (v) {
            r.serial = parseInt(v) || 0;
            saveDraft();
          },
          100,
        );
        addInput(
          container,
          "Refresh",
          r.refresh || 3600,
          function (v) {
            r.refresh = parseInt(v) || 0;
            saveDraft();
          },
          70,
        );
        addInput(
          container,
          "Retry",
          r.retry || 600,
          function (v) {
            r.retry = parseInt(v) || 0;
            saveDraft();
          },
          60,
        );
        addInput(
          container,
          "Expire",
          r.expire || 604800,
          function (v) {
            r.expire = parseInt(v) || 0;
            saveDraft();
          },
          80,
        );
        addInput(
          container,
          "Min",
          r.minimum || 86400,
          function (v) {
            r.minimum = parseInt(v) || 0;
            saveDraft();
          },
          70,
        );
        break;
    }
  }

  function addInput(container, label, value, onChange, width) {
    var lbl = document.createElement("span");
    lbl.className = "rec-label";
    lbl.textContent = label;
    container.appendChild(lbl);
    var inp = document.createElement("input");
    inp.value = value || "";
    if (width) inp.style.width = width + "px";
    inp.oninput = function () {
      onChange(this.value);
    };
    container.appendChild(inp);
  }

  if (edEdit) {
    edEdit.onclick = function () {
      var pubRecords = [];
      try {
        pubRecords = JSON.parse(PUB_RECORDS_JSON);
      } catch (e) {}
      startEditing(pubRecords);
    };
  }

  if (edCancel) {
    edCancel.onclick = function () {
      clearDraft();
      hideForm();
      records = [];
      if (PUB_TS > 0) showButtons(false, true);
      else showButtons(true, false);
    };
  }

  if (edAdd) {
    edAdd.onclick = function () {
      records.push({ type: "A", ttl: 300, name: "", address: "" });
      saveDraft();
      rebuildAll();
    };
  }

  if (edCreate) {
    edCreate.onclick = function () {
      var draft = loadDraft();
      var recs = draft ? draft.records : [];
      startEditing(recs);
    };
  }

  if (edPublish) {
    edPublish.onclick = async function () {
      if (!hasWallet()) {
        edStatus.textContent = "No wallet found. Go to /wallet first.";
        edStatus.className = "editor-status error";
        return;
      }
      edStatus.textContent = "Checking key...";
      edStatus.className = "editor-status";
      edPublish.disabled = true;
      try {
        var walletZsk = localStorage.getItem("mns-wallet-zsk");
        if (!walletZsk) {
          edStatus.textContent = "Wallet not found. Go to /wallet first.";
          edStatus.className = "editor-status error";
          edPublish.disabled = false;
          return;
        }

        if (walletZsk !== ZSK) {
          edStatus.textContent = "Key does not match this name's ZSK.";
          edStatus.className = "editor-status error";
          edPublish.disabled = false;
          return;
        }

        var validRecords = records.filter(function (r) {
          if (!r.type) return false;
          switch (r.type) {
            case "A":
            case "AAAA":
              return !!r.address;
            case "NS":
            case "CNAME":
            case "PTR":
              return !!r.target;
            case "MX":
              return !!r.exchange;
            case "TXT":
              return !!r.txt;
            case "SRV":
              return !!r.target;
            case "HTTPS":
            case "SVCB":
              return true;
            case "SOA":
              return !!r.mname;
            default:
              return true;
          }
        });

        if (validRecords.length === 0) {
          edStatus.textContent = "Add at least one record.";
          edStatus.className = "editor-status error";
          edPublish.disabled = false;
          return;
        }

        var keys = await getCredential("optional");

        if (!keys) {
          edStatus.textContent =
            "No credentials found. Unlock your wallet first.";
          edStatus.className = "editor-status error";
          edPublish.disabled = false;
          return;
        }

        var derived = derive_wallet_from_hex(keys.rskHex, keys.keyType, keys.keyHex);
        if (derived[1] !== ZSK) {
          edStatus.textContent = "Key does not match this name's ZSK.";
          edStatus.className = "editor-status error";
          edPublish.disabled = false;
          return;
        }

        edStatus.textContent = "Signing...";
        var b64 = create_signed_packet(
          keys.keyType,
          keys.keyHex,
          NAME,
          JSON.stringify(validRecords),
        );

        var bytes = Uint8Array.from(atob(b64), function (c) {
          return c.charCodeAt(0);
        });
        var resp = await fetch("/" + NAME, {
          method: "PUT",
          headers: { "Content-Type": "application/mns.mlkut.org#SignedPacket" },
          body: bytes,
        });

        if (resp.ok) {
          if (NS) {
            fetch("https://" + NS + "/" + NAME, {
              method: "PUT",
              headers: { "Content-Type": "application/mns.mlkut.org#SignedPacket" },
              body: bytes,
            }).catch(function () {});
          }
          clearDraft();
          edStatus.textContent = "Published!";
          edStatus.className = "editor-status ok";
          setTimeout(function () {
            window.location.reload();
          }, 1000);
        } else {
          var msg = await resp.text();
          edStatus.textContent = "Error: " + msg;
          edStatus.className = "editor-status error";
        }
      } catch (e) {
        edStatus.textContent = "Error: " + e.message;
        edStatus.className = "editor-status error";
      }
      edPublish.disabled = false;
    };
  }

  // Init
  if (hasWallet()) {
    try {
      var walletZsk = localStorage.getItem("mns-wallet-zsk");
      if (walletZsk === ZSK) {
        var draft = loadDraft();
        if (draft) {
          startEditing(draft.records);
        } else if (PUB_TS > 0) {
          showButtons(false, true);
        } else {
          showButtons(true, false);
        }
      }
    } catch (e) {}
  }
})();
