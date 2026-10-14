import QtQuick
import omatro
import "Labels.js" as Labels

// Run info over the game: hand levels and plays, and the cards left to draw.
Rectangle {
  id: root

  property var game: null
  signal close()

  color: Util.alpha(Color.background, 0.92)

  // Hand types strongest first, as in a poker ranking.
  readonly property var hands: game ? game.hands.slice().reverse() : []

  MouseArea { anchors.fill: parent; onClicked: root.close() }

  component Label: Text {
    color: Color.foreground
    font.family: Style.font.family
    font.pixelSize: Style.font.body
  }

  component Header: Text {
    color: Color.muted
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
  }

  Rectangle {
    anchors.centerIn: parent
    width: content.implicitWidth + 2 * Style.spacing.huge
    height: content.implicitHeight + 2 * Style.spacing.huge
    radius: Style.cornerRadius
    color: Color.background
    border.width: Style.normalBorderWidth
    border.color: Color.accent

    // Clicks inside the panel do not close it.
    MouseArea { anchors.fill: parent }

    Column {
      id: content
      anchors { left: parent.left; top: parent.top; margins: Style.spacing.huge }
      spacing: Style.spacing.xxl

      Row {
        spacing: Style.spacing.xxl
        Text {
          text: "fastfetch"
          color: Color.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.displayLarge
          font.bold: true
        }
        Header {
          anchors.baseline: parent.children[0].baseline
          text: "Your run, beautifully over-reported. · I or Esc: close"
          font.italic: true
        }
      }

      Row {
        spacing: Style.space(40)

        // Hand levels: fixed-width columns.
        Column {
          id: table
          spacing: Style.spacing.md
          readonly property var widths: [Style.space(190), Style.space(56), Style.space(110), Style.space(56)]

          Row {
            Header { width: table.widths[0]; text: "Hand" }
            Header { width: table.widths[1]; text: "Level" }
            Header { width: table.widths[2]; text: "Chips × Mult" }
            Header { width: table.widths[3]; text: "Played"; horizontalAlignment: Text.AlignRight }
          }

          Repeater {
            model: root.hands
            Row {
              Column {
                width: table.widths[0]
                Label { text: modelData.name; font.bold: modelData.level > 1 }
                Header { visible: modelData.quip !== ""; text: modelData.quip; font.italic: true }
              }
              Label { width: table.widths[1]; text: "lvl." + modelData.level }
              Row {
                width: table.widths[2]
                Label { text: Labels.number(modelData.chips); color: Color.accent }
                Label { text: " × "; color: Color.muted }
                Label { text: Labels.mult(modelData.mult); color: Color.urgent }
              }
              Label {
                width: table.widths[3]
                horizontalAlignment: Text.AlignRight
                text: modelData.played
                color: modelData.played > 0 ? Color.foreground : Color.muted
              }
            }
          }
        }

        // Cards left to draw, by suit.
        Column {
          spacing: Style.spacing.md
          Text {
            text: "ls ~/deck"
            color: Color.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.heading
            font.bold: true
          }
          Header {
            text: (root.game ? root.game.deck_size : 52) + " files, no hidden dotfiles. Yet."
            font.italic: true
          }
          Label {
            text: root.game ? root.game.remaining.length + " / " + root.game.deck_size + " cards left" : ""
          }
          Label {
            readonly property int blobs: {
              if (!root.game) return 0
              var n = 0
              for (var i = 0; i < root.game.remaining.length; i++)
                if (root.game.remaining[i].enhancement === "binary_blob") n++
              return n
            }
            visible: blobs > 0
            text: blobs + " Binary Blob" + (blobs > 1 ? "s" : "")
            color: Color.muted
          }
          Repeater {
            model: ["spades", "hearts", "clubs", "diamonds"]
            Row {
              spacing: Style.spacing.lg
              readonly property string suit: modelData
              Label {
                width: Style.space(18)
                text: Labels.suit(parent.suit)
                color: Labels.isRed(parent.suit) ? Color.urgent : Color.foreground
              }
              // Modified cards stand out, in the accent colour.
              Label {
                width: Style.space(260)
                wrapMode: Text.WordWrap
                textFormat: Text.StyledText
                text: {
                  if (!root.game) return ""
                  var ranks = []
                  for (var i = 0; i < root.game.remaining.length; i++) {
                    var card = root.game.remaining[i]
                    if (card.suit !== parent.suit || card.enhancement === "binary_blob") continue
                    var label = Labels.rank(card.rank)
                    var modified = card.enhancement || card.seal || card.edition || card.bonus_chips
                    ranks.push(modified ? "<b><font color='" + Color.accent + "'>" + label + "</font></b>" : label)
                  }
                  return ranks.length ? ranks.join(" ") : "–"
                }
              }
            }
          }
        }
      }
    }
  }
}
