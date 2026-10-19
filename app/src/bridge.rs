//! The bridge between Rust and QML: the game engine and the Omarchy theme.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
    }

    extern "RustQt" {
        /// The engine, behind the same JSON protocol as the `omatro-engine`
        /// binary: one command in, one response out.
        #[qobject]
        #[qml_element]
        type Engine = super::EngineRust;

        #[qinvokable]
        fn handle(self: Pin<&mut Engine>, line: &QString) -> QString;

        /// Debug only: the steps of `OMATRO_SCRIPT`, which the interface plays
        /// to take screenshots without a keyboard (see CLAUDE.md).
        #[qinvokable]
        fn script(self: &Engine) -> QString;
    }

    extern "RustQt" {
        /// The active Omarchy theme, reloaded when it changes.
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QColor, background)]
        #[qproperty(QColor, foreground)]
        #[qproperty(QColor, accent)]
        #[qproperty(QColor, urgent)]
        #[qproperty(QColor, muted)]
        #[qproperty(QColor, warm)]
        #[qproperty(i32, font_base_size, cxx_name = "fontBaseSize")]
        #[qproperty(i32, corner_radius, cxx_name = "cornerRadius")]
        type OmarchyTheme = super::ThemeRust;

        /// Reloads the theme if its files have changed.
        #[qinvokable]
        fn refresh(self: Pin<&mut OmarchyTheme>);
    }

    impl cxx_qt::Initialize for OmarchyTheme {}
}

use crate::theme::{self, Theme};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QString};
use omatro_engine::protocol::{Session, default_save_path};
use std::path::PathBuf;
use std::time::SystemTime;

pub struct EngineRust {
    session: Session,
}

impl Default for EngineRust {
    fn default() -> Self {
        EngineRust {
            // The game in progress is kept between runs.
            session: match default_save_path() {
                Some(path) => Session::from_env().with_save(path),
                None => Session::from_env(),
            },
        }
    }
}

impl qobject::Engine {
    pub fn handle(self: Pin<&mut Self>, line: &QString) -> QString {
        // Translated into the language the interface set.
        let reply = self.rust_mut().session.handle_json(&line.to_string());
        QString::from(&reply)
    }

    pub fn script(&self) -> QString {
        let debug = std::env::var("OMATRO_DEBUG").is_ok_and(|v| v == "1");
        let script = std::env::var("OMATRO_SCRIPT").unwrap_or_default();
        QString::from(if debug { script.as_str() } else { "" })
    }
}

pub struct ThemeRust {
    background: QColor,
    foreground: QColor,
    accent: QColor,
    urgent: QColor,
    muted: QColor,
    warm: QColor,
    font_base_size: i32,
    corner_radius: i32,
    fingerprint: Vec<(PathBuf, Option<SystemTime>)>,
}

impl Default for ThemeRust {
    fn default() -> Self {
        ThemeRust {
            background: QColor::default(),
            foreground: QColor::default(),
            accent: QColor::default(),
            urgent: QColor::default(),
            muted: QColor::default(),
            warm: QColor::default(),
            font_base_size: 12,
            corner_radius: 0,
            fingerprint: Vec::new(),
        }
    }
}

fn color(hex: &str) -> QColor {
    QColor::try_from(&QString::from(hex)).unwrap_or_default()
}

impl cxx_qt::Initialize for qobject::OmarchyTheme {
    fn initialize(self: Pin<&mut Self>) {
        self.refresh();
    }
}

impl qobject::OmarchyTheme {
    pub fn refresh(mut self: Pin<&mut Self>) {
        let fingerprint = theme::fingerprint();
        if fingerprint == self.fingerprint {
            return;
        }
        self.as_mut().rust_mut().fingerprint = fingerprint;
        let Theme {
            palette,
            font_base_size,
            corner_radius,
        } = Theme::load();
        self.as_mut().set_background(color(&palette.background));
        self.as_mut().set_foreground(color(&palette.foreground));
        self.as_mut().set_accent(color(&palette.accent));
        self.as_mut().set_urgent(color(&palette.urgent));
        self.as_mut().set_muted(color(&palette.muted));
        self.as_mut().set_warm(color(&palette.warm));
        self.as_mut().set_font_base_size(font_base_size);
        self.as_mut().set_corner_radius(corner_radius);
    }
}
