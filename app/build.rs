use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};
use qt_build_utils::QResourceFile;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Qt's shader tool: next to qmake's host binaries (QMAKE names qmake, as
/// for cxx-qt), or on the PATH. It comes with qt6-shadertools.
fn qsb() -> PathBuf {
    let qmake = std::env::var("QMAKE").unwrap_or_else(|_| "qmake6".to_string());
    let bins = Command::new(&qmake)
        .args(["-query", "QT_HOST_BINS"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string());
    match bins {
        Some(dir) if Path::new(&dir).join("qsb").exists() => Path::new(&dir).join("qsb"),
        _ => PathBuf::from("qsb"),
    }
}

/// The shaders of `shaders/`, compiled for every graphics API Qt Quick may use,
/// and embedded as `shaders/<name>.qsb` next to the QML.
fn shaders() -> Vec<QResourceFile> {
    println!("cargo:rerun-if-changed=shaders");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let qsb = qsb();
    let mut names: Vec<String> = std::fs::read_dir("shaders")
        .unwrap_or_else(|e| panic!("shaders: {e}"))
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.ends_with(".frag") || name.ends_with(".vert"))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let compiled = out.join(format!("{name}.qsb"));
            let status = Command::new(&qsb)
                .args(["--glsl", "100es,120,150", "--hlsl", "50", "--msl", "12", "-o"])
                .arg(&compiled)
                .arg(format!("shaders/{name}"))
                .status()
                .unwrap_or_else(|e| panic!("{}: {e} (qsb comes with qt6-shadertools)", qsb.display()));
            assert!(status.success(), "qsb could not compile shaders/{name}");
            QResourceFile::new(compiled).alias(format!("shaders/{name}.qsb"))
        })
        .collect()
}

/// The palette letters of the pixel art, in index order: transparent; the
/// theme's foreground, accent, muted, background, warm, urgent, green, cyan,
/// magenta and orange; `d` ink, always dark, and `l` light, always light (for
/// outlines and highlights, whatever the theme); then, in capitals, the dark
/// shades of accent, warm, urgent, green, cyan, magenta and orange.
/// `shaders/palette.frag` and `art/art.py` follow the same order.
const ART_PALETTE: &str = ".kambwrgcpodlAWRGCPO";

/// The drawings of `art/<dir>/<id>.txt`: a grid of palette letters (64 × 64
/// for a joker, 64 × 40 for an oligarchy card), `#` lines being comments. Each
/// becomes a greyscale PGM image, `art/<dir>/<id>.pgm`, whose grey is the
/// palette index × 8, that `shaders/palette.frag` paints in the theme's
/// colours. Gives the files and the ids drawn.
fn drawings(dir: &str, out: &Path) -> (Vec<QResourceFile>, Vec<String>) {
    println!("cargo:rerun-if-changed=art/{dir}");
    let out = out.join(dir);
    std::fs::create_dir_all(&out).unwrap();
    let mut names: Vec<String> = std::fs::read_dir(format!("art/{dir}"))
        .unwrap_or_else(|e| panic!("art/{dir}: {e}"))
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.ends_with(".txt"))
        .collect();
    names.sort();
    let mut files = Vec::new();
    let mut ids = Vec::new();
    for name in names {
        let stem = name.trim_end_matches(".txt");
        let text = std::fs::read_to_string(format!("art/{dir}/{name}")).unwrap();
        let rows: Vec<&str> = text
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        let width = rows.first().map_or(0, |r| r.chars().count());
        assert!(
            width > 0 && rows.iter().all(|r| r.chars().count() == width),
            "art/{dir}/{name}: rows of letters of the same length expected"
        );
        let mut pgm = format!("P5 {width} {} 255\n", rows.len()).into_bytes();
        for c in rows.iter().flat_map(|r| r.chars()) {
            let index = ART_PALETTE
                .find(c)
                .unwrap_or_else(|| panic!("art/{dir}/{name}: unknown letter {c:?}"));
            pgm.push(index as u8 * 8);
        }
        let file = out.join(format!("{stem}.pgm"));
        std::fs::write(&file, pgm).unwrap();
        files.push(QResourceFile::new(file).alias(format!("art/{dir}/{stem}.pgm")));
        ids.push(format!("{stem:?}: true"));
    }
    (files, ids)
}

/// The joker drawings (`art/jokers`) and the oligarchy cards' (`art/oligarchy`),
/// see `drawings`; the theme pictures of `art/themes/<id>.png` keep their own
/// colours (those of the theme they show): they go in as they are. `qml/Drawn.js`
/// lists what is drawn (`Drawn.ids` for the jokers, `Drawn.oligarchy`,
/// `Drawn.themes`), so that the others are not looked for.
fn art() -> Vec<QResourceFile> {
    println!("cargo:rerun-if-changed=art/themes");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("art");
    let (mut files, jokers) = drawings("jokers", &out);
    let (oligarchy_files, oligarchy) = drawings("oligarchy", &out);
    files.extend(oligarchy_files);
    let mut themes: Vec<String> = std::fs::read_dir("art/themes")
        .unwrap_or_else(|e| panic!("art/themes: {e}"))
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter_map(|name| Some(name.strip_suffix(".png")?.to_string()))
        .collect();
    themes.sort();
    for theme in &themes {
        let path = PathBuf::from(format!("art/themes/{theme}.png")).canonicalize().unwrap();
        files.push(QResourceFile::new(path).alias(format!("art/themes/{theme}.png")));
    }
    let themes: Vec<String> = themes.iter().map(|t| format!("{t:?}: true")).collect();
    let drawn = out.join("Drawn.js");
    std::fs::write(
        &drawn,
        format!(
            ".pragma library\nvar ids = {{ {} }}\nvar oligarchy = {{ {} }}\nvar themes = {{ {} }}\n",
            jokers.join(", "),
            oligarchy.join(", "),
            themes.join(", ")
        ),
    )
    .unwrap();
    files.push(QResourceFile::new(drawn).alias("qml/Drawn.js"));
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
        QmlFile::from("qml/CardRow.qml"),
        QmlFile::from("qml/CardPeek.qml"),
        QmlFile::from("qml/ScoreFire.qml"),
        QmlFile::from("qml/Sparks.qml"),
        QmlFile::from("qml/CardView.qml"),
        QmlFile::from("qml/SuitMark.qml"),
        QmlFile::from("qml/BossScene.qml"),
        QmlFile::from("qml/EditionLayer.qml"),
        QmlFile::from("qml/Odometer.qml"),
        QmlFile::from("qml/KernelPanic.qml"),
        QmlFile::from("qml/Confetti.qml"),
        QmlFile::from("qml/Coins.qml"),
        QmlFile::from("qml/SealMark.qml"),
        QmlFile::from("qml/JokerView.qml"),
        QmlFile::from("qml/JokerArt.qml"),
        QmlFile::from("qml/Button.qml"),
        QmlFile::from("qml/SeedLabel.qml"),
        QmlFile::from("qml/OmarchyLogo.qml"),
        QmlFile::from("qml/CardBack.qml"),
        QmlFile::from("qml/DeckView.qml"),
        QmlFile::from("qml/Music.qml"),
        QmlFile::from("qml/Sounds.qml"),
        QmlFile::from("qml/Options.qml"),
        QmlFile::from("qml/VolumeControl.qml"),
        QmlFile::from("qml/Colors.qml").singleton(true),
        QmlFile::from("qml/Style.qml").singleton(true),
        QmlFile::from("qml/Util.qml").singleton(true),
        QmlFile::from("qml/Hints.qml").singleton(true),
        QmlFile::from("qml/Peek.qml").singleton(true),
        QmlFile::from("qml/I18n.qml").singleton(true),
    ]))
    // .js files are not module files: they go next to them, in the resources.
    // So are the music (Music.qml) and the sound effects (Sounds.qml).
    .qrc_resources(resources())
    // Compiled at build time: the .qsb files are not kept in the repository.
    .qrc_resources(shaders())
    .qrc_resources(art())
    .files(["src/bridge.rs"])
    .build();
}
