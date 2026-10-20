import QtQuick
import omatro

// Button in the theme colors.
Rectangle {
  id: root

  property string text: ""
  property bool bordered: true
  // Less padding, for buttons under a joker.
  property bool compact: false
  // Items after the text, part of the button (Skip and its badge).
  default property alias extra: extraRow.data
  readonly property bool hovered: mouse.containsMouse
  signal clicked()

  implicitWidth: content.implicitWidth + 2 * Style.space(compact ? 6 : 14)
  implicitHeight: content.implicitHeight + 2 * Style.space(compact ? 5 : 8)
  radius: Style.cornerRadius
  color: !enabled ? Style.normalFill
    : mouse.pressed ? Style.pressedFill
    : mouse.containsMouse ? Style.hoverFill : Style.normalFill
  border.width: bordered ? Style.normalBorderWidth : 0
  border.color: mouse.containsMouse && enabled ? Colors.accent : Style.normalBorderColor

  Row {
    id: content
    anchors.centerIn: parent
    spacing: Style.spacing.md
    Text {
      anchors.verticalCenter: parent.verticalCenter
      text: root.text
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }
    Row {
      id: extraRow
      anchors.verticalCenter: parent.verticalCenter
      visible: visibleChildren.length > 0
    }
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
