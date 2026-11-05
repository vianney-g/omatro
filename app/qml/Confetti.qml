import QtQuick
import QtQuick.Particles
import omatro

// Confetti in the theme's hues raining over the window for a few seconds,
// fluttering as they fall: `start()` (the game won). Fill the window with it.
Item {
  id: root

  enabled: false  // never in the way of the pointer

  function start() {
    rain.enabled = true
    stopRain.restart()
  }

  readonly property var hues: [Colors.accent, Colors.urgent, Colors.warm, Colors.green, Colors.cyan, Colors.magenta]

  ParticleSystem { id: system }

  Emitter {
    id: rain
    system: system
    enabled: false
    anchors { left: parent.left; right: parent.right; top: parent.top }
    height: 1
    emitRate: 90
    lifeSpan: 4500
    size: Style.space(10)
    velocity: PointDirection { y: Style.space(160); yVariation: Style.space(90); xVariation: Style.space(80) }
    acceleration: PointDirection { y: Style.space(40) }
  }
  Timer { id: stopRain; interval: 2600; onTriggered: rain.enabled = false }

  // Fluttering sideways.
  Wander {
    system: system
    xVariance: Style.space(120)
    pace: Style.space(200)
  }

  ItemParticle {
    system: system
    fade: true
    delegate: Rectangle {
      width: Style.space(6 + Math.random() * 6)
      height: Style.space(3 + Math.random() * 3)
      radius: Style.space(1)
      color: root.hues[Math.floor(Math.random() * root.hues.length)]
      // Each one turns at its own pace.
      NumberAnimation on rotation {
        loops: Animation.Infinite
        from: Math.random() * 360
        to: from + (Math.random() < 0.5 ? -360 : 360)
        duration: 600 + Math.random() * 1400
      }
    }
  }
}
