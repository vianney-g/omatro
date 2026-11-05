import QtQuick
import omatro

// A card's seal, in its corner, with a motion of its own while in view
// (`lively`) and a burst when it acts (`activate()`):
// - Gold Star ★: twinkles, glints lighting up around it; spins as it pays;
// - Retry ↻: rocks gently; spins round as the card scores again;
// - Theme Sync ◐: turns slowly, like day and night;
// - Shell History ❯: a cursor blinks beside it, as at a prompt.
Item {
  id: root

  // The seal's id ("" for none).
  property string kind: ""
  property bool lively: true
  readonly property bool running: lively && visible && kind !== ""

  readonly property string symbol: kind === "gold_star" ? "★" : kind === "retry" ? "↻"
    : kind === "theme_sync" ? "◐" : kind === "shell_history" ? "❯" : ""
  readonly property color hue: kind === "gold_star" ? Colors.warm : kind === "retry" ? Colors.urgent : Colors.accent

  function activate() {
    pop.restart()
    spin.restart()
  }

  implicitWidth: mark.implicitWidth + (kind === "shell_history" ? cursor.implicitWidth : 0)
  implicitHeight: mark.implicitHeight

  Text {
    id: mark
    text: root.symbol
    color: root.hue
    font.family: Style.font.family
    font.pixelSize: Style.font.heading
    font.bold: true
    // The idle motion, and a whole turn more as it acts.
    property real idle: 0
    property real extra: 0
    rotation: idle + extra

    // Retry: rocking; Theme Sync: turning.
    SequentialAnimation on idle {
      running: root.running && root.kind === "retry"
      loops: Animation.Infinite
      NumberAnimation { to: -20; duration: 900; easing.type: Easing.InOutSine }
      NumberAnimation { to: 20; duration: 1800; easing.type: Easing.InOutSine }
      NumberAnimation { to: 0; duration: 900; easing.type: Easing.InOutSine }
    }
    NumberAnimation on idle {
      running: root.running && root.kind === "theme_sync"
      loops: Animation.Infinite
      from: 0; to: 360; duration: 8000
    }
    NumberAnimation on extra {
      id: spin
      running: false
      from: 0; to: root.kind === "retry" ? -720 : 360
      duration: 500
      easing.type: Easing.OutCubic
    }
    // Gold Star: twinkling.
    SequentialAnimation on opacity {
      running: root.running && root.kind === "gold_star"
      loops: Animation.Infinite
      PauseAnimation { duration: 1200 + Math.random() * 1500 }
      NumberAnimation { to: 0.55; duration: 160 }
      NumberAnimation { to: 1; duration: 260 }
    }
  }

  // Gold Star: glints lighting up around it, one at a time.
  Repeater {
    model: root.kind === "gold_star" ? 3 : 0
    Text {
      id: glint
      required property int index
      readonly property real angle: index * 2.1 + 0.6
      x: mark.width / 2 + mark.width * 0.75 * Math.cos(angle) - width / 2
      y: mark.height / 2 + mark.height * 0.6 * Math.sin(angle) - height / 2
      text: "✦"
      color: Qt.lighter(Colors.warm, 1.3)
      font.family: Style.font.family
      font.pixelSize: Math.round(Style.font.caption * 0.7)
      opacity: 0
      scale: 0.5
      SequentialAnimation {
        running: root.running
        loops: Animation.Infinite
        PauseAnimation { duration: 700 + glint.index * 900 + Math.random() * 1500 }
        ParallelAnimation {
          NumberAnimation { target: glint; property: "opacity"; to: 1; duration: 180 }
          NumberAnimation { target: glint; property: "scale"; to: 1.2; duration: 180 }
        }
        ParallelAnimation {
          NumberAnimation { target: glint; property: "opacity"; to: 0; duration: 360 }
          NumberAnimation { target: glint; property: "scale"; to: 0.5; duration: 360 }
        }
        PauseAnimation { duration: 1500 }
      }
    }
  }

  // Shell History: the prompt's cursor.
  Text {
    id: cursor
    visible: root.kind === "shell_history"
    anchors { left: mark.right; baseline: mark.baseline }
    text: "_"
    color: root.hue
    font.family: Style.font.family
    font.pixelSize: Style.font.body
    font.bold: true
    SequentialAnimation on opacity {
      running: root.running && root.kind === "shell_history"
      loops: Animation.Infinite
      PropertyAction { value: 1 }
      PauseAnimation { duration: 530 }
      PropertyAction { value: 0 }
      PauseAnimation { duration: 530 }
    }
  }

  // Acting: it swells for a moment.
  SequentialAnimation {
    id: pop
    NumberAnimation { target: mark; property: "scale"; to: 1.6; duration: 110; easing.type: Easing.OutQuad }
    NumberAnimation { target: mark; property: "scale"; to: 1; duration: 300; easing.type: Easing.OutBack }
  }
}
