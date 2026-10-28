import QtQuick
import omatro

// The back of a card: Omarchy's logo in a double frame.
Rectangle {
  radius: Style.cornerRadius
  color: Qt.tint(Colors.background, Util.alpha(Colors.accent, 0.25))
  border.width: Style.space(2)
  border.color: Util.alpha(Colors.accent, 0.6)
  Rectangle {
    anchors { fill: parent; margins: Style.space(6) }
    radius: Style.cornerRadius
    color: "transparent"
    border.width: Style.normalBorderWidth
    border.color: Util.alpha(Colors.accent, 0.5)
  }
  OmarchyLogo {
    anchors.centerIn: parent
    width: Math.round(parent.width * 0.55)
    height: implicitHeight
    color: Colors.accent
  }
}
