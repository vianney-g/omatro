import QtQuick
import omatro

// An open pack: pick its cards (Install), or leave (Ctrl+C). For a coreutils
// pack, commands that change cards apply to the cards selected in the hand
// drawn below.
Column {
  id: root

  property var opened: null
  property var modifiers: null
  // Whether a choice can be taken now (selection, joker slots).
  property var canPick: function(choice) { return true }
  signal pick(int index)
  signal skip()

  spacing: Style.spacing.xxl

  Text {
    anchors.horizontalCenter: parent.horizontalCenter
    text: root.opened ? "pacman -S " + root.opened.name : ""
    color: Color.urgent
    font.family: Style.font.family
    font.pixelSize: Style.font.displayLarge
    font.bold: true
  }

  Text {
    anchors.horizontalCenter: parent.horizontalCenter
    text: root.opened
      ? root.opened.description + " · " + root.opened.picks_left + " left"
        + (root.opened.with_hand ? " · select cards below for the commands that need them" : "")
      : ""
    color: Color.muted
    font.family: Style.font.family
    font.pixelSize: Style.font.body
  }

  Row {
    anchors.horizontalCenter: parent.horizontalCenter
    spacing: Style.spacing.huge
    Repeater {
      model: root.opened ? root.opened.choices : []
      Column {
        spacing: Style.spacing.md
        JokerView {
          visible: modelData.kind === "joker"
          joker: visible ? modelData : null
        }
        ConsumableView {
          visible: modelData.kind === "theme" || modelData.kind === "command"
          item: visible ? modelData : null
        }
        CardView {
          anchors.horizontalCenter: parent.horizontalCenter
          visible: modelData.kind === "card"
          card: visible ? modelData.card : null
          modifiers: root.modifiers
          interactive: false
        }
        Button {
          anchors.horizontalCenter: parent.horizontalCenter
          text: "Install"
          enabled: root.canPick(modelData)
          opacity: enabled ? 1 : 0.5
          onClicked: root.pick(index)
        }
      }
    }
  }

  Button {
    anchors.horizontalCenter: parent.horizontalCenter
    text: "Ctrl+C (skip)"
    onClicked: root.skip()
  }
}
