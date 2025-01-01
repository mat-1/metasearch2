use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Form,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use maud::{html, Markup, PreEscaped, DOCTYPE};
use serde::{Deserialize, Serialize};

use crate::{config::Config, web::head_html};

pub async fn get(Extension(config): Extension<Config>) -> impl IntoResponse {
    let theme_option = |value: &str, name: &str| -> Markup {
        let selected = config.ui.stylesheet_url == value;
        html! {
            option value=(value) selected[selected] {
                { (name) }
            }
        }
    };

    let html = html! {
        (PreEscaped("<!-- source code: https://github.com/mat-1/metasearch2 -->\n"))
        (DOCTYPE)
        html lang="en" {
            {(head_html(Some("settings"), &config))}
            body {
                div.main-container.settings-page {
                    main {
                        a.back-to-index-button href="/" { "Back" }
                        h1 { "Settings" }
                        form.settings-form method="post" {
                            label for="theme" { "Theme" }
                            select name="stylesheet-url" selected=(config.ui.stylesheet_url) {
                                { (theme_option("", "Ayu Dark")) }
                                { (theme_option("/themes/catppuccin-mocha.css", "Catppuccin Mocha")) }
                                { (theme_option("/themes/catppuccin-macchiato.css", "Catppuccin Macchiato")) }
                                { (theme_option("/themes/catppuccin-latte.css", "Catppuccin Latte")) }
                                { (theme_option("/themes/nord-bluish.css", "Nord Bluish")) }
                                { (theme_option("/themes/discord.css", "Discord")) }
                            }

                            br;

                            // custom css textarea
                            details #custom-css-details {
                                summary { "Custom CSS" }
                                textarea #custom-css name="stylesheet-str"  {
                                    { (config.ui.stylesheet_str) }
                                }
                            }

                            input #save-settings-button type="submit" value="Save";
                        }
                    }
                }
            }
        }
    }
    .into_string();

    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    pub stylesheet_url: String,
    pub stylesheet_str: String,
}

pub async fn post(mut jar: CookieJar, Form(settings): Form<Settings>) -> impl IntoResponse {
    let mut settings_cookie = Cookie::new("settings", serde_json::to_string(&settings).unwrap());
    settings_cookie.make_permanent();
    jar = jar.add(settings_cookie);

    (StatusCode::FOUND, [(header::LOCATION, "/settings")], jar)
}
