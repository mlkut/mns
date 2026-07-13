import init, { render_avatar_svg } from "/static/mns-wasm/mns_wasm.js";
await init();
(function () {
  var avatars = document.querySelectorAll(".history-avatar[data-name]");
  for (var i = 0; i < avatars.length; i++) {
    (function (el) {
      try {
        var svg = render_avatar_svg(
          decodeURIComponent(el.getAttribute("data-name")),
        );
        el.innerHTML = svg;
        var s = el.querySelector("svg");
        if (s) {
          s.style.cssText = "width:100%;height:100%;display:block;";
          s.removeAttribute("width");
          s.removeAttribute("height");
        }
      } catch (e) {}
    })(avatars[i]);
  }
})();
