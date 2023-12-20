// add a datalist after the search input
const searchInputEl = document.getElementById("search-input");
const datalistEl = document.createElement("datalist");
datalistEl.id = "search-input-datalist";
searchInputEl.setAttribute("list", datalistEl.id);
searchInputEl.insertAdjacentElement("afterend", datalistEl);

// update the datalist options on input
searchInputEl.addEventListener("input", async (e) => {
  const value = e.target.value;

  const res = await fetch(`/autocomplete?q=${value}`).then((res) => res.json());
  const options = res[1];

  datalistEl.innerHTML = "";
  options.forEach((option) => {
    const optionEl = document.createElement("option");
    optionEl.value = option;
    datalistEl.appendChild(optionEl);
  });
});
