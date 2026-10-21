use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

/// The files embedded next to the QML: Labels.js, the music, the sounds.
fn resources() -> Vec<String> {
    let mut files = vec!["qml/Labels.js".to_string()];
    for (dir, ext) in [("music", "opus"), ("sounds", "wav")] {
        println!("cargo:rerun-if-changed={dir}");
        let mut found: Vec<String> = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("{dir}: {e}"))
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .filter(|name| name.ends_with(&format!(".{ext}")))
            .map(|name| format!("{dir}/{name}"))
            .collect();
        found.sort();
        files.extend(found);
    }
    files
}

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("omatro").qml_files([
        QmlFile::from("qml/Main.qml"),
        QmlFile::from("qml/BlindSelect.qml"),
        QmlFile::from("qml/FastFetch.qml"),
        QmlFile::from("qml/Collection.qml"),
        QmlFile::from("qml/ConsumableView.qml"),
        QmlFile::from("qml/PackView.qml"),
        QmlFile::from("qml/PackOpening.qml"),
        QmlFile::from("qml/VoucherView.qml"),
        QmlFile::from("qml/BadgeView.qml"),
        QmlFile::from("qml/NewGame.qml"),
        QmlFile::from("qml/ResumeGame.qml"),
        QmlFile::from("qml/Glossary.qml"),
        QmlFile::from("qml/Fan.qml"),
        QmlFile::from("qml/ScoreFire.qml"),
        QmlFile::from("qml/Sparks.qml"),
        QmlFile::from("qml/CardView.qml"),
        QmlFile::from("qml/JokerView.qml"),
        QmlFile::from("qml/Button.qml"),
        QmlFile::from("qml/SeedLabel.qml"),
        QmlFile::from("qml/Music.qml"),
        QmlFile::from("qml/Sounds.qml"),
        QmlFile::from("qml/Colors.qml").singleton(true),
        QmlFile::from("qml/Style.qml").singleton(true),
        QmlFile::from("qml/Util.qml").singleton(true),
        QmlFile::from("qml/Hints.qml").singleton(true),
        QmlFile::from("qml/I18n.qml").singleton(true),
    ]))
    // .js files are not module files: they go next to them, in the resources.
    // So are the music (Music.qml) and the sound effects (Sounds.qml).
    .qrc_resources(resources())
    .files(["src/bridge.rs"])
    .build();
}
