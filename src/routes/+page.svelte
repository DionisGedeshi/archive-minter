<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, tick } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";

  type Article = {
    id: number;
    url: string;
    title: string;
    content: string;
    tag: string | null;
    scroll_progress: number;
  };

  let url = "";
  let loading = false;
  let errorMessage = "";
  let isSidebarOpen = true;
  let searchQuery = "";
  let mintInputEl: HTMLInputElement | null = null;

  let fontSize = 1.1;
  let lineHeight = 1.6;
  let useSerifFont = false;
  let isTextSettingsOpen = false;
  let isLightMode = false;
  let settingsLoaded = false;

  let selectedTagFilter: string | null = null;
  let editingTagValue = "";

  let readerViewEl: HTMLElement | null = null;
  let scrollPercent = 0;
  let scrollSaveTimer: ReturnType<typeof setTimeout>;

  let ftsResults: Article[] | null = null;
  let ftsSearchTimer: ReturnType<typeof setTimeout>;

  let articles: Article[] = [];
  let filteredArticles: Article[] = [];
  let selectedArticle: Article | null = null;

  async function loadArticles() {
    try {
      articles = await invoke<Article[]>("get_articles");

      if (selectedArticle) {
        const updated = articles.find((a) => a.id === selectedArticle?.id);
        selectedArticle = updated || (articles.length > 0 ? articles[0] : null);
      } else if (articles.length > 0) {
        selectedArticle = articles[0];
      }
    } catch (err) {
      console.error("Failed to load articles:", err);
    }
  }

  onMount(async () => {
    const saved = localStorage.getItem("reader-settings");
    if (saved) {
      const parsed = JSON.parse(saved);
      isLightMode = parsed.isLightMode ?? false;
      fontSize = parsed.fontSize ?? 1.1;
      lineHeight = parsed.lineHeight ?? 1.6;
      useSerifFont = parsed.useSerifFont ?? false;
    }

    settingsLoaded = true;

    await loadArticles();
  });

  async function deleteArticle(id: number) {
    try {
      await invoke("delete_article", { id });
      articles = articles.filter((a) => a.id !== id);
      if (selectedArticle?.id === id) {
        selectedArticle = articles.length > 0 ? articles[0] : null;
      }
    } catch (err) {
      errorMessage = String(err);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;

    if (
      document.activeElement?.tagName === "INPUT" ||
      document.activeElement?.tagName === "TEXTAREA"
    ) {
      if (e.key === "Escape") {
        (document.activeElement as HTMLElement).blur();
        searchQuery = "";
        ftsResults = null;
      }
      return;
    }

    if (e.key === "j") {
      e.preventDefault();
      if (filteredArticles.length === 0) return;
      const currentIndex = selectedArticle
        ? filteredArticles.findIndex((a) => a.id === selectedArticle!.id)
        : -1;
      const nextIdx = currentIndex + 1;
      if (nextIdx < filteredArticles.length) {
        selectArticle(filteredArticles[nextIdx]);
      }
    } else if (e.key === "k") {
      e.preventDefault();
      if (filteredArticles.length === 0) return;
      const currentIndex = selectedArticle
        ? filteredArticles.findIndex((a) => a.id === selectedArticle!.id)
        : -1;
      const prevIdx = currentIndex - 1;
      if (prevIdx >= 0) {
        selectArticle(filteredArticles[prevIdx]);
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (readerViewEl) readerViewEl.scrollBy({ top: 100, behavior: "smooth" });
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (readerViewEl)
        readerViewEl.scrollBy({ top: -100, behavior: "smooth" });
    } else if (e.key === "m") {
      e.preventDefault();
      if (mintInputEl) mintInputEl.focus();
    } else if (e.key === "d" && selectedArticle) {
      e.preventDefault();
      deleteArticle(selectedArticle.id);
    } else if (e.key === "Escape") {
      isTextSettingsOpen = false;
      searchQuery = "";
      ftsResults = null;
    }
  }

  function scrollToTop() {
    if (readerViewEl) {
      readerViewEl.scrollTo({ top: 0, behavior: "smooth" });
    }
  }

  $: if (selectedArticle && readerViewEl) {
    const article = selectedArticle;
    const readerView = readerViewEl;
    tick().then(() => {
      const { scrollHeight, clientHeight } = readerView as HTMLElement;
      readerView.scrollTop =
        article.scroll_progress * (scrollHeight - clientHeight);
    });
  }

  function handleScroll() {
    if (!readerViewEl || !selectedArticle) return;
    const article = selectedArticle;
    const { scrollTop, scrollHeight, clientHeight } = readerViewEl;
    const maxScroll = scrollHeight - clientHeight;
    if (maxScroll <= 0) return;

    scrollPercent = scrollTop / maxScroll;

    clearTimeout(scrollSaveTimer);
    scrollSaveTimer = setTimeout(async () => {
      try {
        await invoke("save_scroll_progress", {
          id: article.id,
          progress: scrollPercent,
        });
        article.scroll_progress = scrollPercent;
      } catch (err) {
        console.error(err);
      }
    }, 400);
  }

  async function selectArticle(article: Article) {
    selectedArticle = article;
    isTextSettingsOpen = false;
    scrollPercent = article.scroll_progress || 0;
    await tick();
    if (readerViewEl) {
      if (article.scroll_progress > 0) {
        const maxScroll = readerViewEl.scrollHeight - readerViewEl.clientHeight;
        readerViewEl.scrollTop = scrollPercent * maxScroll;
      } else {
        readerViewEl.scrollTop = 0;
      }
    }
  }

  function onSearchQueryChange() {
    clearTimeout(ftsSearchTimer);
    ftsResults = null;
    if (!searchQuery.trim()) return;
    ftsSearchTimer = setTimeout(async () => {
      try {
        ftsResults = await invoke("deep_fts_search", { query: searchQuery });
      } catch {
        ftsResults = null;
      }
    }, 300);
  }

  async function handleExport() {
    if (!selectedArticle) return;
    try {
      await invoke("export_to_markdown", {
        title: selectedArticle.title,
        content: selectedArticle.content,
        url: selectedArticle.url,
      });
    } catch (err) {
      errorMessage = "Export failed: " + String(err);
      setTimeout(() => (errorMessage = ""), 3000);
    }
  }

  async function handleMint() {
    if (!url) return;

    loading = true;
    errorMessage = "";

    try {
      const newArticle = await invoke<Article>("mint_archive", { url });
      articles = [newArticle, ...articles];
      selectedArticle = newArticle;
      url = "";
    } catch (err) {
      errorMessage = "Failed to mint: " + err;
    } finally {
      loading = false;
    }
  }

  async function handleSaveTag() {
    if (!selectedArticle) return;
    const targetTag = editingTagValue.trim() || null;
    const selectedId = selectedArticle.id;

    try {
      await invoke("update_article_tag", {
        id: selectedId,
        tag: editingTagValue,
      });

      if (selectedArticle) selectedArticle.tag = targetTag;
      articles = articles.map((a) =>
        a.id === selectedId ? { ...a, tag: targetTag } : a,
      );
      editingTagValue = "";
    } catch (err) {
      errorMessage = `Tag configuration fault: ${err}`;
    }
  }

  $: if (typeof window !== "undefined" && settingsLoaded) {
    const settings = {
      isLightMode,
      fontSize,
      lineHeight,
      useSerifFont,
    };
    localStorage.setItem("reader-settings", JSON.stringify(settings));
  }

  $: uniqueTags = Array.from(
    new Set(articles.map((a) => a.tag).filter((tag): tag is string => !!tag)),
  ).sort();

  $: filteredArticles = articles.filter((article: any) => {
    const matchesTag = selectedTagFilter
      ? article.tag === selectedTagFilter
      : true;

    const matchesSearch = searchQuery
      ? article.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        article.content.toLowerCase().includes(searchQuery.toLowerCase())
      : true;

    return matchesTag && matchesSearch;
  });

  $: if (selectedArticle) {
    editingTagValue = selectedArticle.tag || "";
  }

  $: if (searchQuery !== undefined) {
    clearTimeout(ftsSearchTimer);
    ftsSearchTimer = setTimeout(() => {
      triggerSearch();
    }, 250);
  } else if (searchQuery === "") {
    clearTimeout(ftsSearchTimer);
    loadArticles();
  }

  $: displayContent = (() => {
    if (!selectedArticle) return "";
    if (!searchQuery || searchQuery.trim().length === 0)
      return selectedArticle.content;

    try {
      const safeQuery = searchQuery.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const regex = new RegExp(`(${safeQuery})(?![^<]*>)`, "gi");

      return selectedArticle.content.replace(
        regex,
        `<mark class="search-highlight">$1</mark>`,
      );
    } catch (e) {
      console.error("Highlight error:", e);
      return selectedArticle.content;
    }
  })();

  async function triggerSearch() {
    try {
      if (searchQuery.trim().length > 2) {
        articles = await invoke<Article[]>("deep_fts_search", {
          query: searchQuery.trim(),
        });
      } else {
        articles = await invoke<Article[]>("get_articles");
      }

      if (articles.length > 0) {
        if (
          !selectedArticle ||
          !articles.some((a) => selectedArticle && a.id === selectedArticle.id)
        ) {
          selectedArticle = articles[0];
        }
      } else {
        selectedArticle = null;
      }
    } catch (err) {
      console.error("Search invocation failed", err);
    }
  }

  $: if (searchQuery.length >= 3 && selectedArticle) {
    scrollToFirstMatch();
  }

  async function scrollToFirstMatch() {
    await tick();
    setTimeout(() => {
      const firstMatch = document.querySelector("mark.search-highlight");
      if (firstMatch) {
        firstMatch.scrollIntoView({ behavior: "smooth", block: "center" });
      }
    }, 50);
  }

  async function openSupportLink() {
    await openUrl('https://dionisg.gumroad.com/l/hnqqrp')
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="shell">
  <div class="app-layout" class:light-theme={isLightMode}>
    {#if isSidebarOpen}
      <aside class="sidebar">
        <div class="sidebar-header">
          <h2>My Archive</h2>
        </div>

        <div class="search-box">
          <input
            type="text"
            bind:value={searchQuery}
            on:input={onSearchQueryChange}
            placeholder="Search archive... (Esc to clear)"
            class="search-input"
          />
          {#if searchQuery.length > 0}
            <button
              class="clear-search-btn"
              on:click={() => {
                searchQuery = "";
                ftsResults = null;
              }}
              style="position: absolute; right: 5px; background: none; border: none; cursor: pointer; color: #888;"
            >
              X
            </button>
          {/if}
        </div>

        <div class="nav-section">
          <h3>Categories</h3>
          <button
            class="category-row {selectedTagFilter === null ? 'active' : ''}"
            on:click={() => (selectedTagFilter = null)}
          >
            <span class="category-label">📁 All Articles</span>
            <span class="badge">{articles.length}</span>
          </button>
          {#each uniqueTags as tag}
            <button
              class="category-row {selectedTagFilter === tag ? 'active' : ''}"
              on:click={() => (selectedTagFilter = tag)}
            >
              <span class="category-label">🏷️ {tag}</span>
              <span class="badge"
                >{articles.filter((a) => a.tag === tag).length}</span
              >
            </button>
          {/each}
        </div>

        <div class="nav-section streaming-list">
          <h3>Documents</h3>
          {#if filteredArticles.length === 0}
            <p class="empty-text">No matches found.</p>
          {/if}
          {#each filteredArticles as article}
            <div
              class="article-card {selectedArticle?.id === article.id
                ? 'active'
                : ''}"
            >
              <button
                class="card-click-area"
                on:click={() => selectArticle(article)}>{article.title}</button
              >
              <button
                class="card-delete-btn"
                title="Delete Document"
                on:click|stopPropagation={() => deleteArticle(article.id)}
                >🗑️</button
              >
            </div>
          {/each}
        </div>
        <button class="support-button" on:click={openSupportLink}>
          <span class="heart-icon">❤️</span> Support
        </button>
      </aside>
    {/if}

    <main class="main-content">
      <header class="top-bar">
        <div class="top-bar-left">
          <button
            class="hamburger-btn"
            on:click={() => (isSidebarOpen = !isSidebarOpen)}
            title="Toggle Sidebar">☰</button
          >
          <button
            class="theme-toggle-btn"
            on:click={() => (isLightMode = !isLightMode)}
            title="Toggle Light/Dark Mode"
          >
            {isLightMode ? "🌙" : "☀️"}
          </button>
          <div class="branding">
            <h1>ArchiveMinter 🪙</h1>
            <p>Strip the clutter. Own your data.</p>
          </div>
        </div>

        <div class="top-bar-right">
          <form class="mint-ui-form" on:submit|preventDefault={handleMint}>
            <input
              type="url"
              bind:this={mintInputEl}
              bind:value={url}
              placeholder="Paste article URL here..."
              required
            />
            <button type="submit" disabled={loading}>
              {loading ? "Minting..." : "Mint"}
            </button>
          </form>
          {#if errorMessage}<p class="floating-error">{errorMessage}</p>{/if}
        </div>
      </header>

      <div class="reading-progress-track">
        <div
          class="reading-progress-fill"
          style="width: {Math.round(scrollPercent * 100)}%"
        ></div>
      </div>

      <div
        class="reader-view"
        bind:this={readerViewEl}
        on:scroll={handleScroll}
      >
        {#if scrollPercent > 0.05}
          <button
            class="scroll-to-top"
            on:click={scrollToTop}
            aria-label="Scroll to top"
          >
            ↑
          </button>
        {/if}
        {#if selectedArticle}
          <div class="control-hub">
            <div class="hub-left">
              <span class="url-source"
                >Source: <a
                  href={selectedArticle.url}
                  target="_blank"
                  rel="noreferrer">{selectedArticle.url}</a
                ></span
              >
              <div class="inline-tagger">
                <span class="inline-tag-icon">🏷️</span>
                <input
                  type="text"
                  bind:value={editingTagValue}
                  placeholder="Assign label..."
                  on:blur={handleSaveTag}
                  on:keydown={(e) => e.key === "Enter" && handleSaveTag()}
                />
              </div>
            </div>

            <div class="hub-right">
              <button
                class="export-btn"
                on:click={handleExport}
                title="Export to standalone Markdown">💾 Save MD</button
              >
              <button
                class="settings-trigger"
                on:click={() => (isTextSettingsOpen = !isTextSettingsOpen)}
                >⚙️ Typography</button
              >

              {#if isTextSettingsOpen}
                <div class="popover-panel">
                  <div class="popover-row">
                    <span>Font Size</span>
                    <div class="stepper">
                      <button
                        on:click={() =>
                          (fontSize = Math.max(0.7, fontSize - 0.1))}>-</button
                      >
                      <span
                        style="min-width: 45px; text-align: center; font-size: 0.8rem; color: #cdd6f4;"
                      >
                        {Math.round(fontSize * 100)}%
                      </span>
                      <button
                        on:click={() =>
                          (fontSize = Math.min(2.5, fontSize + 0.1))}>+</button
                      >
                    </div>
                  </div>
                  <div class="popover-row">
                    <span>Spacing</span>
                    <div class="toggle-group">
                      <button
                        on:click={() => (lineHeight = 1.4)}
                        class={lineHeight === 1.4 ? "selected" : ""}
                        >Tight</button
                      >
                      <button
                        on:click={() => (lineHeight = 1.6)}
                        class={lineHeight === 1.6 ? "selected" : ""}>Mid</button
                      >
                      <button
                        on:click={() => (lineHeight = 1.9)}
                        class={lineHeight === 1.9 ? "selected" : ""}
                        >Wide</button
                      >
                    </div>
                  </div>
                  <div class="popover-row">
                    <span>Font Type</span>
                    <div class="toggle-group">
                      <button
                        on:click={() => (useSerifFont = false)}
                        class={!useSerifFont ? "selected" : ""}>Sans</button
                      >
                      <button
                        on:click={() => (useSerifFont = true)}
                        class={useSerifFont ? "selected" : ""}>Serif</button
                      >
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          </div>

          <article
            class="article-content {useSerifFont ? 'serif' : ''}"
            style="font-size: {fontSize}rem; line-height: {lineHeight};"
          >
            <h1 class="reader-title">{selectedArticle.title}</h1>
            <div class="rendered-markdown-body">
              {@html displayContent}
            </div>
          </article>
        {:else}
          <div class="empty-billboard">
            <div class="billboard-graphics">🪙</div>
            <h3>No Document Loaded</h3>
            <p>
              Paste a web link up top or press <kbd>m</kbd> to focus and load clean
              content.
            </p>
          </div>
        {/if}
      </div>
    </main>
  </div>
</div>