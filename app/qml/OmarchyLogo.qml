import QtQuick

// Omarchy's logo (its icon.txt, a row per two lines), drawn in a theme
// colour: the back of the cards.
Canvas {
  id: root
  property color color: Colors.accent
  readonly property var rows: [
    "######################################################",
    "####.....................####.....................####",
    "####....#####################.........########....####",
    "####....####..............................####....####",
    "####....####..............................####....####",
    "####....####..............................####....####",
    "############..............................####....####",
    "####....####..............................####....####",
    "####....####..............................####....####",
    "####....####..............................####....####",
    "####....######################################....####",
    "####.....................####.....................####",
    "#############################.....####################"
  ]
  // A character is half as wide as a line is tall.
  implicitWidth: 100
  implicitHeight: width * rows.length * 4 / rows[0].length
  onColorChanged: requestPaint()
  onWidthChanged: requestPaint()
  onHeightChanged: requestPaint()
  onPaint: {
    var ctx = getContext("2d")
    ctx.reset()
    ctx.fillStyle = root.color
    var w = width / rows[0].length
    var h = height / rows.length
    for (var y = 0; y < rows.length; y++) {
      var row = rows[y]
      var start = -1
      for (var x = 0; x <= row.length; x++) {
        var on = x < row.length && row[x] === "#"
        if (on && start < 0) start = x
        if (!on && start >= 0) {
          ctx.fillRect(start * w, y * h, (x - start) * w + 0.5, h + 0.5)
          start = -1
        }
      }
    }
  }
}
