import QtQuick
import omatro

// A pack for sale: name, what it holds, quip.
Rectangle {
  id: root

  property var pack: null

  width: Style.space(120)
  height: Style.space(150)
  radius: Style.cornerRadius
  color: Qt.tint(Colors.background, Util.alpha(Colors.urgent, 0.10))
  border.width: Style.space(2)
  border.color: Util.alpha(Colors.urgent, 0.6)

  Column {
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
    spacing: Style.space(4)

    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      text: I18n.tr("package")
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WrapAnywhere
      text: root.pack ? root.pack.name : ""
      color: Colors.urgent
      font.family: Style.font.family
      font.pixelSize: Style.font.subtitle
      font.bold: true
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.pack ? root.pack.description : ""
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.pack ? root.pack.quip : ""
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.italic: true
    }
  }
}
