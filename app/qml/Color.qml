pragma Singleton
import QtQuick
import omatro

// Omarchy theme colors, under the same names as in the shell.
QtObject {
  readonly property color background: OmarchyTheme.background
  readonly property color foreground: OmarchyTheme.foreground
  readonly property color accent: OmarchyTheme.accent
  readonly property color urgent: OmarchyTheme.urgent
  readonly property color muted: OmarchyTheme.muted
  // The theme's yellow: fire and sparks.
  readonly property color warm: OmarchyTheme.warm
}
