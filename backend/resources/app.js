function openTagsDialog(name, btn) {
    const tags = btn.closest('.video-card').dataset.tags;
    document.getElementById('tags-video-name').value = name;
    document.getElementById('tags-input').value = tags;
    document.getElementById('tags-status').innerHTML = '';
    document.getElementById('tags-dialog').showModal();
}

const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.poster = entry.target.dataset.poster;
      entry.target.style.visibility = "visible";
      observer.unobserve(entry.target);
    }
  });
});


// Tag autocomplete + dynamic filtering
(function() {
    function filterVideos(tag) {
        document.querySelectorAll('.video-card').forEach(card => {
            if (!tag) { card.style.display = ''; return; }
            const cardTags = (card.dataset.tags || '').split(',').map(t => t.trim().toLowerCase());
            card.style.display = cardTags.find(t => t.includes(tag.toLowerCase())) ? '' : 'none';
        });
    }

    new autoComplete({
        selector: '#tag-input',
        data: {
            src: async () => {
                const res = await fetch('/api/tags');
                return res.json();
            },
            cache: true,
        },
        resultItem: { highlight: true },
        resultsList: {
            maxResults: 10,
        },
    });

    const input = document.getElementById('tag-input');
    let debounceTimer;
    input.addEventListener('input', () => {
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => filterVideos(input.value), 300);
    });

    input.addEventListener('autoComplete-select', (e) => {
        filterVideos(e.detail.value);
    });

    document.querySelectorAll('video[data-poster]').forEach(v => observer.observe(v));
    document.querySelectorAll('.video-card').forEach(card => {
        card.querySelector('.video-tags').textContent = card.dataset.tags;
    });
})();