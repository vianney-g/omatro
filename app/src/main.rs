//! omatro: a Balatro-like card game, in the colors of the Omarchy theme.

mod bridge;
mod theme;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    let mut app = QGuiApplication::new();
    // Ties the window to omatro.desktop (Wayland app_id, icon).
    QGuiApplication::set_desktop_file_name(&QString::from("omatro"));
    // Settings (the last deck chosen) need an application identity: they are
    // kept in ~/.config/omatro/omatro.conf.
    if let Some(mut app) = app.as_mut() {
        app.as_mut().set_organization_name(&QString::from("omatro"));
        app.as_mut().set_organization_domain(&QString::from("github.com/vianney-g"));
        app.as_mut().set_application_name(&QString::from("omatro"));
    }
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/omatro/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
