import QtQuick
import omatro

// A consumable (theme, command or oligarchy card): kind, name, effect and quip.
// Oligarchy cards stand out in the urgent colour; a Transparent one takes no slot.
Rectangle {
  id: root

  property var item: null

  width: Style.space(120)
  height: Style.space(150)
  radius: Style.cornerRadius
  clip: true
  // Narrow (three or more in the clipboard): no quip, smaller name.
  readonly property bool narrow: width < Style.space(110)
  readonly property bool oligarchy: item !== null && item.kind === "oligarchy"
  readonly property bool transparent: item !== null && item.transparent === true
  color: Qt.tint(Color.background, Util.alpha(oligarchy ? Color.urgent : Color.foreground, transparent ? 0.03 : 0.08))
  border.width: oligarchy || transparent ? Style.space(2) : Style.normalBorderWidth
  border.color: oligarchy ? Color.urgent : transparent ? Color.accent : Color.foreground

  Column {
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
    spacing: Style.space(4)

    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      text: !root.item ? ""
        : root.transparent ? (root.narrow ? "transparent" : root.item.kind + " · transparent")
        : root.item.kind
      color: root.transparent ? Color.accent : Color.muted
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
      color: root.oligarchy ? Color.urgent : Color.foreground
      font.family: Style.font.family
      font.pixelSize: root.narrow ? Style.font.caption : Style.font.subtitle
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
      visible: !root.narrow
      text: root.item ? root.item.quip : ""
      color: Color.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.italic: true
    }
  }
}
