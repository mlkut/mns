(function () {
  fetch("/stats")
    .then(function (r) {
      return r.json();
    })
    .then(function (d) {
      document.getElementById("stat-owners").textContent = d.total_owners;
      document.getElementById("stat-names").textContent = d.total_names;
      document.getElementById("stat-packets").textContent = d.total_packets;
      document.getElementById("stat-ns").textContent = d.total_ns;
      document.getElementById("stat-zsks").textContent = d.total_zsks;
      document.getElementById("stat-block").textContent = d.last_block;
    })
    .catch(function () {});

  var list;
  try {
    list = JSON.parse(localStorage.getItem("mns-history") || "[]");
  } catch (e) {
    list = [];
  }
  if (list.length === 0) return;
  var html = '<ul class="history-list">';
  var max = Math.min(list.length, 5);
  for (var i = 0; i < max; i++) {
    var name = list[i];
    html +=
      '<li><a class="history-item" href="/' +
      encodeURIComponent(name) +
      '">' +
      '<span class="history-avatar" data-name="' +
      encodeURIComponent(name) +
      '"></span>' +
      '<span class="history-name">' +
      name +
      "</span></a></li>";
  }
  html += "</ul>";
  document.getElementById("history").innerHTML = html;
})();
