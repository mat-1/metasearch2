const searchInputEl = document.getElementById("search-input");

// add a datalist after the search input
const datalistEl = document.createElement("datalist");
datalistEl.id = "search-input-datalist";
searchInputEl.setAttribute("list", datalistEl.id);
searchInputEl.insertAdjacentElement("afterend", datalistEl);

// update the datalist options on input
searchInputEl.addEventListener("input", async (e) => {
  const value = e.target.value;

  const res = await fetch(`/autocomplete?q=${encodeURIComponent(value)}`).then(
    (res) => res.json()
  );
  const options = res[1];

  datalistEl.innerHTML = "";
  options.forEach((option) => {
    const optionEl = document.createElement("option");
    optionEl.value = option;
    datalistEl.appendChild(optionEl);
  });
});

// if the user starts typing but they don't have focus on the input, focus it
document.addEventListener("keydown", (e) => {
  // must be a letter or number
  if (e.key.match(/^[a-z0-9]$/i) && !searchInputEl.matches(":focus")) {
    searchInputEl.focus();
  }
});
