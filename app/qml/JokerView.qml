import QtQuick
import omatro

// A joker of the row: name and effect.
Item {
  id: root

  property var joker: null

  width: Style.space(120)
  height: Style.space(150)

  function flash(text, color) {
    popup.text = text
    popup.color = color
    flashAnim.restart()
  }

  Rectangle {
    id: face
    anchors.fill: parent
    radius: Style.cornerRadius
    color: Qt.tint(Color.background, Util.alpha(Color.accent, 0.10))
    border.width: Style.normalBorderWidth
    border.color: Util.alpha(Color.accent, Style.normalBorderAlpha)

    Column {
      anchors { fill: parent; margins: Style.space(10) }
      spacing: Style.space(8)
      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
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
