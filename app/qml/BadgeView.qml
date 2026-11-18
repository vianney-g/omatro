import QtQuick
import omatro

// A badge, as on a README: its icon (BadgeIcon) and label on a dark part, its
// value on the accent (the urgent red for the oligarchy's), what it would give
// now under it. Hovered, what it does shows beside it (CardPeek, through Peek).
Item {
  id: root

  property var tag: null
  // Hovered from outside: the button it belongs to (Skip), which takes the
  // clicks.
  property bool hovered: false
  property bool hoverable: true
  readonly property bool showTip: (mouse.containsMouse || hovered) && tag !== null
  onShowTipChanged: showTip
    ? Peek.showConsumable(pill, {
        kind: "badge", id: tag.id, name: tag.name, description: tag.description,
        quip: tag.quip, current: tag.current, transparent: false }, "")
    : Peek.hide(pill)
  Component.onDestruction: Peek.hide(pill)

  readonly property bool hasValue: tag !== null && tag.value !== ""
  readonly property color valueColor: tag !== null && tag.id === "funding_oligarchy" ? Colors.urgent : Colors.accent
  readonly property real pad: Style.space(6)
  readonly property real r: Style.cornerRadius

  implicitWidth: Math.max(pill.width, currentText.visible ? currentText.implicitWidth : 0)
  implicitHeight: pill.height + (currentText.visible ? currentText.implicitHeight : 0)

  // What it would give now ("+$15 now"), under the pill.
  Text {
    id: currentText
    anchors { top: pill.bottom; horizontalCenter: pill.horizontalCenter }
    visible: text !== ""
    text: root.tag && root.tag.current ? root.tag.current : ""
    color: Colors.accent
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
    font.bold: true
  }

  // The pill: "★ stars | 100".
  Item {
    id: pill
    anchors.horizontalCenter: parent.horizontalCenter
    width: left.width + (root.hasValue ? valueBox.width : 0)
    height: labelText.implicitHeight + 2 * Style.space(3)

    Rectangle {
      id: leftFill
      width: left.width + (root.hasValue ? root.r : 0)
      height: parent.height
      radius: root.r
      color: Qt.tint(Colors.background, Util.alpha(Colors.foreground, 0.18))
    }
    Item {
      id: left
      width: leftRow.implicitWidth + 2 * root.pad
      height: parent.height
      Row {
        id: leftRow
        anchors.centerIn: parent
        spacing: Style.space(4)
        BadgeIcon {
          anchors.verticalCenter: parent.verticalCenter
          width: labelText.font.pixelSize
          height: width
          tag: root.tag ? root.tag.id : ""
          color: Colors.foreground
        }
        Text {
          id: labelText
          text: root.tag ? root.tag.label : ""
          color: Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
          font.bold: !root.hasValue
        }
      }
    }
    Rectangle {
      id: valueBox
      visible: root.hasValue
      x: left.width
      width: valueText.implicitWidth + 2 * root.pad
      height: parent.height
      radius: root.r
      color: root.valueColor
      // Square where it meets the label.
      Rectangle {
        width: root.r
        height: parent.height
        color: parent.color
      }
      Text {
        id: valueText
        anchors.centerIn: parent
        text: root.tag ? root.tag.value : ""
        color: Colors.background
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.bold: true
      }
    }
  }

  MouseArea {
    id: mouse
    anchors.fill: pill
    enabled: root.hoverable
    hoverEnabled: true
  }
}
