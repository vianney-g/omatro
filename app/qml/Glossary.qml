import QtQuick
import omatro

// Beside the hovered card, what the terms of its text mean (Hints.shown):
// on its right, or its left when there is no room; always in the window.
// Fill the window with it, over everything.
Item {
  id: root

  readonly property Item target: Hints.target
  // Where the card is, taken when it is hovered.
  property rect place: Qt.rect(0, 0, 0, 0)
  onTargetChanged: if (target) {
    var p = target.mapToItem(root, 0, 0)
    place = Qt.rect(p.x, p.y, target.width, target.height)
  }

  Rectangle {
    id: panel
    visible: root.target !== null && Hints.shown.length > 0
    readonly property real gap: Style.space(8)
    readonly property bool onRight: root.place.x + root.place.width + gap + width <= root.width
    x: onRight ? root.place.x + root.place.width + gap : Math.max(0, root.place.x - gap - width)
    y: Math.max(0, Math.min(root.place.y, root.height - height))
    width: Style.space(220)
    height: entries.implicitHeight + 2 * Style.space(8)
    radius: Style.cornerRadius
    color: Color.background
    border.width: Style.normalBorderWidth
    border.color: Color.accent

    Column {
      id: entries
      anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
      spacing: Style.space(8)
      Repeater {
        model: Hints.shown
        Column {
          width: entries.width
          spacing: Style.space(2)
          Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: modelData.name
            color: Color.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.body
            font.bold: true
          }
          Text {
            width: parent.width
            text: modelData.category
            color: Color.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.caption
            font.italic: true
          }
          Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: modelData.description
            color: Color.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.caption
          }
        }
      }
    }
  }
}
