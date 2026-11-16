import QtQuick
import omatro
import "Labels.js" as Labels
import "Drawn.js" as Drawn

// What a card says, while its small form is hovered: its name, effect, what
// it does now and its quip, then what the terms of its text mean (the
// glossary), in a bubble pointing at it (Bubble), so that it never passes
// for one more card. A consumable of the clipboard list beside its line,
// with its picture (the line has none); a joker of the joker row under the
// row (below its buttons, which stay clickable); a drawn joker, theme,
// oligarchy card or command elsewhere beside it, without its picture, which
// the hovered form shows already. Views ask through `Peek`. It takes no
// pointer, and no room.
Item {
  id: root

  Connections {
    target: Peek
    function onJokerShown(item, joker, under, hidden) { root.showJoker(item, joker, under, hidden) }
    function onConsumableShown(item, consumable, note) { root.showConsumable(item, consumable, note, false) }
    function onHidden(item) { root.hide(item) }
  }

  // The small form hovered, and what it shows.
  property Item source: null
  property var consumable: null
  property var joker: null
  // What the consumable needs, said under it ("select 1-2 cards").
  property string note: ""
  // The consumable's picture, when the hovered form has none.
  property bool withPicture: false

  // Where a drawn oligarchy card or command is (JokerArt's family); "" for a
  // theme, whose picture is an image.
  readonly property string drawing: !consumable ? ""
    : consumable.kind === "oligarchy" ? "oligarchy" : consumable.kind === "command" ? "commands" : ""
  readonly property bool pictured: consumable !== null && withPicture
    && (consumable.kind === "theme" ? Drawn.themes[consumable.id] === true
      : drawing !== "" && Drawn[drawing][consumable.id] === true)

  readonly property var card: joker || consumable
  readonly property var stickers: joker && joker.stickers ? joker.stickers : []
  readonly property bool oligarchy: consumable !== null && consumable.kind === "oligarchy"
  // What the glossary explains: the terms of its text (and its stickers).
  readonly property var terms: !card ? []
    : Labels.mentions(card.description + stickers.map(function(s) { return " " + s.name }).join(""),
      Hints.terms, card.name)

  function showConsumable(item, c, note, withPicture) {
    joker = null
    consumable = c
    root.note = note
    root.withPicture = withPicture === true
    place(item, null)
  }
  function showJoker(item, j, under, hidden) {
    // Face down (The Kernel Panic): nothing to tell.
    if (hidden) { hide(source); return }
    consumable = null
    joker = j
    note = ""
    withPicture = false
    place(item, under)
  }
  function hide(item) {
    if (source !== item) return
    source = null
    consumable = null
    joker = null
  }
  // Under `under` (the joker row), or beside the item.
  function place(item, under) {
    source = item
    var p = item.mapToItem(root, 0, 0)
    bubble.place = Qt.rect(p.x, p.y, item.width, item.height)
    bubble.below = under ? under.mapToItem(root, 0, under.height).y : -1
  }

  Bubble {
    id: bubble
    anchors.fill: parent
    shown: root.source !== null && root.card !== null
    // Wider with the picture beside the text.
    panelWidth: root.pictured ? Style.space(310) : Style.space(250)

    Row {
      width: parent.width
      spacing: Style.space(10)

      // The picture, framed, when the hovered form has none.
      Rectangle {
        id: picture
        visible: root.pictured
        width: visible ? Style.space(72) : 0
        height: Math.round(width * 40 / 64)
        color: "transparent"
        border.width: Style.normalBorderWidth
        border.color: root.oligarchy ? Colors.urgent : Colors.foreground
        Image {
          anchors { fill: parent; margins: parent.border.width }
          visible: root.drawing === ""
          smooth: false
          source: root.pictured && root.drawing === "" ? "../art/themes/" + root.consumable.id + ".png" : ""
        }
        JokerArt {
          anchors { fill: parent; margins: parent.border.width }
          visible: root.drawing !== ""
          family: root.drawing || "oligarchy"
          jokerId: root.pictured && root.drawing !== "" ? root.consumable.id : ""
        }
      }

      Column {
        width: parent.width - (picture.visible ? picture.width + parent.spacing : 0)
        spacing: Style.space(4)

        // Its name, and what it is (a joker's rarity, a consumable's kind), as
        // a notification's title and source.
        Item {
          width: parent.width
          height: Math.max(name.implicitHeight, kind.implicitHeight)
          Text {
            id: name
            width: parent.width - kind.implicitWidth - Style.space(8)
            wrapMode: Text.WordWrap
            text: root.card ? root.card.name : ""
            color: root.oligarchy ? Colors.urgent : Colors.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.title
            font.bold: true
          }
          Text {
            id: kind
            anchors { right: parent.right; baseline: name.baseline }
            text: root.joker ? I18n.tr(root.joker.rarity_name)
              : !root.consumable ? ""
              : root.consumable.transparent ? I18n.tr("{kind} · transparent", { kind: I18n.tr(root.consumable.kind) })
              : I18n.tr(root.consumable.kind)
            color: root.joker
              ? (root.joker.rarity === "extra" ? Colors.accent : root.joker.rarity === "core" ? Colors.muted : Colors.urgent)
              : root.consumable && root.consumable.transparent ? Colors.accent : Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.caption
          }
        }
        Text {
          width: parent.width
          wrapMode: Text.WordWrap
          textFormat: Text.StyledText
          text: root.card ? Labels.highlight(root.card.description, Hints.terms, root.card.name) : ""
          color: Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.body
        }
        // What it does now: a joker's bonus, what it copies, a consumable's
        // effect if used now.
        Text {
          width: parent.width
          visible: text !== ""
          wrapMode: Text.WordWrap
          text: !root.card ? ""
            : [root.joker && root.joker.bonus ? root.joker.bonus.text : "", root.card.current || ""]
              .filter(function(t) { return t !== "" }).join(" · ")
          color: root.joker && root.joker.bonus && (root.joker.bonus.kind === "mult" || root.joker.bonus.kind === "x_mult")
            ? Colors.urgent : Colors.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.body
          font.bold: true
        }
        // A joker's edition and stickers.
        Text {
          width: parent.width
          visible: text !== ""
          wrapMode: Text.WordWrap
          text: !root.joker ? ""
            : (root.joker.edition ? [root.joker.edition.name + " · " + root.joker.edition.description] : [])
              .concat(root.stickers.map(function(s) {
                return s.name + (s.blinds_left !== null && s.blinds_left !== undefined ? " · " + s.blinds_left : "")
              })).join("\n")
          color: Colors.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
          font.bold: true
        }
        Text {
          width: parent.width
          visible: text !== ""
          wrapMode: Text.WordWrap
          text: root.card && root.card.quip ? root.card.quip : ""
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
          font.italic: true
        }
        Text {
          width: parent.width
          visible: root.consumable !== null && root.note !== ""
          wrapMode: Text.WordWrap
          text: root.note
          color: Colors.warm
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
        }
      }
    }

    // The glossary, under a rule.
    Rectangle {
      visible: root.terms.length > 0
      width: parent.width
      height: Style.normalBorderWidth
      color: Style.normalBorderColor
    }
    TermList {
      visible: root.terms.length > 0
      width: parent.width
      terms: root.terms
    }
  }
}
