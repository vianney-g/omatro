use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("omatro").qml_files([
        QmlFile::from("qml/Main.qml"),
        QmlFile::from("qml/BlindSelect.qml"),
        QmlFile::from("qml/FastFetch.qml"),
        QmlFile::from("qml/ConsumableView.qml"),
        QmlFile::from("qml/PackView.qml"),
        QmlFile::from("qml/PackOpening.qml"),
        QmlFile::from("qml/VoucherView.qml"),
        QmlFile::from("qml/CardView.qml"),
        QmlFile::from("qml/JokerView.qml"),
        QmlFile::from("qml/Button.qml"),
        QmlFile::from("qml/Color.qml").singleton(true),
        QmlFile::from("qml/Style.qml").singleton(true),
        QmlFile::from("qml/Util.qml").singleton(true),
    ]))
    // .js files are not module files: they go next to them, in the resources.
    .qrc_resources(["qml/Labels.js"])
    .files(["src/bridge.rs"])
    .build();
}
