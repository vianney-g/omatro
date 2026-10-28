import QtQuick
import omatro
import "Labels.js" as Labels

// The whole deck over the game, a row per suit: the cards out of the draw
// pile greyed, Binary Blobs on a row of their own (they have no suit), the
// hovered card's details underneath.
Rectangle {
  id: root

  property var game: null
  signal close()

  color: Util.alpha(Colors.background, 0.92)

  readonly property real cardScale: 0.72
  // Ids of the cards still to draw.
  readonly property var inPile: {
    var ids = {}
    if (game) game.remaining.forEach(function(c) { ids[c.id] = true })
    return ids
  }
  property var hovered: null
  onVisibleChanged: hovered = null

  MouseArea { anchors.fill: parent; onClicked: root.close() }

  component Label: Text {
    color: Colors.foreground
    font.family: Style.font.family
    font.pixelSize: Style.font.body
  }

  Rectangle {
    anchors.centerIn: parent
    width: content.implicitWidth + 2 * Style.spacing.huge
    height: content.implicitHeight + 2 * Style.spacing.huge
    radius: Style.cornerRadius
    color: Colors.background
    border.width: Style.normalBorderWidth
    border.color: Colors.accent

    // Clicks inside the panel do not close it.
    MouseArea { anchors.fill: parent }

    Column {
      id: content
      anchors { left: parent.left; top: parent.top; margins: Style.spacing.huge }
      spacing: Style.spacing.lg

      Row {
        spacing: Style.spacing.xxl
        Text {
          id: title
          text: "ls -la ~/deck"
          color: Colors.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.displayLarge
          font.bold: true
        }
        Text {
          anchors.baseline: title.baseline
          text: root.game ? I18n.tr("{a} / {b} cards left", { a: root.game.remaining.length, b: root.game.deck_size }) : ""
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.body
        }
        Text {
          anchors.baseline: title.baseline
          text: I18n.tr("P or Esc: close")
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
          font.italic: true
        }
      }

      Repeater {
        model: ["spades", "hearts", "clubs", "diamonds", "blobs"]
        Row {
          readonly property string suit: modelData
          readonly property var cards: !root.game ? [] : root.game.deck.filter(function(c) {
            var blob = c.enhancement === "binary_blob"
            return suit === "blobs" ? blob : !blob && c.suit === suit
          })
          readonly property int inPileCount: cards.filter(function(c) { return root.inPile[c.id] }).length
          visible: cards.length > 0
          spacing: Style.space(4)
          // The suit, and how many of its cards are left to draw.
          Column {
            width: Style.space(56)
            anchors.verticalCenter: parent.verticalCenter
            Label {
              text: parent.parent.suit === "blobs" ? "BLOB" : Labels.suit(parent.parent.suit)
              color: parent.parent.suit === "blobs" ? Colors.muted
                : Labels.isRed(parent.parent.suit) ? Colors.urgent : Colors.foreground
              font.pixelSize: parent.parent.suit === "blobs" ? Style.font.body : Style.font.heading
              font.bold: true
            }
            Label {
              text: parent.parent.inPileCount + " / " + parent.parent.cards.length
              color: Colors.muted
              font.pixelSize: Style.font.caption
            }
          }
          Repeater {
            model: parent.cards
            Item {
              width: view.width * root.cardScale
              height: view.height * root.cardScale
              CardView {
                id: view
                scale: root.cardScale
                transformOrigin: Item.TopLeft
                modifiers: root.game ? root.game.modifiers : null
                card: modelData
                interactive: false
                showTip: false
                dimmed: !root.inPile[modelData.id]
                onHoveredChanged: if (hovered) root.hovered = view
                  else if (root.hovered === view) root.hovered = null
              }
            }
          }
        }
      }

      // The hovered card's details.
      Label {
        width: Style.space(60) + 13 * (Style.space(84) * root.cardScale + Style.space(4))
        height: 4 * Style.font.body * 1.4
        wrapMode: Text.WordWrap
        readonly property var view: root.hovered
        text: !view ? I18n.tr("Hover a card for its details.")
          : (view.blob ? "Binary Blob" : view.rankText + view.suitText)
            + (root.inPile[view.card.id] ? "" : " · " + I18n.tr("out of the draw pile"))
            + (view.details ? "\n" + view.details : "")
        color: view ? Colors.foreground : Colors.muted
      }
    }
  }
}
