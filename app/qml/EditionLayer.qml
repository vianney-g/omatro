import QtQuick
import QtQuick.Effects
import omatro

// A card's or a joker's edition, as Hyprland would decorate a window. Laid
// twice by the views: inside the face, under its content (`behind: false`),
// and under the face itself (`behind: true`, the shadow):
// - Blur: frosted glass, colours drifting behind it;
// - Shadow: a real drop shadow, the card lifted off the table;
// - Animated: a holographic sheen sweeping across, in the theme's hues;
// - Transparent: the checkerboard of an image editor.
// MultiEffect draws nothing with QT_QPA_PLATFORM=offscreen: check Blur and
// Shadow in a real window.
Item {
  id: root

  // The edition's id ("" for none).
  property string kind: ""
  property bool behind: false
  property real radius: Style.cornerRadius
  // Moves while in view (off where many cards show at once).
  property bool lively: true
  readonly property bool running: lively && visible

  // ======== under the face: the shadow ========
  // Drawn by its shape rather than from a texture: a MultiEffect shadow
  // came out stretched, and light, once the window was resized.
  RectangularShadow {
    anchors.fill: parent
    visible: root.behind && root.kind === "shadow"
    radius: root.radius
    color: Qt.rgba(0, 0, 0, 0.55)
    blur: Style.space(22)
    offset.x: Style.space(5)
    offset.y: Style.space(9)
  }

  // ======== inside the face, under its content ========
  Item {
    anchors { fill: parent; margins: Style.normalBorderWidth }
    visible: !root.behind && root.kind !== ""
    clip: true

    // Blur: colours drifting behind frosted glass.
    Item {
      id: blobs
      anchors.fill: parent
      visible: false
      Repeater {
        model: [Colors.accent, Colors.magenta, Colors.cyan]
        Rectangle {
          required property int index
          required property color modelData
          width: blobs.width * 0.75
          height: width
          radius: width / 2
          color: modelData
          property real t: index / 3
          x: blobs.width * (0.5 + 0.45 * Math.cos(2 * Math.PI * t)) - width / 2
          y: blobs.height * (0.5 + 0.4 * Math.sin(2 * Math.PI * (t + index * 0.2))) - height / 2
          NumberAnimation on t {
            running: root.running && root.kind === "blur"
            loops: Animation.Infinite
            from: index / 3; to: index / 3 + 1
            duration: 9000 + 2500 * index
          }
        }
      }
    }
    MultiEffect {
      anchors.fill: parent
      visible: root.kind === "blur"
      source: blobs
      blurEnabled: true
      blur: 1
      blurMax: 64
      opacity: 0.35
    }
    // The frost: lighter at the top, like glass catching the light.
    Rectangle {
      anchors.fill: parent
      visible: root.kind === "blur"
      gradient: Gradient {
        GradientStop { position: 0; color: Util.alpha(Colors.background, 0.45) }
        GradientStop { position: 0.6; color: Util.alpha(Colors.background, 0.1) }
        GradientStop { position: 1; color: "transparent" }
      }
    }

    // Animated: a holographic sheen, sweeping across diagonally.
    Rectangle {
      id: holo
      visible: root.kind === "animated"
      width: parent.width * 3
      height: parent.height * 3
      y: -parent.height
      rotation: 25
      property real shift: 0
      x: -parent.width * 2 + parent.width * 2 * shift
      gradient: Gradient {
        orientation: Gradient.Horizontal
        GradientStop { position: 0.0; color: "transparent" }
        GradientStop { position: 0.30; color: Util.alpha(Colors.magenta, 0.22) }
        GradientStop { position: 0.40; color: Util.alpha(Colors.cyan, 0.22) }
        GradientStop { position: 0.50; color: Util.alpha(Colors.green, 0.2) }
        GradientStop { position: 0.60; color: Util.alpha(Colors.warm, 0.22) }
        GradientStop { position: 0.70; color: Util.alpha(Colors.urgent, 0.18) }
        GradientStop { position: 1.0; color: "transparent" }
      }
      SequentialAnimation on shift {
        running: root.running && root.kind === "animated"
        loops: Animation.Infinite
        NumberAnimation { from: 0; to: 1; duration: 2800; easing.type: Easing.InOutSine }
        NumberAnimation { from: 1; to: 0; duration: 2800; easing.type: Easing.InOutSine }
      }
    }

    // Transparent: the checkerboard of an image editor.
    Grid {
      id: checker
      visible: root.kind === "transparent"
      readonly property real cell: Style.space(8)
      // Columns only: the rows follow the cells (a fixed row count warns
      // while a resize changes both).
      readonly property int lines: Math.ceil(parent.height / cell)
      columns: Math.ceil(parent.width / cell)
      Repeater {
        model: checker.visible ? checker.columns * checker.lines : 0
        Rectangle {
          required property int index
          width: checker.cell
          height: checker.cell
          color: (Math.floor(index / checker.columns) + index % checker.columns) % 2
            ? Util.alpha(Colors.foreground, 0.09) : Util.alpha(Colors.background, 0.5)
        }
      }
    }
  }
}
