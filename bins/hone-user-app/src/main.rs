#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Url;
use tauri::{
    WebviewUrl,
    webview::{Color, NewWindowResponse, PageLoadEvent, WebviewWindowBuilder},
};

const HONE_HOST: &str = "hone-claw.com";

fn is_first_party_navigation(url: &Url) -> bool {
    match url.scheme() {
        "tauri" | "asset" | "about" => true,
        "https" => matches!(url.host_str(), Some(HONE_HOST) | Some("www.hone-claw.com")),
        _ => false,
    }
}

fn can_open_externally(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https" | "mailto")
}

fn open_external_url(url: &Url) {
    if can_open_externally(url)
        && let Err(error) = open::that_detached(url.as_str())
    {
        eprintln!("[hone-user-app] failed to open external URL: {error}");
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .title("HONE")
                    .inner_size(1280.0, 840.0)
                    .min_inner_size(860.0, 620.0)
                    .center()
                    .visible(false)
                    .background_color(Color(247, 245, 239, 255))
                    .on_navigation(|url| {
                        if is_first_party_navigation(url) {
                            true
                        } else {
                            open_external_url(url);
                            false
                        }
                    })
                    .on_new_window(|url, _| {
                        open_external_url(&url);
                        NewWindowResponse::Deny
                    })
                    .on_page_load(|window, payload| {
                        eprintln!(
                            "[hone-user-app] page {:?}: {}",
                            payload.event(),
                            payload.url()
                        );
                        if matches!(payload.event(), PageLoadEvent::Finished) {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    });

            #[cfg(target_os = "macos")]
            let window = window.hidden_title(true).allow_link_preview(false);

            window.build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run HONE user app");
}

#[cfg(test)]
mod tests {
    use super::{can_open_externally, is_first_party_navigation};
    use tauri::Url;

    const APP_SCRIPT: &str = include_str!("../ui/app.js");
    const TAURI_CONFIG: &str = include_str!("../tauri.conf.json");

    #[test]
    fn keeps_only_hone_and_bundled_pages_inside_the_app() {
        for url in [
            "tauri://localhost/index.html",
            "asset://localhost/index.html",
            "about:blank",
            "https://hone-claw.com/chat",
            "https://www.hone-claw.com/privacy",
        ] {
            assert!(
                is_first_party_navigation(&Url::parse(url).unwrap()),
                "{url}"
            );
        }

        for url in [
            "http://hone-claw.com/chat",
            "https://evil.example/?next=https://hone-claw.com/chat",
            "https://github.com/B-M-Capital-Research/honeclaw",
            "file:///tmp/private.txt",
        ] {
            assert!(
                !is_first_party_navigation(&Url::parse(url).unwrap()),
                "{url}"
            );
        }
    }

    #[test]
    fn opens_only_browser_and_mail_links_externally() {
        for url in [
            "https://github.com/B-M-Capital-Research/honeclaw",
            "http://example.com",
            "mailto:bm@hone-claw.com",
        ] {
            assert!(can_open_externally(&Url::parse(url).unwrap()), "{url}");
        }
        for url in [
            "file:///tmp/private.txt",
            "javascript:alert(1)",
            "data:text/plain,no",
        ] {
            assert!(!can_open_externally(&Url::parse(url).unwrap()), "{url}");
        }
    }

    #[test]
    fn bundle_contract_stays_remote_only() {
        assert!(APP_SCRIPT.contains("https://hone-claw.com/chat"));
        assert!(TAURI_CONFIG.contains("\"signingIdentity\": \"-\""));

        for forbidden in [
            "externalBin",
            "\"resources\"",
            "hone-mcp",
            "hone-feishu",
            "hone-discord",
            "hone-telegram",
            "hone-imessage",
            "opencode",
            "codex",
        ] {
            assert!(!TAURI_CONFIG.contains(forbidden), "found {forbidden}");
        }
    }
}
