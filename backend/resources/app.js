// ── Tags modal state ──────────────────────────────────────────────────────────

let currentTags = [];   // encoded tags currently on the video
let videoName   = '';   // name of the video being edited

function openTagsDialog(name, tagsStr) {
    videoName = name;
    currentTags = tagsStr ? tagsStr.split(',').map(t => t.trim()).filter(Boolean) : [];

    document.getElementById('tags-video-name').value = name;
    document.getElementById('tags-status').innerHTML      = '';
    document.getElementById('suggest-status').innerHTML   = '';
    document.getElementById('normalize-status').innerHTML = '';
    document.getElementById('delete-status').innerHTML    = '';
    document.getElementById('suggested-chips').innerHTML  = '';
    document.getElementById('tags-new-input').value      = '';
    renderCurrentTags();
    document.getElementById('tags-dialog').showModal();
}

function renderCurrentTags() {
    const container = document.getElementById('tags-chips');
    container.innerHTML = '';
    currentTags.forEach(tag => {
        const chip = makeChip(tag, () => {
            currentTags = currentTags.filter(t => t !== tag);
            renderCurrentTags();
        }, 'chip chip-removable');
        container.appendChild(chip);
    });
}

function makeChip(label, onClick, className) {
    const chip = document.createElement('span');
    chip.className = className;
    chip.textContent = label;
    chip.addEventListener('click', onClick);
    return chip;
}

// Add a tag from the text input on Enter
document.addEventListener('DOMContentLoaded', () => {
    const input = document.getElementById('tags-new-input');
    if (input) {
        input.addEventListener('keydown', e => {
            if (e.key === 'Enter') {
                e.preventDefault();
                const val = input.value.trim();
                if (val) {
                    if (!currentTags.includes(val)) {
                        currentTags.push(val);
                        renderCurrentTags();
                    }
                    input.value = '';
                }
            }
        });
    }
});

// ── Normalize tags ────────────────────────────────────────────────────────────

async function normalizeTags() {
    const statusEl = document.getElementById('normalize-status');
    const btn      = document.getElementById('normalize-btn');

    if (currentTags.length === 0) {
        statusEl.innerHTML = '<pre class="status-error">No tags to normalize.</pre>';
        return;
    }

    statusEl.innerHTML = '<span class="status-loading">Normalizing…</span>';
    btn.disabled = true;

    try {
        const decoded = currentTags.map(t => decodeURIComponent(t));
        const res = await fetch('/api/normalize-tags', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ tags: decoded }),
        });
        if (!res.ok) throw new Error(await res.text());
        const normalized = await res.json();
        currentTags = normalized;
        renderCurrentTags();
        statusEl.innerHTML = '<span class="status-success">Tags normalized.</span>';
    } catch (e) {
        statusEl.innerHTML = `<pre class="status-error">Error: ${e.message}</pre>`;
    } finally {
        btn.disabled = false;
    }
}

// ── Suggest tags ──────────────────────────────────────────────────────────────

let suggestedTags = [];  // last batch of suggested tags

async function suggestTags() {
    const statusEl   = document.getElementById('suggest-status');
    const container  = document.getElementById('suggested-chips');
    const btn        = document.getElementById('suggest-btn');
    const selectAll  = document.getElementById('select-all-btn');

    container.innerHTML = '';
    selectAll.hidden = true;
    suggestedTags = [];
    statusEl.innerHTML = '<span class="status-loading">Fetching suggestions…</span>';
    btn.disabled = true;

    try {
        const res = await fetch(`/api/suggest-tags?name=${encodeURIComponent(videoName)}`);
        if (!res.ok) throw new Error(await res.text());
        const tags = await res.json();
        statusEl.innerHTML = '';
        suggestedTags = tags;
        tags.forEach(tag => {
            const chip = makeChip(tag, () => addSuggestedChip(tag, chip), 'chip chip-suggest');
            container.appendChild(chip);
        });
        if (tags.length === 0) {
            statusEl.innerHTML = '<pre class="status-error">No suggestions returned.</pre>';
        } else {
            selectAll.hidden = false;
        }
    } catch (e) {
        statusEl.innerHTML = `<pre class="status-error">Error: ${e.message}</pre>`;
    } finally {
        btn.disabled = false;
    }
}

function addSuggestedChip(tag, chip) {
    if (!currentTags.includes(tag)) {
        currentTags.push(tag);
        renderCurrentTags();
    }
    chip.classList.add('chip-added');
}

function selectAllSuggested() {
    const container = document.getElementById('suggested-chips');
    suggestedTags.forEach((tag, i) => {
        const chip = container.children[i];
        if (chip) addSuggestedChip(tag, chip);
    });
}

// ── Save tags ─────────────────────────────────────────────────────────────────

async function saveTags() {
    const statusEl = document.getElementById('tags-status');
    statusEl.innerHTML = '';

    const body = new URLSearchParams({ name: videoName, tags: currentTags.join(',') });

    try {
        const res = await fetch('/update-tags', { method: 'POST', body });
        const html = await res.text();
        statusEl.innerHTML = html;

        // Update the video card in the page
        const card = [...document.querySelectorAll('.video-card')]
            .find(c => c.dataset.tags !== undefined &&
                       c.querySelector('video')?.dataset?.videoName === videoName ||
                       c.querySelector('img')?.dataset?.videoName === videoName);
        if (card) {
            card.dataset.tags = tagsDecoded;
            const tagsEl = card.querySelector('.video-tags');
            if (tagsEl) tagsEl.textContent = tagsDecoded;
        }
    } catch (e) {
        statusEl.innerHTML = `<pre class="status-error">Error: ${e.message}</pre>`;
    }
}

// ── Delete video ──────────────────────────────────────────────────────────────

async function deleteVideo() {
    if (!confirm(`Delete video "${videoName}"? This cannot be undone.`)) return;

    const statusEl = document.getElementById('delete-status');
    statusEl.innerHTML = '';

    const body = new URLSearchParams({ name: videoName });
    try {
        const res = await fetch('/delete-video', { method: 'POST', body });
        const html = await res.text();
        statusEl.innerHTML = html;

        if (res.ok) {
            // Remove the card from the page
            const card = [...document.querySelectorAll('.video-card')]
                .find(c =>
                    c.querySelector('video, img')?.dataset?.videoName === videoName);
            if (card) card.remove();
            setTimeout(() => document.getElementById('tags-dialog').close(), 800);
        }
    } catch (e) {
        statusEl.innerHTML = `<pre class="status-error">Error: ${e.message}</pre>`;
    }
}

// ── Video card lazy-loading & tag filtering ───────────────────────────────────

const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            const image = entry.target.querySelector('img');
            const video = document.createElement('video');
            video.src = image.dataset.src;
            video.poster = image.dataset.poster;
            video.dataset.videoName = image.dataset.videoName;
            video.type = 'video/mp4';
            video.controls = true;
            image.replaceWith(video);

            const editBtn = entry.target.querySelector('.video-edit-btn');
            editBtn.addEventListener('click', () => {
                openTagsDialog(image.dataset.videoName, entry.target.dataset.tags);
            });

            const tagsEl = entry.target.querySelector('.video-tags');
            tagsEl.textContent = entry.target.dataset.tags;

            entry.target.classList.replace('invisible', 'visible');
            observer.unobserve(entry.target);
        }
    });
});

// Tag autocomplete + dynamic filtering
(function () {
    function filterVideos(tag) {
        document.querySelectorAll('.video-card').forEach(card => {
            if (!tag) { card.style.display = ''; return; }
            const cardTags = (card.dataset.tags || '').split(',').map(t => t.trim().toLowerCase());
            card.style.display = cardTags.find(t => t.includes(tag.toLowerCase())) ? '' : 'none';
        });
    }

    let aAutoComplete;
    aAutoComplete = new autoComplete({
        selector: '#tag-input',
        data: {
            src: async () => {
                const res = await fetch('/api/tags');
                return res.json();
            },
            cache: true,
        },
        debounce: 300,
        resultItem: { highlight: true },
        resultsList: { maxResults: 10 },
        events: {
            input: {
                selection: (event) => {
                    event.target.value = event.detail.selection.value;
                    filterVideos(event.target.value);
                },
                input: (event) => {
                    filterVideos(event.target.value);
                    aAutoComplete.start();
                }
            }
        }
    });

    document.querySelectorAll('.video-card').forEach(card => observer.observe(card));
})();