import QtQuick
import omatro

// A joker: rarity, name, effect, what it adds now, its quip and its edition.
Item {
  id: root

  property var joker: null
  // What a growing joker gives right now: {kind, text}.
  property var bonus: joker ? joker.bonus : null

  width: Style.space(120)
  height: Style.space(230)

  readonly property var edition: joker ? joker.edition : null
  readonly property color rarityColor: !joker ? Color.muted
    : joker.rarity === "extra" ? Color.accent
    : joker.rarity === "core" ? Color.muted : Color.urgent

  function flash(text, color) {
    popup.text = text
    popup.color = color
    flashAnim.restart()
  }

  Rectangle {
    id: face
    anchors.fill: parent
    radius: Style.cornerRadius
    color: Qt.tint(Color.background, Util.alpha(Color.accent, root.edition ? 0.20 : 0.10))
    border.width: root.edition ? Style.space(2) : Style.normalBorderWidth
    border.color: root.edition ? Color.accent : Util.alpha(Color.accent, Style.normalBorderAlpha)

    Column {
      anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
      spacing: Style.space(4)

      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        text: root.joker ? root.joker.rarity_name : ""
        color: root.rarityColor
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
      }
      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.joker ? root.joker.name : ""
        color: Color.accent
        font.family: Style.font.family
        font.pixelSize: Style.font.subtitle
        font.bold: true
      }
      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.joker ? root.joker.description : ""
        color: Color.foreground
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
      }
      // Fork: what it copies.
      Text {
        width: parent.width
        visible: text !== ""
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.joker && root.joker.current ? root.joker.current : ""
        color: Color.accent
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.bold: true
      }
      // Badge: the bonus a growing joker gives right now; pulses when it changes.
      Rectangle {
        id: badge
        anchors.horizontalCenter: parent.horizontalCenter
        visible: root.bonus !== null && root.bonus !== undefined
        readonly property color tint: !visible ? Color.accent
          : root.bonus.kind === "mult" || root.bonus.kind === "x_mult" ? Color.urgent : Color.accent
        width: badgeText.implicitWidth + 2 * Style.space(8)
        height: badgeText.implicitHeight + 2 * Style.space(3)
        radius: height / 2
        color: Util.alpha(tint, 0.18)
        border.width: Style.normalBorderWidth
        border.color: tint
        Text {
          id: badgeText
          anchors.centerIn: parent
          text: badge.visible ? root.bonus.text : ""
          color: badge.tint
          font.family: Style.font.family
          font.pixelSize: Style.font.body
          font.bold: true
          onTextChanged: if (badge.visible) pulse.restart()
        }
        SequentialAnimation {
          id: pulse
          NumberAnimation { target: badge; property: "scale"; to: 1.25; duration: 120 }
          NumberAnimation { target: badge; property: "scale"; to: 1; duration: 220; easing.type: Easing.OutBack }
        }
      }
      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.joker ? root.joker.quip : ""
        color: Color.muted
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.italic: true
      }
    }

    // Edition, at the bottom.
    Text {
      anchors { left: parent.left; right: parent.right; bottom: parent.bottom; margins: Style.space(6) }
      visible: root.edition !== null
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.edition ? root.edition.name + " · " + root.edition.description : ""
      color: Color.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.bold: true
    }
  }

  Text {
    id: popup
    anchors { horizontalCenter: parent.horizontalCenter; top: parent.bottom; topMargin: Style.space(6) }
    opacity: 0
    font.family: Style.font.family
    font.pixelSize: Style.font.heading
    font.bold: true
  }

  SequentialAnimation {
    id: flashAnim
    ParallelAnimation {
      NumberAnimation { target: face; property: "scale"; from: 1; to: 1.1; duration: 90 }
      NumberAnimation { target: face; property: "rotation"; from: 0; to: -4; duration: 90 }
      NumberAnimation { target: popup; property: "opacity"; to: 1; duration: 90 }
    }
    ParallelAnimation {
      NumberAnimation { target: face; property: "scale"; to: 1; duration: 180; easing.type: Easing.OutBack }
      NumberAnimation { target: face; property: "rotation"; to: 0; duration: 180; easing.type: Easing.OutBack }
    }
    PauseAnimation { duration: 220 }
    NumberAnimation { target: popup; property: "opacity"; to: 0; duration: 200 }
  }
}
