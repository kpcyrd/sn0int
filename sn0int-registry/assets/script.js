window.addEventListener('DOMContentLoaded', function() {
    var clipboard = new ClipboardJS('[data-clipboard-text]');
    clipboard.on('success', function(e) {
        e.clearSelection();
    });

    // un-hide elements that wouldn't work without javascript
    Array.from(document.querySelectorAll('.noscript-hidden'))
        .forEach(function(x) {
            x.classList.remove('noscript-hidden');
        });
});
