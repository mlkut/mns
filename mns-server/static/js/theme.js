(function () {
  var t = localStorage.getItem("mns-theme");
  if (!t) {
    t = window.matchMedia("(prefers-color-scheme:light)").matches
      ? "light"
      : "dark";
  }
  document.documentElement.setAttribute("data-theme", t);

  window._toggleTheme = function () {
    var c = document.documentElement.getAttribute("data-theme");
    var n = c === "light" ? "dark" : "light";
    document.documentElement.setAttribute("data-theme", n);
    localStorage.setItem("mns-theme", n);
  };
})();
