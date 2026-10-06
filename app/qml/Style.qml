pragma Singleton
import QtQuick
import omatro

// Mesures et états des contrôles, d'après les valeurs par défaut du shell
// Omarchy. Tout suit la taille de police d'Omarchy (`omarchy display text size`).
QtObject {
  id: root

  readonly property int cornerRadius: OmarchyTheme.cornerRadius
  readonly property real scale: Math.max(1 / 12, OmarchyTheme.fontBaseSize / 12)

  function space(px) { return Math.round(px * scale) }
  function fontPx(mult) { return Math.max(1, Math.round(OmarchyTheme.fontBaseSize * mult)) }

  readonly property int normalBorderWidth: 1
  readonly property real normalBorderAlpha: 0.4
  readonly property color normalFill: Util.alpha(Color.foreground, 0.04)
  readonly property color hoverFill: Util.alpha(Color.foreground, 0.08)
  readonly property color pressedFill: Util.alpha(Color.foreground, 0.22)
  readonly property color normalBorderColor: Util.alpha(Color.foreground, normalBorderAlpha)

  readonly property QtObject spacing: QtObject {
    readonly property int xs: root.space(3)
    readonly property int sm: root.space(4)
    readonly property int md: root.space(6)
    readonly property int lg: root.space(8)
    readonly property int xl: root.space(10)
    readonly property int xxl: root.space(12)
    readonly property int huge: root.space(18)
  }

  // « monospace » est l'alias fontconfig que règle `omarchy font set`.
  readonly property QtObject font: QtObject {
    readonly property string family: "monospace"
    readonly property int caption: root.fontPx(0.833)
    readonly property int body: root.fontPx(1.0)
    readonly property int subtitle: root.fontPx(1.083)
    readonly property int title: root.fontPx(1.167)
    readonly property int heading: root.fontPx(1.333)
    readonly property int display: root.fontPx(2.0)
    readonly property int displayLarge: root.fontPx(2.333)
  }
}
