pub fn particles_script() -> String {
    r#"<script>
(function() {
  const container = document.getElementById('particles');
  const count = 12;
  for (let i = 0; i < count; i++) {
    const p = document.createElement('div');
    p.className = 'particle';
    const left = Math.random() * 100;
    const size = 1 + Math.random() * 2;
    const duration = 12 + Math.random() * 18;
    const delay = Math.random() * duration;
    p.style.cssText = `
      left: ${left}%;
      width: ${size}px;
      height: ${size}px;
      animation-duration: ${duration}s;
      animation-delay: -${delay}s;
    `;
    container.appendChild(p);
  }
})();
</script>"#
        .to_string()
}
