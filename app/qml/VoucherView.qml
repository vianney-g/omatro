import QtQuick
import omatro
import "Labels.js" as Labels

// A pull request (voucher) for sale: name, what it changes for good, quip.
Rectangle {
  id: root

  property var voucher: null
  // Hovered, its terms are explained beside it (Glossary).
  HoverHandler {
    onHoveredChanged: hovered ? Hints.show(root, root.voucher ? root.voucher.description : "", root.voucher ? root.voucher.name : "") : Hints.hide(root)
  }


  width: Style.space(120)
  height: Style.space(150)
  radius: Style.cornerRadius
  clip: true
  color: Qt.tint(Colors.background, Util.alpha(Colors.accent, 0.14))
  border.width: Style.space(2)
  border.color: Colors.accent

  Column {
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
    spacing: Style.space(4)

    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      text: I18n.tr("pull request")
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.voucher ? root.voucher.name : ""
      color: Colors.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.subtitle
      font.bold: true
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      textFormat: Text.StyledText
      text: root.voucher ? Labels.highlight(root.voucher.description, Hints.terms, root.voucher.name) : ""
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.voucher ? root.voucher.quip : ""
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.italic: true
    }
  }
}
