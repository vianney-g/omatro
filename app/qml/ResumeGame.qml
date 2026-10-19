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

  color: Util.alpha(Colors.background, 0.96)

  MouseArea { anchors.fill: parent }

  // Where the game stands: ante and blind (or shop), deck, money, jokers.
  readonly property string summary: {
    var s = saved
    if (!s) return ""
    var blind = s.blinds.find(function(b) { return b.kind === s.blind })
    var where = s.phase === "shop" ? I18n.tr("in the shop")
      : s.phase === "pack" ? I18n.tr("opening a package")
      : blind ? blind.name : ""
    var parts = [I18n.tr("Ante {a} / {b}", { a: s.ante, b: s.winning_ante }), where]
    if (s.run_deck) parts.push(I18n.tr("on {deck}", { deck: s.run_deck.name }))
    if (s.stake) parts.push(s.stake.name)
    parts.push(Labels.money(s.money))
    parts.push(I18n.tr(s.jokers.length === 1 ? "{n} joker" : "{n} jokers", { n: s.jokers.length }))
    return parts.join(" · ")
  }

  Column {
    anchors.centerIn: parent
    spacing: Style.spacing.huge

    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: I18n.tr("Resume your game?")
      color: Colors.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.displayLarge
      font.bold: true
    }
    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: root.summary
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.title
    }
    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: I18n.tr("A new game replaces it once started.")
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }
    Row {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.xxl
      Button {
        text: I18n.tr("Continue (Enter)")
        bordered: true
        onClicked: root.resume()
      }
      Button {
        text: I18n.tr("New game (N)")
        onClicked: root.newGame()
      }
    }
  }
}
