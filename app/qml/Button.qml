import QtQuick
import omatro

// Button in the theme colors.
Rectangle {
  id: root

  property string text: ""
  property bool bordered: true
  // Less padding, for buttons under a joker.
  property bool compact: false
  signal clicked()

  implicitWidth: label.implicitWidth + 2 * Style.space(compact ? 6 : 14)
  implicitHeight: label.implicitHeight + 2 * Style.space(compact ? 5 : 8)
  radius: Style.cornerRadius
  color: !enabled ? Style.normalFill
    : mouse.pressed ? Style.pressedFill
    : mouse.containsMouse ? Style.hoverFill : Style.normalFill
  border.width: bordered ? Style.normalBorderWidth : 0
  border.color: mouse.containsMouse && enabled ? Colors.accent : Style.normalBorderColor

  Text {
    id: label
    anchors.centerIn: parent
    text: root.text
    color: Colors.foreground
    font.family: Style.font.family
    font.pixelSize: Style.font.body
  }

  MouseArea {
    id: mouse
    anchors.fill: parent
    enabled: root.enabled
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    onClicked: root.clicked()
  }
}
