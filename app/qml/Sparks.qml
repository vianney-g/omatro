import QtQuick
import QtQuick.Particles
import omatro

// Sparks bursting out of an item, over everything: `burst(item, color)`
// when a bonus lands on it. Fill the window with it.
Item {
  id: root

  ParticleSystem { id: system }

  // One painter per colour in use, so each burst keeps its own colour.
  ImageParticle {
    system: system; groups: ["chips"]
    source: "qrc:///particleresources/star.png"
    color: Colors.accent; colorVariation: 0.2
  }
  ImageParticle {
    system: system; groups: ["mult"]
    source: "qrc:///particleresources/star.png"
    color: Colors.urgent; colorVariation: 0.2
  }
  ImageParticle {
    system: system; groups: ["warm"]
    source: "qrc:///particleresources/star.png"
    color: Colors.warm; colorVariation: 0.2
  }

  component Burst: Emitter {
    system: system
    enabled: false
    lifeSpan: 500; lifeSpanVariation: 100
    size: Style.space(12); endSize: Style.space(2)
    velocity: AngleDirection {
      angleVariation: 360
      magnitude: Style.space(160); magnitudeVariation: Style.space(60)
    }
    acceleration: PointDirection { y: Style.space(300) }
  }
  Burst { id: chips; group: "chips" }
  Burst { id: mult; group: "mult" }
  Burst { id: warm; group: "warm" }

  // `kind`: "chips", "mult" or "warm"; sparks fly from the item's top edge.
  function burst(item, kind) {
    if (!item) return
    var p = item.mapToItem(root, item.width / 2, 0)
    var emitter = kind === "chips" ? chips : kind === "mult" ? mult : warm
    emitter.burst(14, p.x, p.y)
  }
}
