use maud::html;

use crate::engines::{EngineResponse, SearchQuery};

use super::regex;

pub fn request(query: &SearchQuery) -> EngineResponse {
    let matched_colors = MatchedColorModel::new(&query.query);

    let rgb;
    let cmyk;
    let hsv;
    let hsl;

    if let Some(rgb_) = matched_colors.rgb {
        rgb = rgb_;
        cmyk = rgb_to_cmyk(rgb);
        hsv = rgb_to_hsv(rgb);
        hsl = rgb_to_hsl(rgb);
    } else if let Some(cmyk_) = matched_colors.cmyk {
        cmyk = cmyk_;
        rgb = cmyk_to_rgb(cmyk);
        hsv = rgb_to_hsv(rgb);
        hsl = rgb_to_hsl(rgb);
    } else if let Some(hsv_) = matched_colors.hsv {
        hsv = hsv_;
        rgb = hsv_to_rgb(hsv);
        cmyk = rgb_to_cmyk(rgb);
        hsl = hsv_to_hsl(hsv);
    } else if let Some(hsl_) = matched_colors.hsl {
        hsl = hsl_;
        rgb = hsv_to_rgb(hsl_to_hsv(hsl));
        cmyk = rgb_to_cmyk(rgb);
        hsv = rgb_to_hsv(rgb);
    } else {
        return EngineResponse::new();
    }

    let (r, g, b) = rgb;
    let (c, m, y, k) = cmyk;
    let (hsv_h, hsv_s, hsv_v) = hsv;
    let (hsl_h, hsl_s, hsl_l) = hsl;

    let hex_str = format!(
        "#{:02x}{:02x}{:02x}",
        (r * 255.) as u8,
        (g * 255.) as u8,
        (b * 255.) as u8
    );
    let rgb_str = format!(
        "{}, {}, {}",
        (r * 255.) as u8,
        (g * 255.) as u8,
        (b * 255.) as u8
    );
    let cmyk_str = format!(
        "{:.0}%, {:.0}%, {:.0}%, {:.0}%",
        c * 100.,
        m * 100.,
        y * 100.,
        k * 100.
    );
    let hsv_str = format!(
        "{:.0}°, {:.0}%, {:.0}%",
        hsv_h * 360.,
        hsv_s * 100.,
        hsv_v * 100.
    );
    let hsl_str = format!(
        "{:.0}°, {:.0}%, {:.0}%",
        hsl_h * 360.,
        hsl_s * 100.,
        hsl_l * 100.
    );

    let hue_picker_x = hsv_h * 100.;
    let picker_x = hsv_s * 100.;
    let picker_y = (1. - hsv_v) * 100.;

    let hue_css_color = format!("hsl({}, 100%, 50%)", hsv_h * 360.);

    // yes the design of this is absolutely nabbed from google's
    EngineResponse::answer_html(html! {
        div.answer-colorpicker {
            div.answer-colorpicker-preview-container {
                div.answer-colorpicker-preview style=(format!("background-color: {hex_str}")) {}
                div.answer-colorpicker-canvas-container {
                    div.answer-colorpicker-picker-container {
                        div.answer-colorpicker-picker style=(format!("background-color: {hex_str}; left: {picker_x}%; top: {picker_y}%;")) {}
                    }
                    svg.answer-colorpicker-canvas {
                        defs {
                            linearGradient id="saturation" x1="0%" x2="100%" y1="0%" y2="0%" {
                                stop offset="0%" stop-color="#fff" {}
                                stop.answer-colorpicker-canvas-hue-svg offset="100%" stop-color=(hex_str) {}
                            }
                            linearGradient id="value" x1="0%" x2="0%" y1="0%" y2="100%" {
                                stop offset="0%" stop-color="#fff" {}
                                stop offset="100%" stop-color="#000" {}
                            }
                        }
                        // the .1 fixes a bug that's present at least on firefox that makes the
                        // rightmost column of pixels look wrong
                        rect width="100.1%" height="100%" fill="url(#saturation)" {}
                        rect width="100.1%" height="100%" fill="url(#value)" style="mix-blend-mode: multiply" {}
                    }
                }
            }
            div.answer-colorpicker-slider-container {
                div.answer-colorpicker-huepicker style=(format!("background-color: {hue_css_color}; left: {hue_picker_x}%")) {}
                svg.answer-colorpicker-slider {
                    defs {
                        linearGradient id="hue" x1="0%" x2="100%" y1="0%" y2="0%" {
                            stop offset="0%" stop-color="#ff0000" {}
                            stop offset="16.666%" stop-color="#ffff00" {}
                            stop offset="33.333%" stop-color="#00ff00" {}
                            stop offset="50%" stop-color="#00ffff" {}
                            stop offset="66.666%" stop-color="#0000ff" {}
                            stop offset="83.333%" stop-color="#ff00ff" {}
                            stop offset="100%" stop-color="#ff0000" {}
                        }
                    }
                    rect width="100%" height="50%" y="25%" fill="url(#hue)" {}
                }
            }
            div.answer-colorpicker-hex-input-container {
                label for="answer-colorpicker-hex-input" { "HEX" }
                div.answer-colorpicker-input-container {
                    input #answer-colorpicker-hex-input type="text" autocomplete="off" value=(hex_str) {}
                }
            }
            div.answer-colorpicker-other-inputs {
                div {
                    label for="answer-colorpicker-rgb-input" { "RGB" }
                    div.answer-colorpicker-input-container {
                        input #answer-colorpicker-rgb-input type="text" autocomplete="off" value=(rgb_str) {}
                    }
                }
                div {
                    label for="answer-colorpicker-cmyk-input" { "CMYK" }
                    div.answer-colorpicker-input-container {
                        input #answer-colorpicker-cmyk-input type="text" autocomplete="off" value=(cmyk_str) {}
                    }
                }
                div {
                    label for="answer-colorpicker-hsv-input" { "HSV" }
                    div.answer-colorpicker-input-container {
                        input #answer-colorpicker-hsv-input type="text" autocomplete="off" value=(hsv_str) {}
                    }
                }
                div {
                    label for="answer-colorpicker-hsl-input" { "HSL" }
                    div.answer-colorpicker-input-container {
                        input #answer-colorpicker-hsl-input type="text" autocomplete="off" value=(hsl_str) {}
                    }
                }
            }
        }
        script src="/scripts/colorpicker.js" {}
    })
}

pub struct MatchedColorModel {
    pub rgb: Option<(f64, f64, f64)>,
    pub cmyk: Option<(f64, f64, f64, f64)>,
    pub hsv: Option<(f64, f64, f64)>,
    pub hsl: Option<(f64, f64, f64)>,
}
impl MatchedColorModel {
    pub fn new(query: &str) -> Self {
        let mut rgb = None;
        let mut cmyk = None;
        let mut hsv = None;
        let mut hsl = None;

        let query = query.trim().to_lowercase();

        if regex!("^color ?picker$").is_match(&query) {
            // default to red
            rgb = Some((1., 0., 0.));
        } else if let Some(caps) = regex!("^#?([0-9a-f]{6})$").captures(&query) {
            let hex_str = caps.get(1).unwrap().as_str();
            let hex_num = u32::from_str_radix(hex_str, 16).unwrap();
            let r = ((hex_num >> 16) & 0xff) as f64 / 255.;
            let g = ((hex_num >> 8) & 0xff) as f64 / 255.;
            let b = (hex_num & 0xff) as f64 / 255.;

            let r = r.clamp(0., 1.);
            let g = g.clamp(0., 1.);
            let b = b.clamp(0., 1.);

            rgb = Some((r, g, b));
        } else if let Some(caps) = regex!("^#?([0-9a-f]{3})$").captures(&query) {
            let hex_str = caps.get(1).unwrap().as_str();
            let hex_num = u32::from_str_radix(hex_str, 16).unwrap();
            let r = ((hex_num >> 8) & 0xf) as f64 / 15.;
            let g = ((hex_num >> 4) & 0xf) as f64 / 15.;
            let b = (hex_num & 0xf) as f64 / 15.;

            let r = r.clamp(0., 1.);
            let g = g.clamp(0., 1.);
            let b = b.clamp(0., 1.);

            rgb = Some((r, g, b));
        } else if let Some(caps) =
            regex!("^rgb\\((\\d{1,3}), ?(\\d{1,3}), ?(\\d{1,3})\\)$").captures(&query)
        {
            let r = caps
                .get(1)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 255.;
            let g = caps
                .get(2)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 255.;
            let b = caps
                .get(3)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 255.;

            let r = r.clamp(0., 1.);
            let g = g.clamp(0., 1.);
            let b = b.clamp(0., 1.);

            rgb = Some((r, g, b));
        } else if let Some(caps) =
            regex!("^cmyk\\((\\d{1,3})%, ?(\\d{1,3})%, ?(\\d{1,3})%, ?(\\d{1,3})%\\)$")
                .captures(&query)
        {
            let c = caps
                .get(1)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 100.;
            let m = caps
                .get(2)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 100.;
            let y = caps
                .get(3)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 100.;
            let k = caps
                .get(4)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 100.;

            let c = c.clamp(0., 1.);
            let m = m.clamp(0., 1.);
            let y = y.clamp(0., 1.);
            let k = k.clamp(0., 1.);

            cmyk = Some((c, m, y, k));
        } else if let Some(caps) =
            regex!("^hsv\\((\\d{1,3})(?:°|deg|), ?(\\d{1,3})%, ?(\\d{1,3})%\\)$").captures(&query)
        {
            let h = caps
                .get(1)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 360.;
            let s = caps
                .get(2)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 100.;
            let v = caps
                .get(3)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 100.;

            let h = h.clamp(0., 1.);
            let s = s.clamp(0., 1.);
            let v = v.clamp(0., 1.);

            hsv = Some((h, s, v));
        } else if let Some(caps) =
            regex!("^hsl\\((\\d{1,3})(?:°|deg|), ?(\\d{1,3})%, ?(\\d{1,3})%\\)$").captures(&query)
        {
            let h = caps
                .get(1)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 360.;
            let s = caps
                .get(2)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 100.;
            let l = caps
                .get(3)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or_default()
                / 100.;

            let h = h.clamp(0., 1.);
            let s = s.clamp(0., 1.);
            let l = l.clamp(0., 1.);

            hsl = Some((h, s, l));
        }

        Self {
            rgb,
            cmyk,
            hsv,
            hsl,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rgb.is_none() && self.cmyk.is_none() && self.hsv.is_none() && self.hsl.is_none()
    }
}

// this is the code from colorpicker.js ported to rust
// see that file for credits
fn hsv_to_hsl((h, s, v): (f64, f64, f64)) -> (f64, f64, f64) {
    let l = v - (v * s) / 2.;
    let m = f64::min(l, 1. - l);
    (h, if m != 0. { (v - l) / m } else { 0. }, l)
}
fn hsl_to_hsv((h, s, l): (f64, f64, f64)) -> (f64, f64, f64) {
    let v = s * f64::min(l, 1. - l) + l;
    (h, if v != 0. { 2. - (2. * l) / v } else { 0. }, v)
}
fn hsv_to_rgb((h, s, v): (f64, f64, f64)) -> (f64, f64, f64) {
    let f = |n: f64| {
        let k = (n + h * 6.) % 6.;
        v - v * s * f64::max(f64::min(k, 4. - k), 0.)
    };
    (f(5.), f(3.), f(1.))
}
fn rgb_to_hsv((r, g, b): (f64, f64, f64)) -> (f64, f64, f64) {
    let v = f64::max(r, f64::max(g, b));
    let c = v - f64::min(r, f64::min(g, b));
    let h = if c != 0. {
        if v == r {
            (g - b) / c
        } else if v == g {
            2. + (b - r) / c
        } else {
            4. + (r - g) / c
        }
    } else {
        0.
    };
    (
        if h < 0. { h + 6. } else { h } / 6.,
        if v != 0. { c / v } else { 0. },
        v,
    )
}
fn rgb_to_hsl((r, g, b): (f64, f64, f64)) -> (f64, f64, f64) {
    let v = f64::max(r, f64::max(g, b));
    let c = v - f64::min(r, f64::min(g, b));
    let f = 1. - f64::abs(v + v - c - 1.);
    let h = if c != 0. {
        if v == r {
            (g - b) / c
        } else if v == g {
            2. + (b - r) / c
        } else {
            4. + (r - g) / c
        }
    } else {
        0.
    };
    (
        if h < 0. { h + 6. } else { h } / 6.,
        if f != 0. { c / f } else { 0. },
        (v + v - c) / 2.,
    )
}
fn rgb_to_cmyk((r, g, b): (f64, f64, f64)) -> (f64, f64, f64, f64) {
    let k = 1. - f64::max(r, f64::max(g, b));
    if k == 1. {
        return (0., 0., 0., 1.);
    }
    let c = (1. - r - k) / (1. - k);
    let m = (1. - g - k) / (1. - k);
    let y = (1. - b - k) / (1. - k);
    (c, m, y, k)
}
fn cmyk_to_rgb((c, m, y, k): (f64, f64, f64, f64)) -> (f64, f64, f64) {
    let r = (1. - c) * (1. - k);
    let g = (1. - m) * (1. - k);
    let b = (1. - y) * (1. - k);
    (r, g, b)
}
