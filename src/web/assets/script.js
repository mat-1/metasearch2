const searchInputEl = document.getElementById("search-input");

// add an element with search suggestions after the search input
const suggestionsEl = document.createElement("div");
suggestionsEl.id = "search-input-suggestions";
suggestionsEl.style.visibility = "hidden";
searchInputEl.insertAdjacentElement("afterend", suggestionsEl);

let lastValue = "";
async function updateSuggestions() {
  const value = searchInputEl.value;

  if (value.trim() === "" || value.length > 65) {
    renderSuggestions([]);
    return;
  }

  if (value === lastValue) {
    suggestionsEl.style.visibility = "visible";
    return;
  }
  lastValue = value;

  const res = await fetch(`/autocomplete?q=${encodeURIComponent(value)}`).then(
    (res) => res.json()
  );
  const options = res[1];

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
    }

    return;
  }

  // if the user starts typing but they don't have focus on the input, focus it

  // no modifier keys
  if (e.ctrlKey || e.metaKey || e.altKey || e.shiftKey) {
    return;
  }
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
// and on focus
searchInputEl.addEventListener("focus", updateSuggestions);
// on unfocus hide the suggestions
searchInputEl.addEventListener("blur", (e) => {
  suggestionsEl.style.visibility = "hidden";
});
