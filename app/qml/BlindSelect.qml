import QtQuick
import omatro
import "Labels.js" as Labels

// The three blinds of the ante: play the next one, or skip it if it is not
// the boss.
Row {
  id: root

  property var blinds: []
  // Change Request: the price of rerolling the boss (-1 when not allowed).
  property int bossRerollCost: -1
  signal play()
  signal skip()
  signal rerollBoss()

  spacing: Style.spacing.huge

  Repeater {
    model: root.blinds

    Rectangle {
      id: panel
      readonly property bool next: modelData.status === "select"
      readonly property bool done: modelData.status === "defeated" || modelData.status === "skipped"

      width: Style.space(210)
      height: col.implicitHeight + 2 * Style.spacing.huge
      radius: Style.cornerRadius
      color: next ? Qt.tint(Color.background, Util.alpha(Color.accent, 0.10)) : Style.normalFill
      border.width: next ? Style.space(2) : Style.normalBorderWidth
      border.color: next ? Color.accent : Style.normalBorderColor
      opacity: done ? 0.5 : 1

      Column {
        id: col
        anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.spacing.huge }
        spacing: Style.spacing.lg

        Text {
          width: parent.width
          horizontalAlignment: Text.AlignHCenter
          wrapMode: Text.WordWrap
          text: modelData.name
          color: panel.next ? Color.accent : Color.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.heading
          font.bold: true
        }
        // The boss's effect.
        Text {
          width: parent.width
          visible: modelData.description !== ""
          horizontalAlignment: Text.AlignHCenter
          wrapMode: Text.WordWrap
          text: modelData.description
          color: Color.urgent
          font.family: Style.font.family
          font.pixelSize: Style.font.body
          font.bold: true
        }
        Text {
          width: parent.width
          height: Style.space(36)
          horizontalAlignment: Text.AlignHCenter
          wrapMode: Text.WordWrap
          text: modelData.quip
          color: Color.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
          font.italic: true
        }
        Column {
          width: parent.width
          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Target"
            color: Color.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.caption
          }
          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: Labels.number(modelData.target)
            color: Color.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.display
            font.bold: true
          }
        }
        Text {
          anchors.horizontalCenter: parent.horizontalCenter
          text: "Reward: " + Labels.money(modelData.reward)
          color: Color.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.body
        }

        // Status, or the choice for the next blind.
        Item {
          width: parent.width
          height: Style.space(84)

          Text {
            anchors.centerIn: parent
            visible: !panel.next
            text: Labels.blindStatus(modelData.status)
            color: Color.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.subtitle
          }
          Column {
            anchors.centerIn: parent
            visible: panel.next
            spacing: Style.spacing.md
            Button {
              anchors.horizontalCenter: parent.horizontalCenter
              text: "Play (Enter)"
              onClicked: root.play()
            }
            Button {
              anchors.horizontalCenter: parent.horizontalCenter
              visible: modelData.kind !== "boss"
              text: "Skip (S)"
              onClicked: root.skip()
            }
            Button {
              anchors.horizontalCenter: parent.horizontalCenter
              visible: modelData.kind === "boss" && root.bossRerollCost >= 0
              text: "Reroll (B) · $" + root.bossRerollCost
              onClicked: root.rerollBoss()
            }
          }
        }
      }
    }
  }
}
