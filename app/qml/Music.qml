import QtQuick
import QtMultimedia

// The music: lofi tracks one after the other. Each track plays to its end;
// the next one depends on where the player is then: an instrumental during
// a blind (lyrics would get in the way of reading cards), a song elsewhere.
// Loaded by Main through a Loader: without Qt Multimedia, the game is silent.
Item {
  id: root

  // In a blind: instrumentals only.
  property bool calm: false
  // Set by Main once loaded: nothing plays before.
  property bool playing: false
  // 0 to 1, as the player sets it (heard on a squared scale).
  property real volume: 0.5
  signal started(string title)

  readonly property var tracks: [
    { file: "just-one-more-blind", title: "Just One More Blind", song: true },
    { file: "omakase-shuffle", title: "Omakase Shuffle", song: true },
    { file: "tiling-rain", title: "Tiling Rain", song: true },
    { file: "vingt-quatre-heures-trente-cinq-heures", title: "Vingt-quatre heures, trente-cinq heures", song: true },
    { file: "omatro-1", title: "Omatro 1", song: false },
    { file: "omatro-2", title: "Omatro 2", song: false },
    { file: "omatro-3", title: "Omatro 3", song: false },
    { file: "omatro-4", title: "Omatro 4", song: false },
  ]
  property int current: -1
  // Tracks that failed in a row: past all of them, the music stops.
  property int failures: 0

  function next() {
    var pool = []
    for (var i = 0; i < tracks.length; i++)
      if (tracks[i].song !== calm && i !== current) pool.push(i)
    if (!pool.length) pool = [current]
    current = pool[Math.floor(Math.random() * pool.length)]
    player.source = Qt.resolvedUrl("../music/" + tracks[current].file + ".opus")
    if (playing) player.play()
  }

  onPlayingChanged: {
    if (!playing) player.pause()
    else if (current < 0) next()
    else player.play()
  }

  MediaPlayer {
    id: player
    audioOutput: AudioOutput { volume: root.volume * root.volume }
    onMediaStatusChanged: {
      if (mediaStatus === MediaPlayer.EndOfMedia) root.next()
    }
    onPlaybackStateChanged: {
      if (playbackState === MediaPlayer.PlayingState && position < 1000) {
        root.failures = 0
        root.started(root.tracks[root.current].title)
      }
    }
    onErrorOccurred: function(error, message) {
      console.warn("omatro: can't play " + source + ": " + message)
      if (++root.failures < root.tracks.length) root.next()
    }
  }
}
