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
  readonly property real cardHeight: Math.round(Style.space(84) * 1.4) * cardScale
  // A line of cards holds a whole suit of a standard deck.
  readonly property real lineWidth: 13 * Style.space(84) * cardScale + 12 * Style.space(4) + 1
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

      // The suits, scrolling when they are taller than the window.
      Flickable {
        id: flick
        // Room on the right for the scroll bar, clear of the cards.
        width: suits.width + Style.space(10)
        height: Math.min(suits.height, Math.max(Style.space(120),
          root.height - 2 * Style.spacing.huge - 2 * Style.spacing.lg
            - title.height - details.height - 2 * content.spacing - Style.space(40)))
        contentWidth: suits.width
        contentHeight: suits.height
        interactive: contentHeight > height
        boundsBehavior: Flickable.StopAtBounds
        clip: true

        // Where it scrolls, a thin bar on its right edge.
        Rectangle {
          parent: flick
          visible: flick.interactive
          anchors.right: parent.right
          y: flick.visibleArea.yPosition * flick.height
          width: Style.space(3)
          height: flick.visibleArea.heightRatio * flick.height
          radius: width / 2
          color: Util.alpha(Colors.muted, 0.6)
          z: 1
        }

        Column {
          id: suits
          spacing: Style.spacing.lg
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
              // Level with the first line of cards, which a long suit has more of.
              Column {
                width: Style.space(56)
                y: (Math.min(parent.height, root.cardHeight) - height) / 2
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
              // Thirteen cards a line: a suit with more goes on to the next.
              Flow {
                width: root.lineWidth
                spacing: Style.space(4)
                Repeater {
                  model: parent.parent.cards
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
                      lively: false
                      dimmed: !root.inPile[modelData.id]
                      onHoveredChanged: if (hovered) root.hovered = view
                        else if (root.hovered === view) root.hovered = null
                    }
                  }
                }
              }
            }
          }
        }
      }

      // The hovered card's details.
      Label {
        id: details
        width: Style.space(60) + root.lineWidth
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
