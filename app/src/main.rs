//! omatro : un jeu de cartes à la Balatro, aux couleurs du thème Omarchy.

mod bridge;
mod theme;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    let mut app = QGuiApplication::new();
    // Relie la fenêtre à omatro.desktop (app_id Wayland, icône).
    QGuiApplication::set_desktop_file_name(&QString::from("omatro"));
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/omatro/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
