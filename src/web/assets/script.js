const searchInputEl = document.getElementById("search-input");

if (searchInputEl) {
  // add an element with search suggestions after the search input
  const suggestionsEl = document.createElement("div");
  suggestionsEl.id = "search-input-suggestions";
  suggestionsEl.style.visibility = "hidden";
  searchInputEl.insertAdjacentElement("afterend", suggestionsEl);

  let lastValue = "";
  let nextQueryId = 0;
  let lastLoadedQueryId = -1;
  async function updateSuggestions() {
    const value = searchInputEl.value;

    if (value === "") {
      suggestionsEl.style.visibility = "hidden";
      nextQueryId++;
      lastLoadedQueryId = nextQueryId;
      return;
    }

    if (value === lastValue) {
      suggestionsEl.style.visibility = "visible";
      return;
    }
    lastValue = value;

    const thisQueryId = nextQueryId;
    nextQueryId++;

    const res = await fetch(
      `/autocomplete?q=${encodeURIComponent(value)}`
    ).then((res) => res.json());
    const options = res[1];

    // this makes sure we don't load suggestions out of order
    if (thisQueryId < lastLoadedQueryId) {
      return;
    }
    lastLoadedQueryId = thisQueryId;

    renderSuggestions(options);
  }

  function renderSuggestions(options) {
    if (options.length === 0) {
      suggestionsEl.style.visibility = "hidden";
      return;
    }

    suggestionsEl.style.visibility = "visible";
    suggestionsEl.innerHTML = "";
    options.forEach((option) => {
      const optionEl = document.createElement("div");
      optionEl.textContent = option;
      optionEl.className = "search-input-suggestion";
      suggestionsEl.appendChild(optionEl);

      optionEl.addEventListener("mousedown", () => {
        searchInputEl.value = option;
        searchInputEl.focus();
        searchInputEl.form.submit();
      });
    });
  }

  let focusedSuggestionIndex = -1;
  let focusedSuggestionEl = null;

  function clearFocusedSuggestion() {
    if (focusedSuggestionEl) {
      focusedSuggestionEl.classList.remove("focused");
      focusedSuggestionEl = null;
      focusedSuggestionIndex = -1;
    }
  }

  function focusSelectionIndex(index) {
    clearFocusedSuggestion();
    focusedSuggestionIndex = index;
    focusedSuggestionEl = suggestionsEl.children[focusedSuggestionIndex];
    focusedSuggestionEl.classList.add("focused");
    searchInputEl.value = focusedSuggestionEl.textContent;
  }

  document.addEventListener("keydown", (e) => {
    // if any modifier keys are pressed, ignore all this
    if (e.ctrlKey || e.metaKey || e.altKey || e.shiftKey) {
      return;
    }

    // if it's focused then use different keybinds
    if (searchInputEl.matches(":focus")) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        if (focusedSuggestionIndex === -1) {
          focusSelectionIndex(0);
        } else if (focusedSuggestionIndex < suggestionsEl.children.length - 1) {
          focusSelectionIndex(focusedSuggestionIndex + 1);
        } else {
          focusSelectionIndex(0);
        }
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        if (focusedSuggestionIndex === -1) {
          focusSelectionIndex(suggestionsEl.children.length - 1);
        } else if (focusedSuggestionIndex > 0) {
          focusSelectionIndex(focusedSuggestionIndex - 1);
        } else {
          focusSelectionIndex(suggestionsEl.children.length - 1);
        }
      } else if (e.key === "Escape") {
        clearFocusedSuggestion();
        suggestionsEl.style.visibility = "hidden";
      }

      return;
    }

    // if the currently selected element is not the search bar and is contenteditable, don't do anything
    const focusedEl = document.querySelector(":focus");
    if (
      focusedEl &&
      (focusedEl.tagName.toLowerCase() == "input" ||
        focusedEl.tagName.toLowerCase() == "textarea" ||
        focusedEl.getAttribute("contenteditable") !== null)
    )
      return;

    // if the user starts typing but they don't have focus on the input, focus it

    // must be a letter or number
    if (e.key.match(/^[a-z0-9]$/i)) {
      searchInputEl.focus();
    }
    // right arrow key focuses it at the end
    else if (e.key === "ArrowRight") {
      searchInputEl.focus();
      searchInputEl.setSelectionRange(
        searchInputEl.value.length,
        searchInputEl.value.length
      );
    }
    // left arrow key focuses it at the beginning
    else if (e.key === "ArrowLeft") {
      searchInputEl.focus();
      searchInputEl.setSelectionRange(0, 0);
    }
    // backspace key focuses it at the end
    else if (e.key === "Backspace") {
      searchInputEl.focus();
      searchInputEl.setSelectionRange(
        searchInputEl.value.length,
        searchInputEl.value.length
      );
    }
  });

  // update the input suggestions on input
  searchInputEl.addEventListener("input", () => {
    clearFocusedSuggestion();
    updateSuggestions();
  });
  // and when they click suggestions
  searchInputEl.addEventListener("click", updateSuggestions);
  // on unfocus hide the suggestions
  searchInputEl.addEventListener("blur", (e) => {
    suggestionsEl.style.visibility = "hidden";
  });
}

const customCssEl = document.getElementById("custom-css");
if (customCssEl) {
  // tab to indent
  // https://stackoverflow.com/a/6637396
  customCssEl.addEventListener("keydown", (e) => {
    if (e.key == "Tab") {
      e.preventDefault();
      var start = customCssEl.selectionStart;
      var end = customCssEl.selectionEnd;
      customCssEl.value =
        customCssEl.value.substring(0, start) +
        "\t" +
        customCssEl.value.substring(end);
      customCssEl.selectionStart = customCssEl.selectionEnd = start + 1;
    }
  });

  // ctrl+enter anywhere on the page to submit
  const saveEl = document.getElementById("save-settings-button");
  document.addEventListener("keydown", (e) => {
    if (e.key == "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      console.log("click");
      saveEl.click();
    }
  });

  // save whether the details are open or not
  const customCssDetailsEl = document.getElementById("custom-css-details");
  const customCssDetailsOpen = localStorage.getItem("custom-css-details-open");
  if (customCssDetailsOpen === "true") customCssDetailsEl.open = true;
  customCssDetailsEl.addEventListener("toggle", () => {
    localStorage.setItem("custom-css-details-open", customCssDetailsEl.open);
  });
}
