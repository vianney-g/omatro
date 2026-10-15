import QtQuick
import omatro

// A pull request (voucher) for sale: name, what it changes for good, quip.
Rectangle {
  id: root

  property var voucher: null

  width: Style.space(120)
  height: Style.space(150)
  radius: Style.cornerRadius
  clip: true
  color: Qt.tint(Color.background, Util.alpha(Color.accent, 0.14))
  border.width: Style.space(2)
  border.color: Color.accent

  Column {
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
    spacing: Style.space(4)

    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      text: "pull request"
      color: Color.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.voucher ? root.voucher.name : ""
      color: Color.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.subtitle
      font.bold: true
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.voucher ? root.voucher.description : ""
      color: Color.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.voucher ? root.voucher.quip : ""
      color: Color.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.italic: true
    }
  }
}
