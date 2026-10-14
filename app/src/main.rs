//! omatro: a Balatro-like card game, in the colors of the Omarchy theme.

mod bridge;
mod theme;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    let mut app = QGuiApplication::new();
    // Ties the window to omatro.desktop (Wayland app_id, icon).
    QGuiApplication::set_desktop_file_name(&QString::from("omatro"));
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/omatro/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
