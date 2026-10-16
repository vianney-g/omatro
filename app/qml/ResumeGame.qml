import QtQuick
import omatro
import "Labels.js" as Labels

// At start, when a game was left in progress: resume it (Enter), or pick a
// deck for a new one (N), which replaces it once started.
Rectangle {
  id: root

  // The saved game's state.
  property var saved: null
  signal resume()
  signal newGame()

  color: Util.alpha(Color.background, 0.96)

  MouseArea { anchors.fill: parent }

  // Where the game stands: ante and blind (or shop), deck, money, jokers.
  readonly property string summary: {
    var s = saved
    if (!s) return ""
    var blind = s.blinds.find(function(b) { return b.kind === s.blind })
    var where = s.phase === "shop" ? "in the shop"
      : s.phase === "pack" ? "opening a package"
      : blind ? blind.name : ""
    var parts = ["Ante " + s.ante + " / " + s.winning_ante, where]
    if (s.run_deck) parts.push("on " + s.run_deck.name)
    parts.push(Labels.money(s.money))
    parts.push(s.jokers.length + (s.jokers.length === 1 ? " joker" : " jokers"))
    return parts.join(" · ")
  }

  Column {
    anchors.centerIn: parent
    spacing: Style.spacing.huge

    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: "Resume your game?"
      color: Color.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.displayLarge
      font.bold: true
    }
    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: root.summary
      color: Color.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.title
    }
    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: "A new game replaces it once started."
      color: Color.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }
    Row {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.xxl
      Button {
        text: "Continue (Enter)"
        bordered: true
        onClicked: root.resume()
      }
      Button {
        text: "New game (N)"
        onClicked: root.newGame()
      }
    }
  }
}
