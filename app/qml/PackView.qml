import QtQuick
import omatro
import "Drawn.js" as Drawn

// A pack for sale: its drawing (art/packs, one per kind, in the current
// theme's colours), name and what it holds. Its size shows as a stack: the
// drawing's frame alone for a normal pack, a second frame behind it for a
// full one, two behind and a shine passing over it for a meta one. Hovered,
// what it says shows beside it, with its quip (CardPeek, through Peek).
Rectangle {
  id: root

  property var pack: null

  readonly property bool drawn: pack !== null && Drawn.packs[pack.kind] === true
  // Frames behind the drawing: 0, 1 or 2.
  readonly property int layers: !pack ? 0 : pack.size === "meta" ? 2 : pack.size === "full" ? 1 : 0
  readonly property real step: Style.space(4)

  HoverHandler {
    onHoveredChanged: hovered && root.pack
      ? Peek.showConsumable(root, {
          kind: "package", id: root.pack.kind, name: root.pack.name,
          description: root.pack.description, quip: root.pack.quip,
          current: null, transparent: false }, "")
      : Peek.hide(root)
  }
  Component.onDestruction: Peek.hide(root)

  width: Style.space(120)
  height: Style.space(150)
  radius: Style.cornerRadius
  color: Qt.tint(Colors.background, Util.alpha(Colors.urgent, 0.10))
  border.width: Style.space(2)
  border.color: Util.alpha(Colors.urgent, 0.6)

  Column {
    id: content
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
    spacing: Style.space(4)

    // The stack: room kept for two frames behind, so that every pack's
    // drawing sits at the same place.
    Item {
      visible: root.drawn
      width: parent.width
      height: front.height + 2 * root.step

      Repeater {
        model: root.layers
        Rectangle {
          // The farthest first.
          readonly property int depth: root.layers - index
          x: depth * root.step
          y: 2 * root.step - depth * root.step
          width: front.width
          height: front.height
          color: Qt.tint(Colors.background, Util.alpha(Colors.urgent, 0.12 + 0.06 * (2 - depth)))
          border.width: Style.normalBorderWidth
          border.color: Util.alpha(Colors.urgent, 0.7)
        }
      }
      Rectangle {
        id: front
        y: 2 * root.step
        width: parent.width - 2 * root.step
        height: Math.round(width * 40 / 64)
        color: Colors.background
        border.width: Style.normalBorderWidth
        border.color: Colors.urgent
        clip: true
        JokerArt {
          anchors { fill: parent; margins: parent.border.width }
          family: "packs"
          jokerId: root.pack ? root.pack.kind : ""
        }
        // Meta: a shine sweeping across now and then.
        Rectangle {
          id: shine
          visible: root.layers === 2
          width: front.width * 0.3
          height: front.height * 2
          y: -front.height / 2
          rotation: 20
          gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.5; color: Qt.rgba(1, 1, 1, 0.45) }
            GradientStop { position: 1; color: "transparent" }
          }
          SequentialAnimation on x {
            running: shine.visible && root.visible
            loops: Animation.Infinite
            NumberAnimation { from: -shine.width * 1.5; to: front.width + shine.width; duration: 900; easing.type: Easing.InOutQuad }
            PauseAnimation { duration: 2200 }
          }
        }
      }
    }

    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      // On one line, smaller if need be: "jokers-git-meta" cut anywhere
      // reads badly.
      fontSizeMode: Text.HorizontalFit
      minimumPixelSize: Style.font.caption
      text: root.pack ? root.pack.name : ""
      color: Colors.urgent
      font.family: Style.font.family
      font.pixelSize: Style.font.subtitle
      font.bold: true
    }
    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.pack ? root.pack.description : ""
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
      text: root.pack ? root.pack.quip : ""
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.italic: true
    }
  }
}
