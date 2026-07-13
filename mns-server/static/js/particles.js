(function () {
  var container = document.getElementById("particles");
  if (!container) return;
  var count = 12;
  for (var i = 0; i < count; i++) {
    var p = document.createElement("div");
    p.className = "particle";
    var left = Math.random() * 100;
    var size = 1 + Math.random() * 2;
    var duration = 12 + Math.random() * 18;
    var delay = Math.random() * duration;
    p.style.cssText =
      "left:" +
      left +
      "%;width:" +
      size +
      "px;height:" +
      size +
      "px;" +
      "animation-duration:" +
      duration +
      "s;animation-delay:-" +
      delay +
      "s;";
    container.appendChild(p);
  }
})();
