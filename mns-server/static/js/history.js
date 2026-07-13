try {
  var h = JSON.parse(localStorage.getItem("mns-history") || "[]");
  h = h.filter(function (n) {
    return n !== MNS_NAME;
  });
  h.unshift(MNS_NAME);
  if (h.length > 5) h.length = 5;
  localStorage.setItem("mns-history", JSON.stringify(h));
} catch (e) {}
