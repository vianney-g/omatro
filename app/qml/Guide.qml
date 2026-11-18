import QtQuick
import omatro

// The guided first game: one bubble at a time, pointing at the part of the
// interface it explains (Bubble), as the game gets there. Main says which
// step is due (`step`) and what it points at (`target`); Enter or a click
// says it was read (`done`), Escape skips the whole guide (`skip`). Fill the
// window with it, over everything.
Item {
  id: root

  property string step: ""
  property Item target: null
  signal done()
  signal skip()

  // Each step: its title and what it says, in English (I18n translates).
  readonly property var texts: ({
    blinds: ["Three blinds per ante.", "Beat each one's target to move on. Play this one (Enter), or skip it for its badge."],
    cards: ["Pick up to 5 cards", "(1 to 9, or click) to make a poker hand."],
    score: ["Chips × mult = your score.", "Better hands give more of both, and each card played adds its chips."],
    play: ["Play it (Enter) to score,", "or discard (D) to draw new cards. Both are limited per blind."],
    target: ["Reach the target before your hands run out.", "Fall short, and it's a kernel panic."],
    shop: ["The shop.", "Jokers boost every hand you play: that's where big scores come from."],
    upgrades: ["Packages and pull requests:", "a pick of cards on the spot, or a lasting upgrade. Money left over earns interest."],
    boss: ["Each ante ends with a boss", "that bends a rule. Read it before you play."]
  })
  readonly property var text: texts[step] || ["", ""]

  // Where the target is, taken when the step comes (or the target moves in).
  function locate() {
    if (!target) return
    var p = target.mapToItem(root, 0, 0)
    bubble.place = Qt.rect(p.x, p.y, target.width, target.height)
  }
  onStepChanged: locate()
  onTargetChanged: locate()
  onWidthChanged: locate()
  onHeightChanged: locate()
  // Layouts settle after the phase changes: look again a moment later.
  Timer { interval: 250; running: root.step !== ""; repeat: true; onTriggered: root.locate() }

  Bubble {
    id: bubble
    anchors.fill: parent
    shown: root.step !== "" && root.target !== null
    clickable: true
    panelWidth: Style.space(260)
    onClicked: root.done()

    Text {
      width: parent.width
      wrapMode: Text.WordWrap
      text: I18n.tr(root.text[0])
      color: Colors.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.title
      font.bold: true
    }
    Text {
      width: parent.width
      wrapMode: Text.WordWrap
      text: I18n.tr(root.text[1])
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }
    Text {
      width: parent.width
      wrapMode: Text.WordWrap
      text: I18n.tr("Enter or click: got it · Esc: skip the guide")
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.italic: true
    }
  }
}
