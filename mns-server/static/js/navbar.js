(function () {
  window._ago = function (t) {
    if (!t) return "";
    var s = Math.floor(Date.now() / 1e3 - t);
    if (s < 60) return s + "s";
    if (s < 3600) return Math.floor(s / 60) + "m";
    if (s < 86400) return Math.floor(s / 3600) + "h";
    return Math.floor(s / 86400) + "d";
  };
  function tick() {
    document.querySelectorAll("[data-ago]").forEach(function (el) {
      var t = (el.textContent = window._ago(
        Number(el.getAttribute("data-ago")),
      ));
      if (t) el.style.display = "inline";
    });
  }
  tick();
  setInterval(tick, 3e4);
  var addr = localStorage.getItem("mns-wallet-addr");
  if (addr) {
    var el = document.getElementById("nav-wallet");
    el.textContent = addr.slice(0, 6) + "\u2026" + addr.slice(-4);
    el.title = addr;
  }
})();
