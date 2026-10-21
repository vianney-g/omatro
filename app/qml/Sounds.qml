import QtQuick
import QtMultimedia

// The sound effects (app/sounds, WAV). Main plays them by name through
// play(); like Music, it is loaded through a Loader, so a missing Qt
// Multimedia leaves the game silent.
Item {
  id: root

  property bool active: false
  // 0 to 1, as the player sets it (heard on a squared scale).
  property real volume: 0.5

  // Each sound and its own level, to even them out.
  readonly property var levels: ({
    "select": 0.5, "deselect": 0.35, "play": 1, "discard": 1, "draw": 0.6, "deal": 0.9,
    "mult": 0.5, "x-mult": 0.6, "money": 0.8, "debuffed": 0.4, "fire": 0.5,
    "blind-won": 0.6, "victory": 0.6, "game-over": 0.3, "boss": 0.8,
    "cash-out": 1, "buy": 0.5, "sell": 1, "reroll": 1, "pack": 1, "pick": 1, "skip": 0.5,
    "use": 0.5, "merge": 0.7, "achievement": 0.7, "found": 0.7, "error": 0.5,
  })
  // The chips steps climb a scale: chips-0 to chips-11.
  readonly property int chipNotes: 12
  property var effects: ({})

  function play(name) {
    var e = effects[name]
    if (!active || !e) return
    e.stop()
    e.play()
  }
  // The n-th chips step of a hand, a note higher each time.
  function chips(n) { play("chips-" + Math.min(n, chipNotes - 1)) }

  Component {
    id: effect
    SoundEffect {
      property real level: 1
      volume: root.volume * root.volume * level
      onStatusChanged: if (status === SoundEffect.Error) console.warn("omatro: can't load " + source)
    }
  }
  Component.onCompleted: {
    var names = Object.keys(levels)
    for (var i = 0; i < chipNotes; i++) names.push("chips-" + i)
    var made = {}
    for (var k = 0; k < names.length; k++) {
      var name = names[k]
      made[name] = effect.createObject(root, {
        source: Qt.resolvedUrl("../sounds/" + name + ".wav"),
        level: levels[name] !== undefined ? levels[name] : 0.5,
      })
    }
    effects = made
  }
}
