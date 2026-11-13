import QtQuick
import omatro
import "Labels.js" as Labels
import "Drawn.js" as Drawn

// A consumable (theme, command or oligarchy card): kind, name, effect and quip.
// Oligarchy cards stand out in the urgent colour; a Transparent one takes no slot.
// A theme with a picture (art/themes/<id>.png, in the theme's own colours, not
// the current theme's) or a drawn oligarchy card (art/oligarchy, in the
// current theme's, as jokers) shows it at the top, framed, in place of its kind;
// with its name and what it would do now only, its full view showing beside it
// while hovered (CardPeek, through Peek), as drawn jokers do.
Rectangle {
  id: root

  property var item: null
  // The full view: every text, the card as tall as they need (CardPeek).
  property bool full: false
  readonly property bool compact: pictured && !full
  // Hovered, its terms are explained beside it (Glossary), or its full view
  // shows, which explains them.
  HoverHandler {
    onHoveredChanged: {
      if (root.compact) hovered ? Peek.showConsumable(root, root.item, "") : Peek.hide(root)
      else hovered ? Hints.show(root, root.item ? root.item.description : "", root.item ? root.item.name : "") : Hints.hide(root)
    }
  }
  Component.onDestruction: Peek.hide(root)

  width: Style.space(120)
  height: full
    ? Math.max(Style.space(150), content.y + content.height + (quip.visible ? content.spacing + quip.implicitHeight : 0) + Style.space(8))
    : Style.space(150)
  radius: Style.cornerRadius
  clip: true
  // Narrow (three or more in the clipboard): no quip, smaller name.
  readonly property bool narrow: width < Style.space(110)
  readonly property bool oligarchy: item !== null && item.kind === "oligarchy"
  readonly property bool transparent: item !== null && item.transparent === true
  readonly property bool pictured: item !== null
    && (item.kind === "theme" ? Drawn.themes[item.id] === true
      : item.kind === "oligarchy" && Drawn.oligarchy[item.id] === true)
  color: Qt.tint(Colors.background, Util.alpha(oligarchy ? Colors.urgent : Colors.foreground, transparent ? 0.03 : 0.08))
  border.width: oligarchy || transparent ? Style.space(2) : Style.normalBorderWidth
  border.color: oligarchy ? Colors.urgent : transparent ? Colors.accent : Colors.foreground

  Column {
    id: content
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
    // Compact, the picture and the name in the middle of the card.
    anchors.topMargin: root.compact ? Math.max(Style.space(8), (root.height - height) / 2) : Style.space(8)
    spacing: Style.space(4)

    // Framed, so that a light theme's picture holds on a dark card and the
    // other way round.
    Rectangle {
      visible: root.pictured
      width: parent.width
      height: Math.round(width * 40 / 64)
      color: "transparent"
      border.width: Style.normalBorderWidth
      border.color: root.oligarchy ? Colors.urgent : Colors.foreground
      Image {
        anchors { fill: parent; margins: parent.border.width }
        visible: !root.oligarchy
        smooth: false
        fillMode: Image.Stretch
        source: root.pictured && !root.oligarchy ? "../art/themes/" + root.item.id + ".png" : ""
      }
      JokerArt {
        anchors { fill: parent; margins: parent.border.width }
        visible: root.oligarchy
        family: "oligarchy"
        jokerId: root.pictured && root.oligarchy ? root.item.id : ""
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
    // Compact, what it does now alone: a theme's level names the hand
    // already, the rest is in the full view.
    Text {
      width: parent.width
      visible: !root.compact
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
    id: quip
    anchors { left: content.left; right: content.right; top: content.bottom; topMargin: content.spacing }
    horizontalAlignment: Text.AlignHCenter
    wrapMode: Text.WordWrap
    visible: !root.narrow && !root.compact && !(root.item && root.item.current)
      && (root.full || content.y + content.height + content.spacing + implicitHeight <= root.height - Style.space(4))
    text: root.item ? root.item.quip : ""
    color: Colors.muted
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
    font.italic: true
  }
}
