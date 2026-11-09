pragma Singleton
import QtQuick
import omatro

// Omarchy theme colors, under the same names as in the shell (not "Color":
// QtQuick has a type of that name since Qt 6.12).
QtObject {
  readonly property color background: OmarchyTheme.background
  readonly property color foreground: OmarchyTheme.foreground
  readonly property color accent: OmarchyTheme.accent
  readonly property color urgent: OmarchyTheme.urgent
  readonly property color muted: OmarchyTheme.muted
  // The theme's yellow: fire and sparks.
  readonly property color warm: OmarchyTheme.warm
  // More of the theme's hues, one per card enhancement (SuitMark).
  readonly property color green: OmarchyTheme.green
  readonly property color cyan: OmarchyTheme.cyan
  readonly property color magenta: OmarchyTheme.magenta
  readonly property color orange: OmarchyTheme.orange
}
