import QtQuick
import omatro
import "Labels.js" as Labels
import "Drawn.js" as Drawn

// A consumable (theme, command or oligarchy card): kind, name, effect and quip.
// Oligarchy cards stand out in the urgent colour; a Transparent one takes no slot.
// A theme with a picture (art/themes/<id>.png, in the theme's own colours, not
// the current theme's) shows it at the top, framed, in place of its kind.
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
  readonly property bool pictured: item !== null && item.kind === "theme" && Drawn.themes[item.id] === true
  color: Qt.tint(Colors.background, Util.alpha(oligarchy ? Colors.urgent : Colors.foreground, transparent ? 0.03 : 0.08))
  border.width: oligarchy || transparent ? Style.space(2) : Style.normalBorderWidth
  border.color: oligarchy ? Colors.urgent : transparent ? Colors.accent : Colors.foreground

  Column {
    id: content
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
    spacing: Style.space(4)

    // Framed, so that a light theme's picture holds on a dark card and the
    // other way round.
    Rectangle {
      visible: root.pictured
      width: parent.width
      height: Math.round(width * 40 / 64)
      color: "transparent"
      border.width: Style.normalBorderWidth
      border.color: Colors.foreground
      Image {
        anchors { fill: parent; margins: parent.border.width }
        smooth: false
        fillMode: Image.Stretch
        source: root.pictured ? "../art/themes/" + root.item.id + ".png" : ""
      }
    }
    Text {
      width: parent.width
      visible: !root.pictured
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
    // A pictured theme says what it does now alone: the level it reaches
    // names the hand already.
    Text {
      width: parent.width
      visible: !(root.pictured && root.item.current)
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
  }
  // The quip, under the rest when there is room left (and nothing said in
  // its place).
  Text {
    anchors { left: content.left; right: content.right; top: content.bottom; topMargin: content.spacing }
    horizontalAlignment: Text.AlignHCenter
    wrapMode: Text.WordWrap
    visible: !root.narrow && !(root.item && root.item.current)
      && content.y + content.height + content.spacing + implicitHeight <= root.height - Style.space(4)
    text: root.item ? root.item.quip : ""
    color: Colors.muted
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
    font.italic: true
  }
}
