use axum::{ http::{header, StatusCode}, response::IntoResponse, Extension, Form};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use maud::{html, Markup, PreEscaped, DOCTYPE};
use serde::Deserialize;

use crate::{config::Config, web::head_html};

pub async fn get(
    Extension(config): Extension<Config>,
) -> impl IntoResponse {
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
                            }

                            br;

                            // custom css textarea
                            details #custom-css-details {
                                summary { "Custom CSS" }
                                textarea name="stylesheet-str" id="custom-css" rows="10" cols="50" {
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

    ( [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    stylesheet_url: String,
    stylesheet_str: String,
}

pub async fn post(
    mut jar: CookieJar,
    Form(settings): Form<Settings>,
) -> impl IntoResponse {
    jar = jar.add(Cookie::new("stylesheet-url", settings.stylesheet_url));
    jar = jar.add(Cookie::new("stylesheet-str", settings.stylesheet_str));

    (
        StatusCode::FOUND,
        [(header::LOCATION, "/settings")],
        jar
    )
}
