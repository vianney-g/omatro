import QtQuick
import omatro
import "Labels.js" as Labels

// A badge, like a README badge ("label: value"); hovering it tells what it
// does.
Item {
  id: root

  property var tag: null

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

  // One pill: "resolution: 4k".
  Rectangle {
    id: pill
    width: labelText.implicitWidth + 2 * Style.space(6)
    height: labelText.implicitHeight + 2 * Style.space(3)
    color: Util.alpha(Colors.accent, 0.3)
    border.width: Style.normalBorderWidth
    border.color: Util.alpha(Colors.accent, 0.6)
    radius: Style.cornerRadius
    Text {
      id: labelText
      anchors.centerIn: parent
      textFormat: Text.StyledText
      text: !root.tag ? "" : root.tag.value === "" ? "<b>" + root.tag.label + "</b>"
        : root.tag.label + ": <b>" + root.tag.value + "</b>"
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
  }

  MouseArea {
    id: mouse
    anchors.fill: pill
    hoverEnabled: true
    onContainsMouseChanged: containsMouse && root.tag
      ? Hints.show(root, root.tag.description, root.tag.name) : Hints.hide(root)
  }

  Rectangle {
    z: 10
    visible: mouse.containsMouse && root.tag !== null
    anchors { horizontalCenter: pill.horizontalCenter; bottom: pill.top; bottomMargin: Style.space(6) }
    width: Math.min(Style.space(240), tip.implicitWidth + 2 * Style.space(8))
    height: tip.implicitHeight + 2 * Style.space(6)
    radius: Style.cornerRadius
    color: Colors.background
    border.width: Style.normalBorderWidth
    border.color: Colors.accent
    Text {
      id: tip
      anchors { fill: parent; margins: Style.space(6) }
      wrapMode: Text.WordWrap
      textFormat: Text.StyledText
      text: root.tag ? Labels.highlight(root.tag.description, Hints.terms, root.tag.name) + "<br>" + root.tag.quip : ""
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
  }
}
