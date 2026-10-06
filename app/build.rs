use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("omatro").qml_files([
        QmlFile::from("qml/Main.qml"),
        QmlFile::from("qml/CardView.qml"),
        QmlFile::from("qml/JokerView.qml"),
        QmlFile::from("qml/Button.qml"),
        QmlFile::from("qml/Color.qml").singleton(true),
        QmlFile::from("qml/Style.qml").singleton(true),
        QmlFile::from("qml/Util.qml").singleton(true),
    ]))
    // Les .js ne sont pas des fichiers du module : on les met à côté, dans les ressources.
    .qrc_resources(["qml/Labels.js"])
    .files(["src/bridge.rs"])
    .build();
}
