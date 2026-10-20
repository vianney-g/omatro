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
  // The width it may take: past it, the extra items go under the text.
  property real maxWidth: Infinity
  readonly property real padding: 2 * Style.space(compact ? 6 : 14)
  signal clicked()

  implicitWidth: content.implicitWidth + padding
  implicitHeight: content.implicitHeight + 2 * Style.space(compact ? 5 : 8)
  radius: Style.cornerRadius
  color: !enabled ? Style.normalFill
    : mouse.pressed ? Style.pressedFill
    : mouse.containsMouse ? Style.hoverFill : Style.normalFill
  border.width: bordered ? Style.normalBorderWidth : 0
  border.color: mouse.containsMouse && enabled ? Colors.accent : Style.normalBorderColor

  Grid {
    id: content
    anchors.centerIn: parent
    columns: extraRow.visible
      && label.implicitWidth + columnSpacing + extraRow.implicitWidth + root.padding > root.maxWidth ? 1 : 2
    columnSpacing: Style.spacing.md
    rowSpacing: Style.spacing.sm
    horizontalItemAlignment: Grid.AlignHCenter
    verticalItemAlignment: Grid.AlignVCenter
    Text {
      id: label
      text: root.text
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }
    Row {
      id: extraRow
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
