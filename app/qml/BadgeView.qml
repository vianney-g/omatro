import QtQuick
import omatro

// A badge, like a README badge: grey label, coloured value; hovering it tells
// what it does.
Item {
  id: root

  property var tag: null

  implicitWidth: pill.width
  implicitHeight: pill.height

  Row {
    id: pill
    Rectangle {
      width: labelText.implicitWidth + 2 * Style.space(6)
      height: labelText.implicitHeight + 2 * Style.space(3)
      color: Util.alpha(Color.foreground, 0.25)
      radius: Style.cornerRadius
      Text {
        id: labelText
        anchors.centerIn: parent
        text: root.tag ? root.tag.label : ""
        color: Color.foreground
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
      }
    }
    Rectangle {
      visible: root.tag !== null && root.tag.value !== ""
      width: valueText.implicitWidth + 2 * Style.space(6)
      height: valueText.implicitHeight + 2 * Style.space(3)
      color: Util.alpha(Color.accent, 0.45)
      radius: Style.cornerRadius
      Text {
        id: valueText
        anchors.centerIn: parent
        text: root.tag ? root.tag.value : ""
        color: Color.foreground
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.bold: true
      }
    }
  }

  MouseArea {
    id: mouse
    anchors.fill: pill
    hoverEnabled: true
  }

  Rectangle {
    z: 10
    visible: mouse.containsMouse && root.tag !== null
    anchors { horizontalCenter: pill.horizontalCenter; bottom: pill.top; bottomMargin: Style.space(6) }
    width: Math.min(Style.space(240), tip.implicitWidth + 2 * Style.space(8))
    height: tip.implicitHeight + 2 * Style.space(6)
    radius: Style.cornerRadius
    color: Color.background
    border.width: Style.normalBorderWidth
    border.color: Color.accent
    Text {
      id: tip
      anchors { fill: parent; margins: Style.space(6) }
      wrapMode: Text.WordWrap
      text: root.tag ? root.tag.description + "\n" + root.tag.quip : ""
      color: Color.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
  }
}
