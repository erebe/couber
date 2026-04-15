function openTagsDialog(name, tags) {
    document.getElementById('tags-video-name').value = name;
    document.getElementById('tags-input').value = tags;
    document.getElementById('tags-status').innerHTML = '';
    document.getElementById('tags-dialog').showModal();
}

const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.poster = entry.target.dataset.poster;
      observer.unobserve(entry.target);
    }
  });
});

document.querySelectorAll('video[data-poster]').forEach(v => observer.observe(v));

// Tag autocomplete + dynamic filtering
(function() {
    function filterVideos(tag) {
        document.querySelectorAll('.video-card').forEach(card => {
            if (!tag) { card.style.display = ''; return; }
            const tagsEl = card.querySelector('.video-tags');
            const cardTags = tagsEl ? tagsEl.title.split(',').map(t => t.trim().toLowerCase()) : [];
            card.style.display = cardTags.includes(tag.toLowerCase()) ? '' : 'none';
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
        events: {
            input: {
                selection(event) {
                    const value = event.detail.selection.value;
                    document.getElementById('tag-input').value = value;
                    filterVideos(value);
                }
            }
        }
    });

})();