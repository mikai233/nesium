(function () {
  const progressFill = document.getElementById('progress-fill');
  const progressText = document.getElementById('progress-text');
  const assetsProgress = Object.create(null);

  let finished = false;
  let lastProgress = 0; // displayed progress
  let targetProgress = 0;
  let animationFrameId = null;
  let fallbackFinishTimerId = null;
  const startMs = (typeof performance !== 'undefined' && performance.now) ? performance.now() : Date.now();
  let lastTickMs = startMs;

  function setProgress(p) {
    const next = Math.max(lastProgress, Math.min(100, p));
    lastProgress = next;

    if (progressFill) progressFill.style.transform = `scaleX(${next / 100})`;
    if (progressText) progressText.innerText = Math.round(next) + '%';
  }

  function getActualProgressPercent() {
    let totalReceived = 0;
    let totalExpected = 0;

    for (const url in assetsProgress) {
      totalReceived += assetsProgress[url].received;
      totalExpected += assetsProgress[url].expected;
    }

    if (totalExpected <= 0) return null;
    return (totalReceived / totalExpected) * 100;
  }

  function getSimulatedProgressPercent(elapsedMs) {
    const clamp01 = (x) => Math.max(0, Math.min(1, x));
    const easeOutQuad = (t) => 1 - (1 - t) * (1 - t);

    // 0% -> 20% quickly, 20% -> 70% moderately, 70% -> 90% slowly.
    const stage1Ms = 500;
    const stage2Ms = 3500;
    const stage3Ms = 15000;

    if (elapsedMs <= stage1Ms) return 20 * clamp01(elapsedMs / stage1Ms);

    if (elapsedMs <= stage2Ms) {
      const t = clamp01((elapsedMs - stage1Ms) / (stage2Ms - stage1Ms));
      return 20 + (70 - 20) * easeOutQuad(t);
    }

    const t = clamp01((elapsedMs - stage2Ms) / (stage3Ms - stage2Ms));
    return 70 + (90 - 70) * easeOutQuad(t);
  }

  function finishLoading() {
    if (finished) return;
    finished = true;

    if (animationFrameId != null) cancelAnimationFrame(animationFrameId);
    if (fallbackFinishTimerId != null) clearTimeout(fallbackFinishTimerId);
    // Make the last "fill to 100%" visible before fading out.
    if (progressFill) progressFill.style.transition = 'transform 0.25s ease-out';
    setProgress(100);

    const loader = document.getElementById('loading');
    if (loader) {
      setTimeout(() => {
        loader.style.opacity = '0';
        setTimeout(() => loader.remove(), 500);
      }, 180);
    }
  }

  function tickProgress() {
    if (finished) return;

    const nowMs = (typeof performance !== 'undefined' && performance.now) ? performance.now() : Date.now();
    // If the main thread was blocked (e.g. compilation), avoid catching up in one huge jump.
    const dtMsRaw = nowMs - lastTickMs;
    const dtMs = Math.min(50, Math.max(0, dtMsRaw));
    lastTickMs = nowMs;

    const elapsedMs = nowMs - startMs;
    const simulated = getSimulatedProgressPercent(elapsedMs);
    const actual = getActualProgressPercent();

    // Keep space for the final 100% on Flutter first frame.
    // Also avoid jumping to ~100% early when we only observed a few small fetches.
    const plateau = 95;
    const actualBound = actual == null ? 0 : Math.min(actual, simulated + 8, 92);
    const computedTarget = Math.min(plateau, Math.max(simulated, actualBound));
    targetProgress = Math.max(targetProgress, computedTarget);

    // Smoothly animate towards the target, with an additional speed cap so large target
    // changes still turn into a visible ramp instead of a jump.
    const tauMs = 260;
    const alpha = 1 - Math.exp(-dtMs / tauMs);
    const desired = lastProgress + (targetProgress - lastProgress) * alpha;

    const maxSpeedPercentPerSecond = 35;
    const maxStep = (maxSpeedPercentPerSecond * dtMs) / 1000;
    const step = Math.min(Math.max(0, desired - lastProgress), maxStep);
    setProgress(lastProgress + step);
    animationFrameId = requestAnimationFrame(tickProgress);
  }

  // Best signal: Flutter rendered the first frame.
  window.addEventListener('flutter-first-frame', finishLoading, { once: true });

  // Fallback for environments that don't emit the event.
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const node of mutation.addedNodes) {
        if (!node || !node.tagName) continue;
        if (node.tagName === 'FLUTTER-VIEW' || node.tagName === 'FLT-GLASS-PANE') {
          // Avoid hiding too early on builds that do emit `flutter-first-frame`.
          fallbackFinishTimerId = setTimeout(finishLoading, 2000);
          observer.disconnect();
          return;
        }
      }
    }
  });
  if (document.body) observer.observe(document.body, { childList: true });

  // Best-effort network telemetry: read a clone for progress, return original Response unchanged.
  const originalFetch = typeof window.fetch === 'function' ? window.fetch.bind(window) : null;
  if (originalFetch) {
    window.fetch = async function (...args) {
      const response = await originalFetch(...args);

      let cloned;
      try {
        cloned = response.clone();
      } catch {
        return response;
      }

      const url = response.url;
      if (!url.match(/\.(js|wasm|json|otf|ttf|mjs)$/) && !url.includes('main.dart')) return response;

      const contentLength = +response.headers.get('Content-Length');
      if (!contentLength || response.status !== 200 || !cloned.body) return response;

      assetsProgress[url] = { received: 0, expected: contentLength };

      const reader = cloned.body.getReader();
      (function read() {
        reader.read().then(({ done, value }) => {
          if (done) return;
          assetsProgress[url].received += value?.length ?? 0;
          read();
        }).catch(() => {
          // Ignore progress errors; this is best-effort telemetry.
        });
      })();

      return response;
    };
  }

  tickProgress();
})();
