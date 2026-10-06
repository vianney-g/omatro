pragma Singleton
import QtQuick
import omatro

// Couleurs du thème Omarchy, sous les mêmes noms que dans le shell.
QtObject {
  readonly property color background: OmarchyTheme.background
  readonly property color foreground: OmarchyTheme.foreground
  readonly property color accent: OmarchyTheme.accent
  readonly property color urgent: OmarchyTheme.urgent
  readonly property color muted: OmarchyTheme.muted
}
