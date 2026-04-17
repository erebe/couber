function openTagsDialog(name, tags) {
    document.getElementById('tags-video-name').value = name;
    document.getElementById('tags-input').value = tags;
    document.getElementById('tags-status').innerHTML = '';
    document.getElementById('tags-dialog').showModal();
}

const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      const video = document.createElement('video');
      const image = entry.target.querySelector('img');
      video.src = image.dataset.src;
      video.poster = image.dataset.poster;
      video.type = 'video/mp4';
      video.controls = true;
      image.replaceWith(video);

      const edit_btn = entry.target.querySelector(".video-edit-btn");
      edit_btn.addEventListener("click", () => {
        openTagsDialog(image.dataset.videoName, image.dataset.tags);
      });

     const tags = entry.target.querySelector(".video-tags");
     tags.textContent = image.dataset.tags;

      entry.target.classList.replace("invisible", "visible");
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

    document.querySelectorAll('.video-card').forEach(card => {
        observer.observe(card)
    });
})();