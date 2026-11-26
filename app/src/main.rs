//! omatro: a Balatro-like card game, in the colors of the Omarchy theme.

mod bridge;
mod theme;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use std::ffi::{CStr, c_char};
use std::path::Path;

unsafe extern "C" {
    /// The version of the Qt libraries loaded at run time ("6.12.0").
    fn qVersion() -> *const c_char;
}

/// Qt 6.12's PipeWire audio backend can freeze the app: a socket notifier
/// toggled from its thread ends up on a closed socket, and the event loop
/// spins on "QSocketNotifier: Invalid socket" forever
/// (https://forum.qt.io/post/840600). Its PulseAudio backend, through
/// pipewire-pulse, plays the same. So on 6.12 only, when a PulseAudio server
/// answers and the player hasn't chosen a backend, use that one.
/// `QT_AUDIO_BACKEND=pipewire omatro` tries the PipeWire backend again.
fn work_around_pipewire_freeze() {
    if std::env::var_os("QT_AUDIO_BACKEND").is_some() {
        return;
    }
    // SAFETY: qVersion returns a static, nul-terminated string.
    let version = unsafe { CStr::from_ptr(qVersion()) }.to_string_lossy();
    if !version.starts_with("6.12.") {
        return;
    }
    let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR") else {
        return;
    };
    if Path::new(&runtime).join("pulse/native").exists() {
        // SAFETY: no other thread runs yet (Qt starts its own later on).
        unsafe { std::env::set_var("QT_AUDIO_BACKEND", "pulseaudio") };
    }
}

fn main() {
    work_around_pipewire_freeze();
    let mut app = QGuiApplication::new();
    // Ties the window to omatro.desktop (Wayland app_id, icon).
    QGuiApplication::set_desktop_file_name(&QString::from("omatro"));
    // Settings (the last deck chosen) need an application identity: they are
    // kept in ~/.config/omatro/omatro.conf.
    if let Some(mut app) = app.as_mut() {
        app.as_mut().set_organization_name(&QString::from("omatro"));
        app.as_mut()
            .set_organization_domain(&QString::from("github.com/vianney-g"));
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
