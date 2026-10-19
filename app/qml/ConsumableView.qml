import QtQuick
import omatro
import "Labels.js" as Labels

// A consumable (theme, command or oligarchy card): kind, name, effect and quip.
// Oligarchy cards stand out in the urgent colour; a Transparent one takes no slot.
Rectangle {
  id: root

  property var item: null
  // Hovered, its terms are explained beside it (Glossary).
  HoverHandler {
    onHoveredChanged: hovered ? Hints.show(root, root.item ? root.item.description : "", root.item ? root.item.name : "") : Hints.hide(root)
  }


  width: Style.space(120)
  height: Style.space(150)
  radius: Style.cornerRadius
  clip: true
  // Narrow (three or more in the clipboard): no quip, smaller name.
  readonly property bool narrow: width < Style.space(110)
  readonly property bool oligarchy: item !== null && item.kind === "oligarchy"
  readonly property bool transparent: item !== null && item.transparent === true
  color: Qt.tint(Colors.background, Util.alpha(oligarchy ? Colors.urgent : Colors.foreground, transparent ? 0.03 : 0.08))
  border.width: oligarchy || transparent ? Style.space(2) : Style.normalBorderWidth
  border.color: oligarchy ? Colors.urgent : transparent ? Colors.accent : Colors.foreground

  Column {
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
    spacing: Style.space(4)

    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      text: !root.item ? ""
        : root.transparent ? (root.narrow ? I18n.tr("transparent") : I18n.tr("{kind} · transparent", { kind: I18n.tr(root.item.kind) }))
        : I18n.tr(root.item.kind)
      color: root.transparent ? Colors.accent : Colors.muted
      fontSizeMode: Text.HorizontalFit
      minimumPixelSize: Style.fontPx(0.6)
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.item ? root.item.name : ""
      color: root.oligarchy ? Colors.urgent : Colors.foreground
      font.family: Style.font.family
      font.pixelSize: root.narrow ? Style.font.caption : Style.font.subtitle
      font.bold: true
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      textFormat: Text.StyledText
      text: root.item ? Labels.highlight(root.item.description, Hints.terms, root.item.name) : ""
      color: Colors.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    // What it would do now (the level a theme reaches, money…), in place
    // of the quip.
    Text {
      width: parent.width
      visible: text !== ""
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.item && root.item.current ? root.item.current : ""
      color: Colors.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.bold: true
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      visible: !root.narrow && !(root.item && root.item.current)
      text: root.item ? root.item.quip : ""
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.italic: true
    }
  }
}
