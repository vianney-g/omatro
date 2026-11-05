import QtQuick
import omatro

// A card's enhancement as a band at the bottom of the card: in the theme hue
// of the enhancement, with an icon and a small motion of its own while the
// card is in view (`lively`), and a burst when the enhancement scores
// (`activate()`):
// - Optimized: a light sweeps the band, like a build bar;
// - Multithreaded: three threads run along it, each at its own pace;
// - Overclocked: the band runs hot, and shakes when it scores;
// - Daemon: a gear turns slowly in the background;
// - Symlink: the icon goes through every suit;
// - Binary Blob: bits scroll behind the name;
// - Enterprise: a golden light over the dollar sign;
// - Flaky Test: the icon passes and fails at random.
Rectangle {
  id: root

  // The enhancement's id ("" for none) and the text shown (its name, chips gained).
  property string kind: ""
  property alias text: label.text
  property bool lively: true

  // The enhancement's hue in the theme.
  readonly property color hue: hueOf(kind)
  function hueOf(id) {
    switch (id) {
    case "optimized": return Colors.cyan
    case "multithreaded": return Colors.urgent
    case "overclocked": return Colors.orange
    case "daemon": return Colors.magenta
    case "symlink": return Colors.accent
    case "binary_blob": return Colors.green
    case "enterprise": return Colors.warm
    case "flaky_test": return flaky.passing ? Colors.green : Colors.urgent
    default: return Colors.accent
    }
  }
  readonly property string icon: {
    switch (kind) {
    case "optimized": return "»"
    case "multithreaded": return "≡"
    case "overclocked": return "▲"
    case "daemon": return "⚙"
    case "symlink": return "→" + suits.symbols[suits.index]
    case "binary_blob": return "01"
    case "enterprise": return "$"
    case "flaky_test": return flaky.passing ? "✓" : "✗"
    default: return ""
    }
  }
  readonly property bool running: lively && visible && kind !== ""

  // The enhancement scored: the band pops, flashes and plays its burst.
  function activate() {
    burst.restart()
    if (kind === "optimized" || kind === "enterprise") { shine.stop(); shineFast.restart() }
    else if (kind === "daemon") spin.restart()
    else if (kind === "multithreaded") threads.rush()
  }

  height: label.implicitHeight + Style.space(4)
  radius: Style.cornerRadius
  clip: true
  border.width: Style.normalBorderWidth
  border.color: Util.alpha(hue, 0.85)
  Behavior on border.color { ColorAnimation { duration: 120 } }
  gradient: Gradient {
    orientation: Gradient.Horizontal
    GradientStop { position: 0; color: Util.alpha(root.hue, root.kind === "overclocked" ? 0.25 + 0.3 * heat.level : 0.4) }
    GradientStop { position: 1; color: Util.alpha(root.kind === "overclocked" ? Colors.urgent : root.hue, 0.12) }
  }

  // ---- what moves behind the text ----

  // Optimized, Enterprise: a light sweeping the band now and then.
  Rectangle {
    id: light
    visible: root.kind === "optimized" || root.kind === "enterprise"
    width: root.width * 0.35
    height: root.height
    x: -width
    gradient: Gradient {
      orientation: Gradient.Horizontal
      GradientStop { position: 0; color: "transparent" }
      GradientStop { position: 0.5; color: Util.alpha(root.kind === "enterprise" ? Qt.lighter(Colors.warm, 1.4) : Colors.foreground, 0.35) }
      GradientStop { position: 1; color: "transparent" }
    }
    SequentialAnimation on x {
      id: shine
      running: root.running && light.visible
      loops: Animation.Infinite
      PauseAnimation { duration: 2200 }
      NumberAnimation { from: -light.width; to: root.width; duration: 900; easing.type: Easing.InOutQuad }
    }
    NumberAnimation on x {
      id: shineFast
      running: false
      from: -light.width; to: root.width; duration: 300
      onFinished: if (root.running) shine.restart()
    }
  }

  // Multithreaded: threads running along the band, each at its own pace.
  Item {
    id: threads
    anchors.fill: parent
    visible: root.kind === "multithreaded"
    // Faster for a moment after scoring.
    property real pace: 1
    function rush() { rushAnim.restart() }
    NumberAnimation on pace { id: rushAnim; running: false; from: 4; to: 1; duration: 900; easing.type: Easing.OutCubic }
    // Along the edges, clear of the text.
    Repeater {
      model: 3
      Rectangle {
        required property int index
        y: index === 2 ? threads.height - height - Style.space(1) : Style.space(1) + index * Style.space(2)
        width: threads.width * 0.3
        height: Math.max(1, Style.space(1))
        radius: height / 2
        color: Util.alpha(Colors.urgent, 0.7)
        property real phase: 0
        x: -width + (threads.width + width) * phase
        NumberAnimation on phase {
          running: root.running && threads.visible
          loops: Animation.Infinite
          from: 0; to: 1
          duration: (900 + 450 * index) / threads.pace
        }
      }
    }
  }

  // Overclocked: running hot.
  QtObject {
    id: heat
    property real level: 0
    property SequentialAnimation pulse: SequentialAnimation {
      running: root.running && root.kind === "overclocked"
      loops: Animation.Infinite
      NumberAnimation { target: heat; property: "level"; to: 1; duration: 380; easing.type: Easing.InOutSine }
      NumberAnimation { target: heat; property: "level"; to: 0; duration: 520; easing.type: Easing.InOutSine }
    }
  }

  // Binary Blob: bits scrolling behind the name.
  Text {
    id: bits
    visible: root.kind === "binary_blob"
    anchors.verticalCenter: parent.verticalCenter
    text: "0110 1001 0011 1010 0110 1001 0011 1010"
    color: Util.alpha(Colors.green, 0.45)
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
    NumberAnimation on x {
      running: root.running && bits.visible
      loops: Animation.Infinite
      from: 0; to: -bits.implicitWidth / 2
      duration: 6000
    }
  }

  // Symlink: the icon goes through the suits.
  QtObject {
    id: suits
    readonly property var symbols: ["♠", "♥", "♦", "♣"]
    property int index: 0
    property Timer timer: Timer {
      running: root.running && root.kind === "symlink"
      repeat: true
      interval: 700
      onTriggered: suits.index = (suits.index + 1) % 4
    }
  }

  // Flaky Test: passes, fails, passes…
  QtObject {
    id: flaky
    property bool passing: true
    property Timer timer: Timer {
      running: root.running && root.kind === "flaky_test"
      repeat: true
      interval: 900
      onTriggered: {
        flaky.passing = Math.random() < 0.7
        interval = 300 + Math.random() * 1500
      }
    }
  }

  // ---- icon and text ----
  Row {
    id: content
    anchors.centerIn: parent
    width: Math.min(implicitWidth, root.width - 2 * Style.space(2))
    spacing: Style.space(3)
    Text {
      id: iconText
      anchors.verticalCenter: parent.verticalCenter
      text: root.icon
      color: root.hue
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.bold: true
      // Daemon: turns slowly, and a whole turn more as it scores.
      property real idle: 0
      property real extra: 0
      rotation: idle + extra
      NumberAnimation on idle {
        running: root.running && root.kind === "daemon"
        loops: Animation.Infinite
        from: 0; to: 360; duration: 4000
      }
      NumberAnimation on extra {
        id: spin
        running: false
        from: 0; to: 360; duration: 450
        easing.type: Easing.OutCubic
      }
    }
    Text {
      id: label
      anchors.verticalCenter: parent.verticalCenter
      width: Math.min(implicitWidth, root.width - iconText.width - content.spacing - 2 * Style.space(2))
      fontSizeMode: Text.HorizontalFit
      minimumPixelSize: Style.fontPx(0.6)
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.bold: true
    }
  }

  // Lights up as it scores.
  Rectangle {
    id: flashCover
    anchors.fill: parent
    radius: root.radius
    color: root.hue
    opacity: 0
  }

  SequentialAnimation {
    id: burst
    ParallelAnimation {
      NumberAnimation { target: content; property: "scale"; to: 1.3; duration: 90; easing.type: Easing.OutQuad }
      NumberAnimation { target: flashCover; property: "opacity"; to: 0.55; duration: 90 }
    }
    ParallelAnimation {
      NumberAnimation { target: content; property: "scale"; to: 1; duration: 260; easing.type: Easing.OutBack }
      NumberAnimation { target: flashCover; property: "opacity"; to: 0; duration: 380 }
    }
  }
}
