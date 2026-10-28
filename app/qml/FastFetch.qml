import QtQuick
import omatro
import "Labels.js" as Labels

// Run info over the game: hand levels and plays, and the cards left to draw.
Rectangle {
  id: root

  property var game: null
  signal close()
  signal openDeck()

  function copySeed() { seedLabel.copy() }

  color: Util.alpha(Colors.background, 0.92)

  // Hand types strongest first, as in a poker ranking.
  readonly property var hands: game ? game.hands.slice().reverse() : []

  MouseArea { anchors.fill: parent; onClicked: root.close() }

  component Label: Text {
    color: Colors.foreground
    font.family: Style.font.family
    font.pixelSize: Style.font.body
  }

  component Header: Text {
    color: Colors.muted
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
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
      spacing: Style.spacing.xxl

      Row {
        spacing: Style.spacing.xxl
        Text {
          text: "fastfetch"
          color: Colors.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.displayLarge
          font.bold: true
        }
        Header {
          anchors.baseline: parent.children[0].baseline
          text: I18n.tr("Your run, beautifully over-reported. · I or Esc: close")
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
            Header { width: table.widths[0]; text: I18n.tr("Hand") }
            Header { width: table.widths[1]; text: I18n.tr("Level") }
            Header { width: table.widths[2]; text: I18n.tr("Chips × Mult") }
            Header { width: table.widths[3]; text: I18n.tr("Played"); horizontalAlignment: Text.AlignRight }
          }

          Repeater {
            model: root.hands
            Row {
              Column {
                width: table.widths[0]
                Label { text: modelData.name; font.bold: modelData.level > 1 }
                Header { visible: modelData.quip !== ""; text: modelData.quip; font.italic: true }
              }
              Label { width: table.widths[1]; text: I18n.tr("lvl.{n}", { n: modelData.level }) }
              Row {
                width: table.widths[2]
                Label { text: Labels.number(modelData.chips); color: Colors.accent }
                Label { text: " × "; color: Colors.muted }
                Label { text: Labels.mult(modelData.mult); color: Colors.urgent }
              }
              Label {
                width: table.widths[3]
                horizontalAlignment: Text.AlignRight
                text: modelData.played
                color: modelData.played > 0 ? Colors.foreground : Colors.muted
              }
            }
          }
        }

        // Cards left to draw, by suit.
        Column {
          spacing: Style.spacing.md

          // The machine and the difficulty level, with every rule in force.
          Text {
            text: "uname -a"
            color: Colors.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.heading
            font.bold: true
          }
          Label {
            width: Style.space(280)
            visible: root.game !== null && root.game.run_deck !== null
            wrapMode: Text.WordWrap
            textFormat: Text.StyledText
            text: visible ? "<b>" + root.game.run_deck.name + "</b>: " + root.game.run_deck.description : ""
          }
          Label {
            width: Style.space(280)
            visible: root.game !== null && root.game.stake !== null
            wrapMode: Text.WordWrap
            textFormat: Text.StyledText
            readonly property var stake: visible ? root.game.stake : null
            text: !stake ? "" : "<b>" + stake.name + "</b>: " + (stake.rules.length
              ? stake.rules.map(function(r) { return "<br>· " + I18n.tr(r) }).join("")
              : I18n.tr(stake.description))
          }
          SeedLabel {
            id: seedLabel
            code: root.game ? root.game.seed_code : ""
            shortcut: "Ctrl+C"
          }
          Header {
            width: Style.space(280)
            visible: root.game !== null && root.game.seeded
            wrapMode: Text.WordWrap
            text: I18n.tr("A chosen seed: this game counts for no stat nor achievement.")
            font.italic: true
          }

          Text {
            topPadding: Style.spacing.lg
            text: "ls ~/deck"
            color: Colors.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.heading
            font.bold: true
          }
          Header {
            text: I18n.tr("{n} files, no hidden dotfiles. Yet.", { n: root.game ? root.game.deck_size : 52 })
            font.italic: true
          }
          Row {
            spacing: Style.spacing.lg
            Label {
              anchors.verticalCenter: parent.verticalCenter
              text: root.game ? I18n.tr("{a} / {b} cards left", { a: root.game.remaining.length, b: root.game.deck_size }) : ""
            }
            Button {
              compact: true
              text: I18n.tr("Deck (P)")
              onClicked: root.openDeck()
            }
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
            text: I18n.tr(blobs > 1 ? "{n} Binary Blobs" : "{n} Binary Blob", { n: blobs })
            color: Colors.muted
          }
          Repeater {
            model: ["spades", "hearts", "clubs", "diamonds"]
            Row {
              spacing: Style.spacing.lg
              readonly property string suit: modelData
              Label {
                width: Style.space(18)
                text: Labels.suit(parent.suit)
                color: Labels.isRed(parent.suit) ? Colors.urgent : Colors.foreground
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
                    ranks.push(modified ? "<b><font color='" + Colors.accent + "'>" + label + "</font></b>" : label)
                  }
                  return ranks.length ? ranks.join(" ") : "–"
                }
              }
            }
          }

          Text {
            topPadding: Style.spacing.lg
            text: "git log --merges"
            color: Colors.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.heading
            font.bold: true
          }
          Label {
            width: Style.space(280)
            wrapMode: Text.WordWrap
            text: root.game && root.game.vouchers.length
              ? root.game.vouchers.map(function(v) { return v.name + ": " + v.description }).join("\n")
              : I18n.tr("No pull request merged yet.")
            color: root.game && root.game.vouchers.length ? Colors.foreground : Colors.muted
          }
        }
      }
    }
  }
}
