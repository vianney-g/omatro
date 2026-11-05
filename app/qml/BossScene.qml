import QtQuick
import omatro

// What the boss in play looks like over the play area, while its effect
// applies. Main lays it twice over the play area: `front: false` behind the
// cards (the scenery), `front: true` over them (what covers the screen:
// scanlines, the boss's entrance). Visual only, in the theme's colors:
// - The 4:3 Monitor: black bars on both sides;
// - The Screen Lock: a lock screen's big clock;
// - The Black Screen: a dead screen until the first hand is played, then it
//   boots (a CRT line opening up);
// - The Flaky Wi-Fi: a signal icon whose bars come and go;
// - The Windows Update: a progress bar stuck near the end, and a spinner;
// - The Kubernetes Cluster, The Microservices: pods blinking in a grid;
// - The Thermal Throttle: heat rising from the bottom;
// - The Kernel Panic: a stack trace scrolling, and the screen shakes;
// - The Rate Limiter: 429; The Read-Only Filesystem: r--r--r--;
// - The Deadline: a bar running out;
// - bosses that keep a suit (or face cards) from scoring: that suit, crossed out.
Item {
  id: root

  property bool front: false
  // The boss whose effect applies now ("" for none).
  property string boss: ""
  // The Black Screen: the hand is still dealt face down.
  property bool dark: false
  property string bossName: ""
  property string bossDescription: ""
  // The screen should shake (The Kernel Panic's entrance).
  signal quake()

  enabled: false  // never in the way of the pointer

  // The suit (or faces) a boss keeps from scoring.
  readonly property string crossed: {
    switch (boss) {
    case "electron_app": return "♣"
    case "nvidia_driver": return "♠"
    case "paywall": return "♦"
    case "burnout": return "♥"
    case "anonymous_account": return "K"
    default: return ""
    }
  }

  onBossChanged: {
    if (boss === "") return
    if (front) entrance.restart()
    if (front && boss === "kernel_panic") quake()
  }

  // Shown big and faint behind the cards.
  component Watermark: Text {
    anchors.centerIn: parent
    color: Util.alpha(Colors.foreground, 0.07)
    font.family: Style.font.family
    font.pixelSize: Math.min(root.height * 0.45, root.width * 0.3)
    font.bold: true
  }

  // ======== behind the cards ========
  Item {
    anchors.fill: parent
    visible: !root.front

    // The 4:3 Monitor: what is left of the screen is 4:3.
    Repeater {
      model: root.boss === "monitor43" ? 2 : 0
      Rectangle {
        required property int index
        // A 4:3 screen, or at least bands that show.
        readonly property real band: Math.max(root.width * 0.1, (root.width - root.height * 4 / 3) / 2)
        width: band
        height: root.height
        x: index === 0 ? 0 : root.width - width
        color: Qt.rgba(0, 0, 0, 0.78)
        NumberAnimation on width { from: 0; to: band; duration: 600; easing.type: Easing.OutCubic }
      }
    }

    // The Screen Lock: the clock of a lock screen.
    Column {
      anchors.centerIn: parent
      visible: root.boss === "screen_lock"
      spacing: Style.spacing.md
      Text {
        id: clock
        anchors.horizontalCenter: parent.horizontalCenter
        text: Qt.formatTime(new Date(), "hh:mm")
        color: Util.alpha(Colors.foreground, 0.12)
        font.family: Style.font.family
        font.pixelSize: root.height * 0.3
        font.bold: true
        Timer {
          running: root.boss === "screen_lock"
          interval: 5000
          repeat: true
          triggeredOnStart: true
          onTriggered: clock.text = Qt.formatTime(new Date(), "hh:mm")
        }
      }
      // The password field, dots appearing one by one.
      Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Style.spacing.lg
        property int typed: 0
        Timer {
          running: root.boss === "screen_lock"
          interval: 450
          repeat: true
          onTriggered: parent.typed = (parent.typed + 1) % 9
        }
        Repeater {
          model: 8
          Rectangle {
            required property int index
            width: Style.space(10); height: width; radius: width / 2
            color: Util.alpha(Colors.foreground, index < parent.typed ? 0.18 : 0.05)
          }
        }
      }
    }

    // The Black Screen: off, until the first hand is played.
    Rectangle {
      anchors.fill: parent
      color: "black"
      radius: Style.cornerRadius
      opacity: root.boss === "black_screen" && root.dark ? 0.82 : 0
      Behavior on opacity { NumberAnimation { duration: 700; easing.type: Easing.InQuad } }
    }

    // The Flaky Wi-Fi: a signal icon, its bars coming and going.
    Row {
      id: wifi
      visible: root.boss === "flaky_wifi"
      anchors { right: parent.right; top: parent.top; margins: Style.spacing.huge }
      spacing: Style.space(4)
      property int bars: 4
      Timer {
        running: wifi.visible
        interval: 600
        repeat: true
        onTriggered: {
          wifi.bars = Math.floor(Math.random() * 5)
          interval = 250 + Math.random() * 900
        }
      }
      Repeater {
        model: 4
        Rectangle {
          required property int index
          anchors.bottom: parent.bottom
          width: Style.space(10)
          height: Style.space(12) * (index + 1)
          radius: Style.space(2)
          color: index < wifi.bars ? Util.alpha(Colors.accent, 0.55) : Util.alpha(Colors.foreground, 0.08)
          Behavior on color { ColorAnimation { duration: 120 } }
        }
      }
    }

    // The Windows Update: a spinner, a bar stuck near the end.
    Column {
      visible: root.boss === "windows_update"
      anchors.centerIn: parent
      spacing: Style.spacing.xxl
      Item {
        anchors.horizontalCenter: parent.horizontalCenter
        width: Style.space(60); height: width
        RotationAnimation on rotation {
          running: root.boss === "windows_update"
          loops: Animation.Infinite
          from: 0; to: 360; duration: 1600
        }
        Repeater {
          model: 6
          Rectangle {
            required property int index
            width: Style.space(8); height: width; radius: width / 2
            // Around a circle, fading towards the tail.
            color: Util.alpha(Colors.accent, 0.08 + 0.07 * index)
            x: parent.width / 2 + parent.width / 2.4 * Math.cos(index * Math.PI / 3) - width / 2
            y: parent.height / 2 + parent.height / 2.4 * Math.sin(index * Math.PI / 3) - height / 2
          }
        }
      }
      Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        width: Style.space(260); height: Style.space(10)
        radius: height / 2
        color: Util.alpha(Colors.foreground, 0.06)
        Rectangle {
          id: progressBar
          property real done: 0
          width: parent.width * done; height: parent.height
          radius: parent.radius
          color: Util.alpha(Colors.accent, 0.3)
          // Fast at first, then stuck at 99%, then it starts over.
          SequentialAnimation on done {
            running: root.boss === "windows_update"
            loops: Animation.Infinite
            NumberAnimation { from: 0; to: 0.99; duration: 14000; easing.type: Easing.OutExpo }
            PauseAnimation { duration: 6000 }
          }
        }
      }
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: Math.floor(progressBar.done * 100) + "%"
        color: Util.alpha(Colors.foreground, 0.2)
        font.family: Style.font.family
        font.pixelSize: Style.font.display
        font.bold: true
      }
    }

    // The Kubernetes Cluster, The Microservices: pods coming up and down.
    Grid {
      id: pods
      visible: root.boss === "kubernetes_cluster" || root.boss === "microservices"
      anchors.centerIn: parent
      readonly property real cell: Style.space(root.boss === "microservices" ? 14 : 26)
      spacing: Math.round(cell / 3)
      columns: Math.max(1, Math.floor(root.width / (cell + spacing)))
      rows: Math.max(1, Math.floor(root.height / (cell + spacing)))
      Repeater {
        model: pods.visible ? pods.columns * pods.rows : 0
        Rectangle {
          width: pods.cell; height: pods.cell
          radius: Math.max(2, pods.cell / 5)
          color: Colors.accent
          opacity: 0.04
          SequentialAnimation on opacity {
            loops: Animation.Infinite
            PauseAnimation { duration: 500 + Math.random() * 6000 }
            NumberAnimation { to: 0.22; duration: 160 }
            PauseAnimation { duration: 300 + Math.random() * 2500 }
            NumberAnimation { to: 0.04; duration: 400 }
          }
        }
      }
    }

    // The Thermal Throttle: heat rising from the bottom.
    Rectangle {
      anchors.fill: parent
      visible: root.boss === "thermal_throttle"
      radius: Style.cornerRadius
      gradient: Gradient {
        GradientStop { position: 0.35; color: "transparent" }
        GradientStop { position: 0.8; color: Util.alpha(Colors.orange, 0.12) }
        GradientStop { position: 1; color: Util.alpha(Colors.urgent, 0.3) }
      }
      SequentialAnimation on opacity {
        running: root.boss === "thermal_throttle"
        loops: Animation.Infinite
        NumberAnimation { to: 0.55; duration: 1400; easing.type: Easing.InOutSine }
        NumberAnimation { to: 1; duration: 1400; easing.type: Easing.InOutSine }
      }
    }

    // The Kernel Panic: a stack trace scrolling up.
    Item {
      anchors.fill: parent
      visible: root.boss === "kernel_panic"
      clip: true
      Column {
        id: trace
        width: parent.width
        function hex(n) {
          var s = ""
          for (var i = 0; i < n; i++) s += "0123456789abcdef"[Math.floor(Math.random() * 16)]
          return s
        }
        Repeater {
          model: root.boss === "kernel_panic" ? 80 : 0
          Text {
            required property int index
            x: Style.spacing.huge
            text: "[" + (index * 0.0371).toFixed(6) + "] ffff" + trace.hex(12) + "  " + trace.hex(8) + " " + trace.hex(8)
            color: Util.alpha(Colors.urgent, 0.2)
            font.family: Style.font.family
            font.pixelSize: Style.font.caption
          }
        }
        NumberAnimation on y {
          running: root.boss === "kernel_panic"
          loops: Animation.Infinite
          from: 0; to: -trace.height / 2; duration: 40000
        }
      }
    }

    // The Deadline: time running out, along the top.
    Rectangle {
      visible: root.boss === "deadline"
      anchors { left: parent.left; top: parent.top; right: parent.right }
      height: Style.space(4)
      color: Util.alpha(Colors.foreground, 0.06)
      Rectangle {
        id: timeLeft
        height: parent.height
        width: parent.width
        color: Colors.urgent
        NumberAnimation on width {
          running: root.boss === "deadline"
          from: timeLeft.parent.width; to: 0; duration: 45000
        }
        // Empty: it blinks.
        SequentialAnimation on opacity {
          running: root.boss === "deadline"
          loops: Animation.Infinite
          NumberAnimation { to: 0.35; duration: 500 }
          NumberAnimation { to: 0.9; duration: 500 }
        }
      }
    }

    Watermark { visible: root.boss === "rate_limiter"; text: "429" }
    Watermark {
      visible: root.boss === "read_only_filesystem"
      text: "r--r--r--"
      font.pixelSize: Math.min(root.height * 0.3, root.width * 0.12)
    }
    // A suit (or the faces) that doesn't score: crossed out.
    Watermark {
      id: crossedOut
      visible: root.crossed !== ""
      text: root.crossed
      color: Util.alpha(root.crossed === "♥" || root.crossed === "♦" ? Colors.urgent : Colors.foreground, 0.08)
      Rectangle {
        anchors.centerIn: parent
        width: parent.font.pixelSize * 1.1
        height: Style.space(10)
        rotation: -35
        radius: height / 2
        color: Util.alpha(Colors.urgent, 0.18)
      }
    }
  }

  // ======== over the cards ========
  Item {
    anchors.fill: parent
    visible: root.front

    // The Black Screen: scanlines while it is off, and a CRT line as it boots.
    Column {
      anchors.fill: parent
      visible: root.boss === "black_screen" && root.dark
      clip: true
      Repeater {
        model: Math.ceil(root.height / Style.space(4))
        Rectangle {
          width: root.width
          height: Style.space(4)
          color: index % 2 ? Qt.rgba(0, 0, 0, 0.12) : "transparent"
        }
      }
    }
    Rectangle {
      id: crtLine
      anchors.centerIn: parent
      width: root.width
      height: 0
      opacity: 0
      color: Qt.lighter(Colors.accent, 1.6)
    }
    Connections {
      target: root
      function onDarkChanged() {
        if (root.front && root.boss === "black_screen" && !root.dark) boot.restart()
      }
    }
    SequentialAnimation {
      id: boot
      ScriptAction { script: { crtLine.height = Style.space(3); crtLine.opacity = 1 } }
      NumberAnimation { target: crtLine; property: "height"; to: root.height; duration: 260; easing.type: Easing.OutCubic }
      NumberAnimation { target: crtLine; property: "opacity"; to: 0; duration: 380 }
    }

    // The boss comes in: its name and effect slide down, then away.
    Rectangle {
      id: banner
      anchors.horizontalCenter: parent.horizontalCenter
      y: -height - Style.spacing.huge
      width: Math.min(root.width * 0.8, bannerText.implicitWidth + 2 * Style.spacing.huge)
      height: bannerText.implicitHeight + 2 * Style.spacing.xxl
      radius: Style.cornerRadius
      color: Colors.background
      border.width: Style.space(2)
      border.color: Colors.urgent
      opacity: 0
      Text {
        id: bannerText
        anchors.centerIn: parent
        width: Math.min(implicitWidth, root.width * 0.8 - 2 * Style.spacing.huge)
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        textFormat: Text.StyledText
        text: "<b>" + root.bossName + "</b><br>" + root.bossDescription
        color: Colors.urgent
        font.family: Style.font.family
        font.pixelSize: Style.font.title
      }
    }
    SequentialAnimation {
      id: entrance
      ParallelAnimation {
        NumberAnimation { target: banner; property: "y"; from: -banner.height; to: Style.spacing.huge * 2; duration: 420; easing.type: Easing.OutBack }
        NumberAnimation { target: banner; property: "opacity"; from: 0; to: 1; duration: 200 }
      }
      PauseAnimation { duration: 2200 }
      ParallelAnimation {
        NumberAnimation { target: banner; property: "y"; to: -banner.height; duration: 320; easing.type: Easing.InCubic }
        NumberAnimation { target: banner; property: "opacity"; to: 0; duration: 320 }
      }
    }
  }
}
