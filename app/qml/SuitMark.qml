import QtQuick
import omatro
import "Labels.js" as Labels

// The big suit symbol in the middle of a card, drawn after its enhancement,
// which tints the card in its hue (`hue`):
// - Optimized: leaning forward, speed lines streaming behind it;
// - Multithreaded: four smaller symbols, the cores, each busy at its own pace;
// - Symlink: behind the symbol, a ghost of each other suit in turn, as it
//   counts as every one of them; now and then it glitches through them all;
// - Overclocked: the symbol in a heat haze (shaders/heat.frag), hot from its
//   bottom up, with embers rising and the card warming from below;
// - Daemon: a little devil (😈) in the card's colour instead of the suit,
//   hovering;
// - Binary Blob: no symbol, bits scrolling over the card, BLOB over them;
// - Enterprise: the symbol in gold foil (shaders/foil.frag), a light passing
//   over it now and then, trademarked;
// - Flaky Test: the symbol split in two, its halves out of line; it jitters,
//   passes (green) and fails (red, dim) at random;
// - none: the plain symbol.
// Fills the card face; each stays recognisable when not `lively`.
Item {
  id: root

  // The enhancement's id ("" for none), the card's suit id, the text shown
  // (its suit symbol, BLOB).
  property string kind: ""
  property string suit: ""
  property string text: ""
  property color ink: Colors.foreground
  property int pixelSize: Style.fontPx(2)
  property bool bold: false
  property bool lively: true
  readonly property bool running: lively && visible

  // The enhancement's hue in the theme (the card's tint, its glow).
  readonly property color hue: {
    switch (kind) {
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

  function inkOf(s) { return Labels.isRed(s) ? Colors.urgent : Colors.foreground }

  // The enhancement scored.
  function activate() {
    switch (kind) {
    case "optimized": dash.restart(); break
    case "multithreaded": cores.rush(); break
    case "overclocked": overheat.restart(); break
    case "symlink": glitchTimer.restart(); glitchAnim.restart(); break
    case "daemon": hop.restart(); break
    case "binary_blob": bits.rush(); break
    case "enterprise": shine.stop(); shineFast.restart(); break
    case "flaky_test": flaky.burst(); break
    }
  }

  // ======== none ========
  Text {
    anchors.centerIn: parent
    visible: root.kind === ""
    text: root.text
    color: root.ink
    font.family: Style.font.family
    font.pixelSize: root.pixelSize
    font.bold: root.bold
  }

  // ======== Optimized: built for speed ========
  Item {
    id: fast
    anchors.centerIn: parent
    visible: root.kind === "optimized"
    width: fastGlyph.implicitWidth
    height: fastGlyph.implicitHeight
    // How far it has dashed forward (as it scores).
    property real lunge: 0
    SequentialAnimation {
      id: dash
      NumberAnimation { target: fast; property: "lunge"; to: 1; duration: 110; easing.type: Easing.OutQuad }
      NumberAnimation { target: fast; property: "lunge"; to: 0; duration: 420; easing.type: Easing.OutBack }
    }

    // Speed lines behind it, streaming away.
    Repeater {
      model: 3
      Rectangle {
        id: line
        required property int index
        readonly property real length: fast.width * [0.55, 0.8, 0.45][index] * (1 + fast.lunge)
        property real t: [0.2, 0.6, 0.9][index]
        width: length * (1 - 0.4 * t)
        height: Math.max(2, Style.space(2))
        radius: height / 2
        x: -length * 0.55 - Style.space(10) * t + Style.space(8) * fast.lunge
        y: fast.height * [0.36, 0.52, 0.68][index]
        color: Util.alpha(Colors.cyan, 0.75 * (1 - t) + 0.15)
        NumberAnimation on t {
          running: root.running && fast.visible
          loops: Animation.Infinite
          from: 0; to: 1
          duration: 520 + 160 * line.index
        }
      }
    }
    Text {
      id: fastGlyph
      x: Style.space(8) * fast.lunge
      text: root.text
      color: root.ink
      font.family: Style.font.family
      font.pixelSize: root.pixelSize
      // Leaning into the run.
      transform: Matrix4x4 {
        matrix: Qt.matrix4x4(1, -0.22, 0, fastGlyph.height * 0.11, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)
      }
    }
  }

  // ======== Multithreaded: the cores ========
  Grid {
    id: cores
    anchors.centerIn: parent
    visible: root.kind === "multithreaded"
    columns: 2
    columnSpacing: Style.space(2)
    rowSpacing: -Style.space(2)
    // Faster for a moment after scoring.
    property real pace: 1
    function rush() { rushAnim.restart() }
    NumberAnimation on pace { id: rushAnim; running: false; from: 4; to: 1; duration: 1100; easing.type: Easing.OutCubic }

    Repeater {
      model: 4
      Text {
        id: core
        required property int index
        text: root.text
        color: root.ink
        font.family: Style.font.family
        font.pixelSize: Math.round(root.pixelSize * 0.52)
        transformOrigin: Item.Center
        // Each core works, then rests, at its own pace.
        SequentialAnimation {
          running: root.running && cores.visible
          loops: Animation.Infinite
          onStopped: { core.scale = 1; core.opacity = 1 }
          PauseAnimation { duration: (180 + 230 * ((core.index * 3) % 4)) / cores.pace }
          ParallelAnimation {
            NumberAnimation { target: core; property: "scale"; to: 1.18; duration: 140 / cores.pace; easing.type: Easing.OutQuad }
            NumberAnimation { target: core; property: "opacity"; to: 1; duration: 140 / cores.pace }
          }
          ParallelAnimation {
            NumberAnimation { target: core; property: "scale"; to: 1; duration: (260 + 90 * core.index) / cores.pace; easing.type: Easing.InOutQuad }
            NumberAnimation { target: core; property: "opacity"; to: 0.7; duration: (260 + 90 * core.index) / cores.pace }
          }
        }
      }
    }
  }

  // ======== Overclocked: running hot ========
  // How hot: 0.5 to 0.8 at rest, past 1 as it scores.
  property real heat: 0.65
  property real time: 0
  readonly property bool hot: root.kind === "overclocked"
  SequentialAnimation on heat {
    running: root.running && root.hot && !overheat.running
    loops: Animation.Infinite
    NumberAnimation { to: 0.8; duration: 700; easing.type: Easing.InOutSine }
    NumberAnimation { to: 0.5; duration: 900; easing.type: Easing.InOutSine }
  }
  SequentialAnimation {
    id: overheat
    NumberAnimation { target: root; property: "heat"; to: 1.8; duration: 120; easing.type: Easing.OutQuad }
    NumberAnimation { target: root; property: "heat"; to: 0.65; duration: 900; easing.type: Easing.InQuad }
  }
  NumberAnimation on time {
    running: root.running && root.hot
    loops: Animation.Infinite
    from: 0; to: 2 * Math.PI
    duration: 2400
  }

  // The card warms from below.
  Rectangle {
    anchors.fill: parent
    visible: root.hot
    radius: Style.cornerRadius
    gradient: Gradient {
      GradientStop { position: 0.35; color: "transparent" }
      GradientStop { position: 1; color: Util.alpha(Colors.orange, 0.12 + 0.18 * Math.min(root.heat, 1.4)) }
    }
  }

  // Embers rising past the symbol.
  Repeater {
    model: root.hot ? 5 : 0
    Rectangle {
      id: ember
      required property int index
      readonly property real spread: [-0.22, 0.12, -0.05, 0.25, -0.15][index]
      property real t: [0.1, 0.55, 0.3, 0.8, 0.45][index]
      width: Math.max(2, Style.space(3) - index % 2 * Style.space(1))
      height: width
      radius: width / 2
      color: index % 2 ? Colors.orange : Colors.warm
      x: root.width * (0.5 + spread) - width / 2 + Math.sin(t * 7 + index) * Style.space(4)
      y: root.height * (0.78 - 0.55 * t)
      opacity: Math.sin(Math.PI * t) * 0.9
      NumberAnimation on t {
        running: root.running
        loops: Animation.Infinite
        from: 0; to: 1
        duration: 1500 + 350 * ember.index
      }
    }
  }

  // The symbol, drawn through the heat shader (with room to ripple).
  Item {
    anchors.centerIn: parent
    visible: root.hot
    width: hotGlyph.implicitWidth * 1.5
    height: hotGlyph.implicitHeight * 1.2
    layer.enabled: visible
    layer.effect: ShaderEffect {
      property real time: root.time
      property real strength: root.heat
      property color hot: Colors.orange
      fragmentShader: "../shaders/heat.frag.qsb"
    }
    Text {
      id: hotGlyph
      anchors.centerIn: parent
      text: root.text
      color: root.ink
      font.family: Style.font.family
      font.pixelSize: root.pixelSize
    }
  }

  // ======== Symlink: a link to every suit ========
  // The card's own suit, with a ghost of the suit it points to now behind it
  // (each of the others in turn), and a glitch now and then.
  property string shown: suit
  property int target: 1
  property real glitch: 0
  readonly property bool linking: kind === "symlink"
  readonly property var suitOrder: ["spades", "hearts", "clubs", "diamonds"]
  function otherSuit(step) {
    var i = suitOrder.indexOf(suit)
    return suitOrder[((i < 0 ? 0 : i) + step) % 4]
  }

  Timer {
    running: root.running && root.linking
    repeat: true
    interval: 1400
    onTriggered: root.target = root.target % 3 + 1
  }
  Timer {
    id: glitchTimer
    running: root.running && root.linking
    repeat: true
    interval: 4300
    onTriggered: glitchAnim.restart()
  }
  SequentialAnimation {
    id: glitchAnim
    onStopped: { root.shown = Qt.binding(function () { return root.suit }); root.glitch = 0 }
    ScriptAction { script: { root.glitch = 1; root.shown = root.otherSuit(1) } }
    PauseAnimation { duration: 80 }
    ScriptAction { script: root.shown = root.otherSuit(2) }
    PauseAnimation { duration: 80 }
    ScriptAction { script: root.shown = root.otherSuit(3) }
    PauseAnimation { duration: 80 }
    ScriptAction { script: { root.shown = root.suit; root.glitch = 0.4 } }
    PauseAnimation { duration: 70 }
    ScriptAction { script: root.glitch = 0 }
  }

  Item {
    anchors.centerIn: parent
    visible: root.linking
    width: linkGlyph.implicitWidth
    height: linkGlyph.implicitHeight

    // Where it points: up and to the right, faint.
    Text {
      x: Style.space(9)
      y: -Style.space(7)
      text: Labels.suit(root.otherSuit(root.target))
      color: Util.alpha(root.inkOf(root.otherSuit(root.target)), 0.28)
      font.family: Style.font.family
      font.pixelSize: root.pixelSize
    }
    // Split colours while it glitches, off either side.
    Repeater {
      model: root.glitch > 0 ? [Colors.cyan, Colors.magenta] : []
      Text {
        required property color modelData
        required property int index
        x: (index ? 1 : -1) * Style.space(4) * root.glitch
        y: (index ? -1 : 1) * Style.space(1) * root.glitch
        text: Labels.suit(root.shown)
        color: Util.alpha(modelData, 0.6)
        font.family: Style.font.family
        font.pixelSize: root.pixelSize
      }
    }
    Text {
      id: linkGlyph
      text: Labels.suit(root.shown)
      color: root.inkOf(root.shown)
      font.family: Style.font.family
      font.pixelSize: root.pixelSize
    }
  }

  // ======== Daemon: a little devil ========
  // 😈 in the card's colour, hovering. DejaVu Sans draws it in one colour
  // (colour emoji fonts ignore the text colour); without it, Qt falls back to
  // whatever emoji font there is.
  Text {
    id: devil
    anchors.centerIn: parent
    anchors.verticalCenterOffset: -Style.space(2) * bob
    visible: root.kind === "daemon"
    text: "😈\uFE0E"
    color: root.ink
    font.family: "DejaVu Sans"
    font.pixelSize: Math.round(root.pixelSize * 0.9)
    rotation: 6 * sway

    property real bob: 0
    property real sway: 0
    SequentialAnimation on bob {
      running: root.running && devil.visible
      loops: Animation.Infinite
      NumberAnimation { to: 1; duration: 900; easing.type: Easing.InOutSine }
      NumberAnimation { to: 0; duration: 900; easing.type: Easing.InOutSine }
    }
    SequentialAnimation on sway {
      running: root.running && devil.visible
      loops: Animation.Infinite
      NumberAnimation { to: 1; duration: 1300; easing.type: Easing.InOutSine }
      NumberAnimation { to: -1; duration: 2600; easing.type: Easing.InOutSine }
      NumberAnimation { to: 0; duration: 1300; easing.type: Easing.InOutSine }
    }
    // Scored: a hop.
    SequentialAnimation {
      id: hop
      NumberAnimation { target: devil; property: "bob"; to: 3; duration: 140; easing.type: Easing.OutQuad }
      NumberAnimation { target: devil; property: "bob"; to: 0; duration: 420; easing.type: Easing.OutBounce }
    }
  }

  // ======== Binary Blob: nobody knows what's inside ========
  Item {
    id: bits
    anchors { fill: parent; margins: Style.space(5) }
    visible: root.kind === "binary_blob"
    clip: true
    // Faster for a moment after scoring.
    property real pace: 1
    function rush() { bitsRush.restart() }
    NumberAnimation on pace { id: bitsRush; running: false; from: 6; to: 1; duration: 1000; easing.type: Easing.OutCubic }
    property real scroll: 0
    NumberAnimation on scroll {
      running: root.running && bits.visible
      loops: Animation.Infinite
      from: 0; to: 1
      duration: 7000 / bits.pace
    }
    // The same rows twice, one under the other, for a seamless loop.
    readonly property var rows: {
      var out = []
      var n = 0x5bd1
      for (var i = 0; i < 12; i++) {
        var line = ""
        for (var j = 0; j < 8; j++) {
          n = (n * 1103 + 12345) % 65536
          line += n & 0x100 ? "1" : "0"
        }
        out.push(line)
      }
      return out.join("\n")
    }
    Column {
      anchors.horizontalCenter: parent.horizontalCenter
      y: -height / 2 * bits.scroll
      Repeater {
        model: 2
        Text {
          text: bits.rows
          color: Util.alpha(Colors.green, 0.4)
          lineHeight: 0.95
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
        }
      }
    }
  }
  Rectangle {
    anchors.centerIn: parent
    visible: root.kind === "binary_blob"
    width: blobText.implicitWidth + Style.space(6)
    height: blobText.implicitHeight + Style.space(2)
    radius: Style.cornerRadius
    color: Util.alpha(Colors.background, 0.85)
    border.width: Style.normalBorderWidth
    border.color: Util.alpha(Colors.green, 0.6)
    Text {
      id: blobText
      anchors.centerIn: parent
      text: root.text
      color: root.ink
      font.family: Style.font.family
      font.pixelSize: root.pixelSize
      font.bold: root.bold
    }
  }

  // ======== Enterprise: gold foil, with a support contract ========
  property real sweep: -0.5
  SequentialAnimation on sweep {
    id: shine
    running: root.running && root.kind === "enterprise"
    loops: Animation.Infinite
    PauseAnimation { duration: 2400 }
    NumberAnimation { from: -0.5; to: 1.5; duration: 1100; easing.type: Easing.InOutQuad }
  }
  NumberAnimation on sweep {
    id: shineFast
    running: false
    from: -0.5; to: 1.5; duration: 380
    onFinished: if (root.running) shine.restart()
  }
  Item {
    anchors.centerIn: parent
    visible: root.kind === "enterprise"
    width: goldGlyph.implicitWidth
    height: goldGlyph.implicitHeight

    // Embossed: a darker copy under it.
    Text {
      x: Style.space(1.5)
      y: Style.space(1.5)
      text: root.text
      color: Util.alpha(Qt.darker(Colors.warm, 2.2), 0.55)
      font.family: Style.font.family
      font.pixelSize: root.pixelSize
    }
    Item {
      anchors.fill: parent
      layer.enabled: parent.visible
      layer.effect: ShaderEffect {
        property real sweep: root.sweep
        property color gold: Colors.warm
        fragmentShader: "../shaders/foil.frag.qsb"
      }
      Text {
        id: goldGlyph
        text: root.text
        color: "white"
        font.family: Style.font.family
        font.pixelSize: root.pixelSize
      }
    }
    Text {
      anchors { left: parent.right; top: parent.top; leftMargin: -Style.space(4); topMargin: Style.space(4) }
      text: "™"
      color: Colors.warm
      font.family: Style.font.family
      font.pixelSize: Math.round(root.pixelSize * 0.3)
      font.bold: true
    }
  }

  // ======== Flaky Test: passes sometimes ========
  QtObject {
    id: flaky
    property bool passing: true
    // How far the halves are out of line (fractions of the symbol's width).
    property real top: 0.08
    property real bottom: -0.04
    function shake() {
      top = (Math.random() - 0.5) * 0.3
      bottom = (Math.random() - 0.5) * 0.2
    }
    function settle() {
      top = 0.08
      bottom = -0.04
    }
    // Scored: a spectacular pass.
    function burst() {
      passing = true
      jitter.restart()
    }
    property Timer timer: Timer {
      running: root.running && root.kind === "flaky_test"
      repeat: true
      interval: 1200
      onTriggered: {
        flaky.passing = Math.random() < 0.7
        interval = 500 + Math.random() * 2000
        jitter.restart()
      }
    }
  }
  SequentialAnimation {
    id: jitter
    ScriptAction { script: flaky.shake() }
    PauseAnimation { duration: 60 }
    ScriptAction { script: flaky.shake() }
    PauseAnimation { duration: 60 }
    ScriptAction { script: flaky.shake() }
    PauseAnimation { duration: 60 }
    ScriptAction { script: flaky.settle() }
  }
  Item {
    anchors.centerIn: parent
    visible: root.kind === "flaky_test"
    width: flakyHalf.implicitWidth
    height: flakyHalf.implicitHeight
    opacity: flaky.passing ? 1 : 0.45
    Behavior on opacity { NumberAnimation { duration: 120 } }

    // Top half, then bottom half, each clipped and out of line.
    Repeater {
      model: 2
      Item {
        required property int index
        x: parent.width * (index ? flaky.bottom : flaky.top)
        y: index * parent.height / 2
        width: parent.width
        height: parent.height / 2
        clip: true
        Text {
          y: -parent.y
          text: root.text
          color: root.ink
          font.family: Style.font.family
          font.pixelSize: root.pixelSize
        }
      }
    }
    // Only for its size.
    Text {
      id: flakyHalf
      visible: false
      text: root.text
      font.family: Style.font.family
      font.pixelSize: root.pixelSize
    }
  }
}
