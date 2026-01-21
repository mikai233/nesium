const progressFill = document.getElementById('progress-fill');
const progressText = document.getElementById('progress-text');
const assetsProgress = {};

function updateProgressUI() {
    let totalReceived = 0;
    let totalExpected = 0;
    let count = 0;

    for (const url in assetsProgress) {
        totalReceived += assetsProgress[url].received;
        totalExpected += assetsProgress[url].expected;
        count++;
    }

    if (totalExpected > 0) {
        const p = Math.min(100, (totalReceived / totalExpected) * 100);
        if (progressFill) {
            progressFill.style.width = p + '%';
        }
        if (progressText) {
            progressText.innerText = Math.round(p) + '%';
        }
    }
}

async function trackAsset(url) {
    assetsProgress[url] = { received: 0, expected: 0 };
    try {
        const response = await fetch(url);
        const reader = response.body.getReader();
        const contentLength = +response.headers.get('Content-Length');
        // Fallback if no length or 0, though WASM/JS should have it if served correctly
        assetsProgress[url].expected = contentLength || 1000000;

        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            assetsProgress[url].received += value.length;
            updateProgressUI();
        }
    } catch (e) {
        console.error('Failed to track ' + url, e);
        // On error, mark as finished to avoid blocking total progress
        assetsProgress[url].received = assetsProgress[url].expected;
        updateProgressUI();
    }
}

// Function to initialize progress tracking
function initProgressTracking() {
    const observer = new MutationObserver((mutations) => {
        for (const mutation of mutations) {
            for (const node of mutation.addedNodes) {
                if (node.tagName === 'FLUTTER-VIEW') {
                    // Force complete all tracked assets
                    for (const url in assetsProgress) {
                        assetsProgress[url].received = assetsProgress[url].expected;
                    }
                    updateProgressUI();

                    const loader = document.getElementById('loading');
                    if (loader) {
                        loader.style.opacity = '0';
                        setTimeout(() => loader.remove(), 500);
                        observer.disconnect();
                    }
                }
            }
        }
    });
    observer.observe(document.body, { childList: true });

    // Track both main JS and WASM bundle
    trackAsset('main.dart.js');
    trackAsset('nes/pkg/nesium_wasm_bg.wasm');
}

// Execute on load
window.addEventListener('load', initProgressTracking);
