import QtQuick
import omatro

// A consumable (for now, a theme): kind, name, effect and quip.
Rectangle {
  id: root

  property var item: null

  width: Style.space(120)
  height: Style.space(150)
  radius: Style.cornerRadius
  color: Qt.tint(Color.background, Util.alpha(Color.foreground, 0.08))
  border.width: Style.normalBorderWidth
  border.color: Color.foreground

  Column {
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
    spacing: Style.space(4)

    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      text: root.item ? root.item.kind : ""
      color: Color.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.item ? root.item.name : ""
      color: Color.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.subtitle
      font.bold: true
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.item ? root.item.description : ""
      color: Color.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.item ? root.item.quip : ""
      color: Color.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.italic: true
    }
  }
}
