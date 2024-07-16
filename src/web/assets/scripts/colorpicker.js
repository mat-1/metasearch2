// some guy on stackoverflow wrote a bunch of codegolfed color space conversion functions so i
// stole them for this (except the cmyk functions, those were stolen from other places)

// https://stackoverflow.com/a/54116681
function hsvToHsl(h, s, v) {
  const l = v - (v * s) / 2;
  const m = Math.min(l, 1 - l);
  return [h, m ? (v - l) / m : 0, l];
}
function hslToHsv(h, s, l) {
  let v = s * Math.min(l, 1 - l) + l;
  return [h, v ? 2 - (2 * l) / v : 0, v];
}

// https://stackoverflow.com/a/54024653
function hsvToRgb(h, s, v) {
  let f = (n, k = (n + h / 60) % 6) =>
    v - v * s * Math.max(Math.min(k, 4 - k, 1), 0);
  return [f(5), f(3), f(1)];
}
// https://stackoverflow.com/a/54070620
function rgbToHsv(r, g, b) {
  let v = Math.max(r, g, b),
    c = v - Math.min(r, g, b);
  let h =
    c && (v == r ? (g - b) / c : v == g ? 2 + (b - r) / c : 4 + (r - g) / c);
  return [60 * (h < 0 ? h + 6 : h), v && c / v, v];
}
// https://stackoverflow.com/a/54071699
function rgbToHsl(r, g, b) {
  let v = Math.max(r, g, b),
    c = v - Math.min(r, g, b),
    f = 1 - Math.abs(v + v - c - 1);
  let h =
    c && (v == r ? (g - b) / c : v == g ? 2 + (b - r) / c : 4 + (r - g) / c);
  return [60 * (h < 0 ? h + 6 : h), f ? c / f : 0, (v + v - c) / 2];
}

// https://www.codeproject.com/Articles/4488/XCmyk-CMYK-to-RGB-Calculator-with-source-code
function rgbToCmyk(r, g, b) {
  const k = 1 - Math.max(r, g, b);
  if (k === 1) return [0, 0, 0, 1];
  const c = (1 - r - k) / (1 - k);
  const m = (1 - g - k) / (1 - k);
  const y = (1 - b - k) / (1 - k);
  return [c, m, y, k];
}
// https://stackoverflow.com/a/37643472
function cmykToRgb(c, m, y, k) {
  const r = (1 - c) * (1 - k);
  const g = (1 - m) * (1 - k);
  const b = (1 - y) * (1 - k);
  return [r, g, b];
}

// used for making it so an input isn't modified if we just typed in it
let activeInput = null;
document.addEventListener("keydown", () => {
  activeInput = document.activeElement;
});
document.addEventListener("focusout", () => {
  activeInput = null;

  // in case they set an input to an invalid value
  updateColorPreview();
});

const colorPickerEl = document.getElementsByClassName("answer-colorpicker")[0];

const canvasEl = colorPickerEl.getElementsByClassName(
  "answer-colorpicker-canvas"
)[0];
const canvasHueSvgEl = canvasEl.getElementsByClassName(
  "answer-colorpicker-canvas-hue-svg"
)[0];
const pickerEl = colorPickerEl.getElementsByClassName(
  "answer-colorpicker-picker"
)[0];
const previewEl = colorPickerEl.getElementsByClassName(
  "answer-colorpicker-preview"
)[0];
const sliderEl = colorPickerEl.getElementsByClassName(
  "answer-colorpicker-slider"
)[0];
const huepickerEl = colorPickerEl.getElementsByClassName(
  "answer-colorpicker-huepicker"
)[0];

const hexInputEl = document.getElementById("answer-colorpicker-hex-input");
const rgbInputEl = document.getElementById("answer-colorpicker-rgb-input");
const cmykInputEl = document.getElementById("answer-colorpicker-cmyk-input");
const hsvInputEl = document.getElementById("answer-colorpicker-hsv-input");
const hslInputEl = document.getElementById("answer-colorpicker-hsl-input");

// let hsv = [initialHue, 1, 1];
let hsv = parseHsv(hsvInputEl.value);
let hsl = parseHsl(hslInputEl.value);
let rgb = parseRgb(rgbInputEl.value);
let cmyk = parseCmyk(cmykInputEl.value);

function clamp(n, min, max) {
  return Math.max(min, Math.min(max, n));
}

function setHsv(h, s, v) {
  h = clamp(h, 0, 360);
  s = clamp(s, 0, 1);
  v = clamp(v, 0, 1);

  hsv = [h, s, v];
  hsl = hsvToHsl(...hsv);
  rgb = hsvToRgb(...hsv);
  cmyk = rgbToCmyk(...rgb);
  updateColorPreview();
}
function setHsl(h, s, l) {
  h = clamp(h, 0, 360);
  s = clamp(s, 0, 1);
  l = clamp(l, 0, 1);

  hsl = [h, s, l];
  hsv = hslToHsv(...hsl);
  rgb = hsvToRgb(...hsv);
  cmyk = rgbToCmyk(...rgb);
  updateColorPreview();
}
function setRgb(r, g, b) {
  r = clamp(r, 0, 1);
  g = clamp(g, 0, 1);
  b = clamp(b, 0, 1);

  rgb = [r, g, b];
  hsl = rgbToHsl(...rgb);
  hsv = hslToHsv(...hsl);
  cmyk = rgbToCmyk(...rgb);
  updateColorPreview();
}
function setCmyk(c, m, y, k) {
  c = clamp(c, 0, 1);
  m = clamp(m, 0, 1);
  y = clamp(y, 0, 1);
  k = clamp(k, 0, 1);

  cmyk = [c, m, y, k];
  rgb = cmykToRgb(...cmyk);
  hsl = rgbToHsl(...rgb);
  hsv = rgbToHsv(...rgb);
  updateColorPreview();
}

let mouseInCanvas = false;
function canvasMouseDown(clientX, clientY) {
  activeInput = null;
  updatePicker(clientX, clientY);
  mouseInCanvas = true;
}
function canvasMouseMove(clientX, clientY) {
  activeInput;
  if (mouseInCanvas) updatePicker(clientX, clientY);
}
function canvasMouseUp() {
  mouseInCanvas = false;
}
canvasEl.addEventListener("mousedown", (e) => {
  canvasMouseDown(e.clientX, e.clientY);
});
canvasEl.addEventListener("touchstart", (e) => {
  canvasMouseDown(e.touches[0].clientX, e.touches[0].clientY);
});
document.addEventListener("mouseup", () => {
  canvasMouseUp();
});
document.addEventListener("touchend", () => {
  canvasMouseUp();
});
document.addEventListener("mousemove", (e) => {
  canvasMouseMove(e.clientX, e.clientY);
});
document.addEventListener("touchmove", (e) => {
  canvasMouseMove(e.touches[0].clientX, e.touches[0].clientY);
});

let mouseInSlider = false;
function sliderMouseDown(clientX) {
  updateHuePicker(clientX);
  mouseInSlider = true;
}
function sliderMouseMove(clientX) {
  if (mouseInSlider) updateHuePicker(clientX);
}
function sliderMouseUp() {
  mouseInSlider = false;
}
sliderEl.addEventListener("mousedown", (e) => {
  sliderMouseDown(e.clientX);
});
sliderEl.addEventListener("touchstart", (e) => {
  sliderMouseDown(e.touches[0].clientX);
});
huepickerEl.addEventListener("mousedown", (e) => {
  sliderMouseDown(e.clientX);
});
huepickerEl.addEventListener("touchstart", (e) => {
  sliderMouseDown(e.touches[0].clientX);
});
document.addEventListener("mouseup", () => {
  sliderMouseUp();
});
document.addEventListener("touchend", () => {
  sliderMouseUp();
});
document.addEventListener("mousemove", (e) => {
  sliderMouseMove(e.clientX);
});
document.addEventListener("touchmove", (e) => {
  sliderMouseMove(e.touches[0].clientX);
});

function updatePicker(clientX, clientY) {
  const rect = canvasEl.getBoundingClientRect();
  let x = clientX - rect.left;
  let y = clientY - rect.top;
  if (x < 0) x = 0;
  if (y < 0) y = 0;
  if (x > rect.width) x = rect.width;
  if (y > rect.height) y = rect.height;

  pickerEl.style.left = `${(x / rect.width) * 100}%`;
  pickerEl.style.top = `${(y / rect.height) * 100}%`;

  const hue = hsv[0];
  setHsv(hue, x / rect.width, 1 - y / rect.height);
}

function updateHuePicker(clientX) {
  const rect = sliderEl.getBoundingClientRect();
  let x = clientX - rect.left;
  if (x < 0) x = 0;
  if (x > rect.width) x = rect.width;

  // percentage
  huepickerEl.style.left = `${(x / rect.width) * 100}%`;

  const hue = (x / rect.width) * 360;
  setHsv(hue, hsv[1], hsv[2]);
}

function updateColorPreview() {
  const [r, g, b] = rgb;
  const [hue, saturation, value] = hsv;

  const color = `rgb(${r * 255}, ${g * 255}, ${b * 255})`;
  pickerEl.style.backgroundColor = color;
  previewEl.style.backgroundColor = color;

  const hueColor = `hsl(${hue}, 100%, 50%)`;
  huepickerEl.style.backgroundColor = hueColor;
  canvasHueSvgEl.style.setProperty("stop-color", hueColor);

  pickerEl.style.left = `${saturation * 100}%`;
  pickerEl.style.top = `${(1 - value) * 100}%`;

  if (activeInput !== hexInputEl) {
    hexInputEl.value =
      "#" +
      rgb
        .map((c) =>
          Math.round(c * 255)
            .toString(16)
            .padStart(2, "0")
        )
        .join("");
  }
  if (activeInput !== rgbInputEl) {
    rgbInputEl.value = rgb.map((c) => Math.round(c * 255)).join(", ");
  }
  if (activeInput !== cmykInputEl) {
    const cmykPercent = cmyk.map((c) => Math.round(c * 100));
    cmykInputEl.value = `${cmykPercent[0]}%, ${cmykPercent[1]}%, ${cmykPercent[2]}%, ${cmykPercent[3]}%`;
  }
  if (activeInput !== hsvInputEl) {
    const hAngle = Math.round(hsv[0]);
    hsvInputEl.value = `${hAngle}°, ${Math.round(hsv[1] * 100)}%, ${Math.round(
      hsv[2] * 100
    )}%`;
  }
  if (activeInput !== hslInputEl) {
    hslInputEl.value = `${Math.round(hsl[0])}°, ${Math.round(
      hsl[1] * 100
    )}%, ${Math.round(hsl[2] * 100)}%`;
  }
}

function parseHex(value) {
  value = hexInputEl.value.replace("#", "");
  if (value.length === 6) {
    const r = parseInt(value.slice(0, 2), 16) / 255;
    const g = parseInt(value.slice(2, 4), 16) / 255;
    const b = parseInt(value.slice(4, 6), 16) / 255;
    return [r, g, b];
  } else if (value.length === 3) {
    const r = parseInt(value[0] + value[0], 16) / 255;
    const g = parseInt(value[1] + value[1], 16) / 255;
    const b = parseInt(value[2] + value[2], 16) / 255;
    return [r, g, b];
  }
}
function setFromHexInput() {
  setRgb(...parseHex(hexInputEl.value));
}
hexInputEl.addEventListener("input", setFromHexInput);

function parseRgb(value) {
  return value.split(",").map((c) => parseInt(c) / 255);
}
function setFromRgbInput() {
  setRgb(...parseRgb(rgbInputEl.value));
}
rgbInputEl.addEventListener("input", setFromRgbInput);

function parseCmyk(value) {
  return value.split(",").map((c) => parseInt(c) / 100);
}
function setFromCmykInput() {
  setCmyk(...parseCmyk(cmykInputEl.value));
}
cmykInputEl.addEventListener("input", setFromCmykInput);

function parseHsv(value) {
  value = hsvInputEl.value.split(",").map((c) => parseInt(c));
  value[1] /= 100;
  value[2] /= 100;
  return value;
}
function setFromHsvInput() {
  setHsv(...parseHsv(hsvInputEl.value));
}
hsvInputEl.addEventListener("input", setFromHsvInput);

function parseHsl(value) {
  value = hslInputEl.value.split(",").map((c) => parseInt(c));
  value[1] /= 100;
  value[2] /= 100;
  return value;
}
function setFromHslInput() {
  setHsl(...parseHsl(hslInputEl.value));
}
hslInputEl.addEventListener("input", setFromHslInput);

updateColorPreview();
