import QtQuick
import omatro
import "Labels.js" as Labels
import "Drawn.js" as Drawn

// A pull request (voucher) for sale: its pair's drawing (art/vouchers, named
// after the first of the pair, in the theme's colours), name and what it
// changes for good. The second of a pair (`follows`) is the deluxe card: the
// same drawing in a golden frame, a shine passing over it, sparkles
// twinkling around. Hovered, what it says shows beside it, with its quip
// (CardPeek, through Peek).
Rectangle {
  id: root

  property var voucher: null

  readonly property bool deluxe: voucher !== null && !!voucher.follows
  readonly property string drawing: !voucher ? "" : deluxe ? voucher.follows : voucher.id
  readonly property bool drawn: Drawn.vouchers[drawing] === true
  readonly property color tint: deluxe ? Colors.warm : Colors.accent

  HoverHandler {
    onHoveredChanged: hovered && root.voucher
      ? Peek.showConsumable(root, {
          kind: "pull request", id: root.voucher.id, name: root.voucher.name,
          description: root.voucher.description, quip: root.voucher.quip,
          current: null, transparent: false }, "")
      : Peek.hide(root)
  }
  Component.onDestruction: Peek.hide(root)

  width: Style.space(120)
  height: Style.space(150)
  radius: Style.cornerRadius
  clip: true
  color: Qt.tint(Colors.background, Util.alpha(tint, deluxe ? 0.20 : 0.14))
  border.width: Style.space(2)
  border.color: tint

  // Deluxe: a second frame inside the first.
  Rectangle {
    anchors { fill: parent; margins: Style.space(4) }
    visible: root.deluxe
    radius: Style.cornerRadius
    color: "transparent"
    border.width: Style.normalBorderWidth
    border.color: Util.alpha(Colors.warm, 0.7)
  }

  Column {
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(root.deluxe ? 10 : 8) }
    spacing: Style.space(4)

    Item {
      id: pictureBox
      visible: root.drawn
      width: parent.width
      height: Math.round(width * 40 / 64)

      Rectangle {
        id: frame
        anchors.fill: parent
        color: Colors.background
        border.width: root.deluxe ? Style.space(2) : Style.normalBorderWidth
        border.color: root.tint
        clip: true
        JokerArt {
          anchors { fill: parent; margins: parent.border.width }
          family: "vouchers"
          jokerId: root.drawing
        }
        // Deluxe: a shine sweeping across.
        Rectangle {
          id: shine
          visible: root.deluxe
          width: frame.width * 0.3
          height: frame.height * 2
          y: -frame.height / 2
          rotation: 20
          gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.5; color: Qt.rgba(1, 1, 1, 0.5) }
            GradientStop { position: 1; color: "transparent" }
          }
          SequentialAnimation on x {
            running: shine.visible && root.visible
            loops: Animation.Infinite
            NumberAnimation { from: -shine.width * 1.5; to: frame.width + shine.width; duration: 800; easing.type: Easing.InOutQuad }
            PauseAnimation { duration: 1600 }
          }
        }
      }

      // Deluxe: sparkles twinkling at the corners, each at its own pace.
      Repeater {
        model: root.deluxe ? [[0.02, 0.05, 0], [0.96, 0.0, 500], [0.9, 0.92, 1000], [0.04, 0.88, 1500], [0.55, -0.08, 750]] : []
        Text {
          x: modelData[0] * pictureBox.width - width / 2
          y: modelData[1] * pictureBox.height - height / 2
          text: "✦"
          color: Colors.warm
          font.pixelSize: Style.font.heading
          opacity: 0
          SequentialAnimation on opacity {
            running: root.visible
            loops: Animation.Infinite
            PauseAnimation { duration: modelData[2] }
            NumberAnimation { to: 1; duration: 350; easing.type: Easing.OutQuad }
            NumberAnimation { to: 0; duration: 650; easing.type: Easing.InQuad }
            PauseAnimation { duration: 2000 - modelData[2] }
          }
        }
      }
    }

    Text {
      width: parent.width
      visible: !root.drawn
      horizontalAlignment: Text.AlignHCenter
      text: I18n.tr("pull request")
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      // Under a drawing, on one line, smaller if need be: the effect keeps
      // its room.
      wrapMode: root.drawn ? Text.NoWrap : Text.WordWrap
      fontSizeMode: root.drawn ? Text.HorizontalFit : Text.FixedSize
      minimumPixelSize: Style.font.caption
      text: root.voucher ? root.voucher.name : ""
      color: root.tint
      font.family: Style.font.family
      font.pixelSize: Style.font.subtitle
      font.bold: true
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      textFormat: Text.StyledText
      text: root.voucher ? Labels.highlight(root.voucher.description, Hints.terms, root.voucher.name) : ""
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    // Not drawn: its quip on the card, as before.
    Text {
      width: parent.width
      visible: !root.drawn
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.voucher ? root.voucher.quip : ""
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.italic: true
    }
  }
}
