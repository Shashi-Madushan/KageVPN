/**
 * Apzur VPN docs — router, rendering, search, TOC scrollspy, theme, drawer.
 * Routes: #/<category>/<page> with an optional ?s=<section-id>.
 */
(function () {
  'use strict';

  var DEFAULT_SLUG = 'getting-started/overview';
  var IMAGE_SIZES = {
    'home.png': [1600, 925], 'home-dark.png': [1600, 1075], 'welcome.png': [1600, 1075],
    'servers.png': [1600, 1000], 'connections.png': [1600, 1000], 'logs.png': [1600, 1000],
    'routing.png': [1600, 1125], 'diagnostics.png': [1600, 1125], 'engine.png': [1600, 1125],
    'settings.png': [1600, 1375], 'usage.png': [1600, 1475], 'menu-bar-popover.png': [640, 920]
  };

  var nav = window.DOCS_NAVIGATION || [];
  var pages = window.DOCS_PAGES || {};
  var flat = [];
  nav.forEach(function (g) {
    g.items.forEach(function (it) { flat.push({ slug: it.slug, title: it.title, description: it.description || '', category: g.category }); });
  });

  var $ = function (id) { return document.getElementById(id); };
  var sidebar = $('docs-sidebar'), sidebarNav = $('sidebar-nav'), article = $('article'), main = $('main');
  var tocList = $('toc-list'), overlay = $('drawer-overlay'), menuBtn = $('menu-btn');
  var currentSlug = '';
  var spy = null;
  var lastFocus = null;

  function esc(s) {
    return String(s).replace(/[&<>"']/g, function (c) { return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]; });
  }
  function store(k, v) { try { if (v === undefined) return localStorage.getItem(k); localStorage.setItem(k, v); } catch (e) {} return null; }

  // ---- Theme ----
  function isDark() {
    var t = document.documentElement.getAttribute('data-theme');
    return t ? t === 'dark' : window.matchMedia('(prefers-color-scheme: dark)').matches;
  }
  function syncThemeImages() {
    document.querySelectorAll('img[data-dark]').forEach(function (img) {
      img.src = isDark() ? img.getAttribute('data-dark') : img.getAttribute('data-light');
    });
  }
  $('theme-btn').addEventListener('click', function () {
    var next = isDark() ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    store('apzur-docs-theme', next);
    syncThemeImages();
  });

  // ---- Sidebar (rendered once; active state toggled) ----
  function renderSidebar() {
    sidebarNav.innerHTML = nav.map(function (g) {
      return '<div class="nav-group"><div class="nav-group-title">' + esc(g.category) + '</div><ul class="nav-list">' +
        g.items.map(function (it) {
          return '<li><a class="nav-link" href="#/' + esc(it.slug) + '" data-slug="' + esc(it.slug) + '">' + esc(it.title) + '</a></li>';
        }).join('') + '</ul></div>';
    }).join('');
  }
  function markActive(slug) {
    sidebarNav.querySelectorAll('.nav-link').forEach(function (a) {
      if (a.getAttribute('data-slug') === slug) {
        a.setAttribute('aria-current', 'page');
        a.scrollIntoView({ block: 'nearest' });
      } else a.removeAttribute('aria-current');
    });
  }

  // ---- Drawer ----
  function setDrawer(open) {
    sidebar.classList.toggle('open', open);
    overlay.classList.toggle('open', open);
    document.body.classList.toggle('no-scroll', open);
    menuBtn.setAttribute('aria-expanded', String(open));
    if (open) { var a = sidebar.querySelector('[aria-current]') || sidebar.querySelector('a'); if (a) a.focus(); }
    else if (document.activeElement && sidebar.contains(document.activeElement)) menuBtn.focus();
  }
  menuBtn.addEventListener('click', function () { setDrawer(!sidebar.classList.contains('open')); });
  overlay.addEventListener('click', function () { setDrawer(false); });
  sidebar.addEventListener('click', function (e) { if (e.target.closest('a') && sidebar.classList.contains('open')) setDrawer(false); });
  window.matchMedia('(min-width: 900px)').addEventListener('change', function (e) { if (e.matches) setDrawer(false); });

  // ---- Article ----
  function figure(h) {
    var file = h.src.split('/').pop();
    var size = IMAGE_SIZES[file];
    var dims = size ? ' width="' + size[0] + '" height="' + size[1] + '"' : '';
    var dark = file === 'home.png' ? ' data-light="' + esc(h.src) + '" data-dark="' + esc(h.src.replace('home.png', 'home-dark.png')) + '"' : '';
    var src = dark && isDark() ? h.src.replace('home.png', 'home-dark.png') : h.src;
    return '<figure class="shot' + (file === 'menu-bar-popover.png' ? ' narrow' : '') + '"><img src="' + esc(src) + '"' + dims + dark +
      ' alt="' + esc(h.alt || '') + '" decoding="async">' + '' + '</figure>';
  }

  function renderPage(slug, section) {
    var page = pages[slug];
    var idx = flat.findIndex(function (p) { return p.slug === slug; });
    currentSlug = slug;
    markActive(page ? slug : '');

    if (!page) {
      document.title = 'Page not found — Apzur VPN Docs';
      article.innerHTML = '<div class="not-found"><h1 tabindex="-1">Page not found</h1>' +
        '<p class="lead">That page doesn’t exist or has moved.</p><a class="btn-primary" href="#/' + DEFAULT_SLUG + '">Go to the overview</a></div>';
      renderToc([]);
      window.scrollTo(0, 0);
      return;
    }

    document.title = page.title + ' — Apzur VPN Docs';
    var toc = page.toc || [];
    var html = '<header><h1 tabindex="-1">' + esc(page.title) + '</h1>' +
      (page.lead ? '<p class="lead">' + page.lead + '</p>' : '') + '</header>';
    if (page.heroImage) html += figure(page.heroImage);
    html += page.contentHtml;

    var prev = idx > 0 ? flat[idx - 1] : null, next = idx >= 0 && idx < flat.length - 1 ? flat[idx + 1] : null;
    if (prev || next) {
      html += '<nav class="article-pagination" aria-label="Pagination">' +
        (prev ? '<a class="pagination-card prev" href="#/' + esc(prev.slug) + '"><span class="pagination-label">Previous</span><span class="pagination-title">' + esc(prev.title) + '</span></a>' : '') +
        (next ? '<a class="pagination-card next" href="#/' + esc(next.slug) + '"><span class="pagination-label">Next</span><span class="pagination-title">' + esc(next.title) + '</span></a>' : '') + '</nav>';
    }
    article.innerHTML = html;

    renderToc(toc);
    initSpy();
    if (section && document.getElementById(section)) {
      document.getElementById(section).scrollIntoView();
    } else {
      window.scrollTo(0, 0);
    }
    // Move focus for screen readers / keyboard users without scrolling.
    var h1 = article.querySelector('h1'); if (h1 && !section) h1.focus({ preventScroll: true });
  }

  // ---- TOC ----
  function tocItem(slug, t) {
    return '<li><a class="toc-link' + (t.sub ? ' sub' : '') + '" href="#/' + esc(slug) + '?s=' + esc(t.id) + '" data-target="' + esc(t.id) + '">' + esc(t.label) + '</a></li>';
  }
  function renderToc(items) {
    tocList.innerHTML = items.map(function (t) { return tocItem(currentSlug, t); }).join('');
    $('docs-toc').style.visibility = items.length ? '' : 'hidden';
  }
  function initSpy() {
    if (spy) spy.disconnect();
    var links = document.querySelectorAll('#toc-list .toc-link');
    if (!links.length || !('IntersectionObserver' in window)) return;
    spy = new IntersectionObserver(function (entries) {
      entries.forEach(function (en) {
        if (!en.isIntersecting) return;
        links.forEach(function (l) { l.classList.toggle('active', l.getAttribute('data-target') === en.target.id); });
      });
    }, { rootMargin: '-80px 0px -70% 0px' });
    links.forEach(function (l) { var el = document.getElementById(l.getAttribute('data-target')); if (el) spy.observe(el); });
    if (links[0]) links[0].classList.add('active');
  }
  // Section links keep the route intact and just scroll.
  document.addEventListener('click', function (e) {
    var a = e.target.closest('a.toc-link');
    if (!a) return;
    e.preventDefault();
    var id = a.getAttribute('data-target'), el = document.getElementById(id);
    history.replaceState(null, '', '#/' + currentSlug + '?s=' + id);
    if (el) el.scrollIntoView({ behavior: 'smooth' });
  });

  // ---- Code copy ----
  article.addEventListener('click', function (e) {
    var img = e.target.closest('.shot img');
    if (img && img.naturalWidth > 480) { openLightbox(img); return; }
    var btn = e.target.closest('.btn-copy-code');
    if (!btn) return;
    var box = btn.closest('.code-block-container');
    var text = btn.getAttribute('data-code') || box.querySelector('code').innerText;
    var done = function () {
      var old = btn.textContent; btn.textContent = 'Copied'; btn.classList.add('copied');
      setTimeout(function () { btn.textContent = old; btn.classList.remove('copied'); }, 1500);
    };
    var fallback = function () {
      var ta = document.createElement('textarea'); ta.value = text; ta.style.position = 'fixed'; ta.style.opacity = '0';
      document.body.appendChild(ta); ta.select();
      try { document.execCommand('copy'); done(); } catch (err) {}
      ta.remove();
    };
    if (navigator.clipboard && window.isSecureContext) navigator.clipboard.writeText(text).then(done, fallback); else fallback();
  });

  // ---- Lightbox ----
  var lightbox = $('lightbox');
  function openLightbox(img) { lightbox.querySelector('img').src = img.currentSrc || img.src; lightbox.querySelector('img').alt = img.alt; lightbox.hidden = false; }
  lightbox.addEventListener('click', function () { lightbox.hidden = true; });

  // ---- Search ----
  var backdrop = $('search-backdrop'), input = $('search-input'), results = $('search-results');
  var index = null, hits = [], active = 0;
  function buildIndex() {
    if (index) return;
    var tmp = document.createElement('div');
    index = flat.map(function (p) {
      var pg = pages[p.slug] || {};
      tmp.innerHTML = pg.contentHtml || '';
      return { p: p, text: (tmp.textContent || '').replace(/\s+/g, ' ').trim(), lead: pg.lead || '' };
    });
  }
  function highlight(text, q) {
    var i = text.toLowerCase().indexOf(q);
    if (i < 0) return esc(text);
    return esc(text.slice(0, i)) + '<mark>' + esc(text.slice(i, i + q.length)) + '</mark>' + esc(text.slice(i + q.length));
  }
  function snippet(text, q) {
    var i = text.toLowerCase().indexOf(q);
    if (i < 0) return text.slice(0, 140);
    var s = Math.max(0, i - 50);
    return (s ? '…' : '') + text.slice(s, i + q.length + 90);
  }
  function runSearch(raw) {
    var q = raw.toLowerCase().trim();
    active = 0;
    if (!q) {
      hits = flat.map(function (p) { return { p: p, snip: p.description }; }).slice(0, 8);
    } else {
      buildIndex();
      hits = [];
      index.forEach(function (it) {
        var score = 0, snip = it.p.description;
        if (it.p.title.toLowerCase().indexOf(q) >= 0) score += 10;
        if (it.p.description.toLowerCase().indexOf(q) >= 0) score += 4;
        if (it.p.category.toLowerCase().indexOf(q) >= 0) score += 1;
        if (it.text.toLowerCase().indexOf(q) >= 0) { score += 2; if (score < 4) snip = snippet(it.text, q); }
        if (score) hits.push({ p: it.p, snip: snip, score: score });
      });
      hits.sort(function (a, b) { return b.score - a.score; });
      hits = hits.slice(0, 20);
    }
    if (!hits.length) { results.innerHTML = '<div class="search-empty">No results for “' + esc(raw.trim()) + '”</div>'; return; }
    results.innerHTML = hits.map(function (h, i) {
      return '<a class="search-item' + (i === 0 ? ' active' : '') + '" role="option" href="#/' + esc(h.p.slug) + '" data-i="' + i + '">' +
        '<div class="search-item-cat">' + esc(h.p.category) + '</div><div class="search-item-title">' + highlight(h.p.title, q) + '</div>' +
        '<div class="search-item-snip">' + highlight(h.snip, q) + '</div></a>';
    }).join('');
  }
  function setActive(i) {
    var items = results.querySelectorAll('.search-item');
    if (!items.length) return;
    active = (i + items.length) % items.length;
    items.forEach(function (el, n) { el.classList.toggle('active', n === active); });
    items[active].scrollIntoView({ block: 'nearest' });
  }
  function openSearch() {
    lastFocus = document.activeElement;
    backdrop.hidden = false; input.value = ''; runSearch(''); input.focus();
  }
  function closeSearch() {
    if (backdrop.hidden) return;
    backdrop.hidden = true;
    if (lastFocus && lastFocus.focus) lastFocus.focus();
  }
  $('search-trigger').addEventListener('click', openSearch);
  $('search-close').addEventListener('click', closeSearch);
  backdrop.addEventListener('click', function (e) { if (e.target === backdrop) closeSearch(); });
  results.addEventListener('click', function (e) { if (e.target.closest('a')) { lastFocus = null; closeSearch(); } });
  input.addEventListener('input', function () { runSearch(input.value); });
  input.addEventListener('keydown', function (e) {
    if (e.key === 'ArrowDown') { e.preventDefault(); setActive(active + 1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); setActive(active - 1); }
    else if (e.key === 'Enter' && hits[active]) { e.preventDefault(); lastFocus = null; location.hash = '#/' + hits[active].p.slug; closeSearch(); }
  });
  document.addEventListener('keydown', function (e) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); backdrop.hidden ? openSearch() : closeSearch(); }
    else if (e.key === '/' && backdrop.hidden && !/^(INPUT|TEXTAREA)$/.test(document.activeElement.tagName)) { e.preventDefault(); openSearch(); }
    else if (e.key === 'Escape') {
      if (!lightbox.hidden) lightbox.hidden = true;
      else if (!backdrop.hidden) closeSearch();
      else if (sidebar.classList.contains('open')) setDrawer(false);
    } else if (e.key === 'Tab' && !backdrop.hidden) {
      var f = backdrop.querySelectorAll('input, button, a'); // keep focus inside the dialog
      if (!f.length) return;
      var first = f[0], last = f[f.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    }
  });
  $('skip-link').addEventListener('click', function (e) { e.preventDefault(); main.focus(); });

  // ---- Router ----
  function route() {
    var raw = location.hash.replace(/^#\/?/, '');
    if (!raw) { location.replace('#/' + DEFAULT_SLUG); return; }
    var parts = raw.split('?s=');
    var slug = parts[0].replace(/\/$/, ''), section = parts[1] ? decodeURIComponent(parts[1]) : '';
    if (slug === currentSlug && article.firstChild) {
      var el = section && document.getElementById(section);
      if (el) el.scrollIntoView();
      return;
    }
    renderPage(slug, section);
  }
  window.addEventListener('hashchange', route);

  renderSidebar();
  route();
})();
