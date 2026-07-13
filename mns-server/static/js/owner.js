document.getElementById("copy-addr")?.addEventListener("click", function () {
  var addr = this.getAttribute("data-addr");
  navigator.clipboard
    .writeText(addr)
    .then(
      function () {
        this.classList.add("copied");
        var orig = this.innerHTML;
        this.innerHTML =
          '<svg class="copy-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>' +
          '<span class="copy-label">Copied!</span>';
        var that = this;
        setTimeout(function () {
          that.innerHTML = orig;
          that.classList.remove("copied");
        }, 1500);
      }.bind(this),
    )
    .catch(function () {});
});
try {
  var walletAddr = localStorage.getItem("mns-wallet-addr");
  var ownerAddr = document
    .getElementById("you-badge")
    ?.getAttribute("data-owner-address");
  if (
    walletAddr &&
    ownerAddr &&
    walletAddr.toLowerCase() === ownerAddr.toLowerCase()
  ) {
    document.getElementById("you-badge").classList.add("show");
  }
} catch (e) {}
