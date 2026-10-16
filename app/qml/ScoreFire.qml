import QtQuick
import QtQuick.Particles
import omatro

// Flames filling the item it covers, from its bottom edge, behind what is
// declared after it: in the theme's red and yellow, a red outer layer, a blend,
// a yellow core and a few embers, each rising, shrinking and fading, shaken by
// turbulence. `heat` (0 to 3) sets how high and how thick they burn; at 0 the
// flames die down.
Item {
  id: root

  property int heat: 0
  clip: true

  // How high the flames rise (pixels per second for a given life).
  readonly property real rise: height * (0.35 + 0.2 * heat)

  readonly property color outer: Color.urgent
  readonly property color middle: Qt.tint(Color.urgent, Util.alpha(Color.warm, 0.6))
  readonly property color core: Qt.lighter(Color.warm, 1.25)

  // Always running: when the emitters stop, the flames still burn out.
  ParticleSystem { id: fire }

  ImageParticle {
    system: fire; groups: ["outer"]
    source: "qrc:///particleresources/glowdot.png"
    color: root.outer; colorVariation: 0.08
    alpha: 0.7
  }
  ImageParticle {
    system: fire; groups: ["middle"]
    source: "qrc:///particleresources/glowdot.png"
    color: root.middle; colorVariation: 0.08
    alpha: 0.8
  }
  ImageParticle {
    system: fire; groups: ["core"]
    source: "qrc:///particleresources/glowdot.png"
    color: root.core; colorVariation: 0.05
  }
  ImageParticle {
    system: fire; groups: ["ember"]
    source: "qrc:///particleresources/star.png"
    color: root.core; colorVariation: 0.2
  }

  Emitter {
    system: fire; group: "outer"
    x: Style.space(6); width: root.width - 2 * x; y: root.height - Style.space(4); height: Style.space(8)
    enabled: root.heat > 0
    emitRate: 70 * root.heat
    lifeSpan: 700; lifeSpanVariation: 150
    size: Style.space(44); endSize: Style.space(8); sizeVariation: Style.space(10)
    velocity: AngleDirection {
      angle: 270; angleVariation: 8
      magnitude: root.rise / 0.7; magnitudeVariation: root.rise / 4
    }
  }
  Emitter {
    system: fire; group: "middle"
    x: Style.space(10); width: root.width - 2 * x; y: root.height - Style.space(3); height: Style.space(6)
    enabled: root.heat > 0
    emitRate: 80 * root.heat
    lifeSpan: 550; lifeSpanVariation: 120
    size: Style.space(34); endSize: Style.space(4); sizeVariation: Style.space(8)
    velocity: AngleDirection {
      angle: 270; angleVariation: 8
      magnitude: root.rise * 0.8 / 0.55; magnitudeVariation: root.rise / 4
    }
  }
  Emitter {
    system: fire; group: "core"
    x: Style.space(14); width: root.width - 2 * x; y: root.height - Style.space(2); height: Style.space(4)
    enabled: root.heat > 0
    emitRate: 70 * root.heat
    lifeSpan: 380; lifeSpanVariation: 80
    size: Style.space(22); endSize: Style.space(2); sizeVariation: Style.space(6)
    velocity: AngleDirection {
      angle: 270; angleVariation: 6
      magnitude: root.rise * 0.5 / 0.38; magnitudeVariation: root.rise / 6
    }
  }
  Emitter {
    system: fire; group: "ember"
    width: root.width; y: root.height - root.rise; height: Style.space(4)
    enabled: root.heat > 0
    emitRate: 5 * root.heat
    lifeSpan: 900; lifeSpanVariation: 300
    size: Style.space(8); endSize: Style.space(2)
    velocity: AngleDirection {
      angle: 270; angleVariation: 25
      magnitude: Style.space(60); magnitudeVariation: Style.space(30)
    }
    acceleration: PointDirection { xVariation: Style.space(50) }
  }
  Turbulence {
    system: fire; groups: ["outer", "middle", "ember"]
    anchors.fill: parent
    strength: Style.space(30)
  }
}
