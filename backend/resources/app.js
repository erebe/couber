function openTagsDialog(name, tags) {
    document.getElementById('tags-video-name').value = name;
    document.getElementById('tags-input').value = tags;
    document.getElementById('tags-status').innerHTML = '';
    document.getElementById('tags-dialog').showModal();
}

document.addEventListener('htmx:afterRequest', function(evt) {
    var form = evt.detail.elt.closest('#tags-form');
    if (form && evt.detail.successful) {
        setTimeout(function() {
            document.getElementById('tags-dialog').close();
        }, 600);
    }
});

const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.poster = entry.target.dataset.poster;
      observer.unobserve(entry.target);
    }
  });
});

document.querySelectorAll('video[data-poster]').forEach(v => observer.observe(v));