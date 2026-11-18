import QtQuick
import omatro

// A badge's icon, at the left of its pill as on a README badge: a sign as
// plain as the decks' (DeckMark), drawn on a 16 × 16 grid in one colour, so
// that it reads at the size of the text. Nothing for a badge without one.
Canvas {
  id: root

  property string tag: ""
  property color color: Colors.foreground

  readonly property var known: ["stars100", "stars10k", "opacity0", "blur_on", "shadow_on",
    "animations_on", "series_a", "prs_welcome", "wontfix", "build_passing", "docs_latest",
    "theme_dark", "fun_yes", "funding_oligarchy", "commits1k", "gc_done", "price_free",
    "badges2x", "resolution4k", "cache_hit", "contributors2", "speed_blazing", "version3",
    "sponsors2x"]
  readonly property bool drawn: known.indexOf(tag) >= 0

  implicitWidth: 16
  implicitHeight: 16
  visible: drawn

  // Redrawn when the badge, the size or the colour changes.
  readonly property var key: [tag, width, height, color]
  onKeyChanged: requestPaint()

  onPaint: {
    var ctx = getContext("2d")
    ctx.reset()
    var s = Math.min(width, height) / 16
    ctx.translate((width - 16 * s) / 2, (height - 16 * s) / 2)
    ctx.scale(s, s)
    ctx.fillStyle = root.color
    ctx.strokeStyle = root.color
    ctx.lineWidth = 1.6
    ctx.lineCap = "round"
    ctx.lineJoin = "round"

    function circle(x, y, r) { ctx.beginPath(); ctx.arc(x, y, r, 0, 2 * Math.PI); ctx.fill() }
    function ring(x, y, r) { ctx.beginPath(); ctx.arc(x, y, r, 0, 2 * Math.PI); ctx.stroke() }
    function poly(points, stroke) {
      ctx.beginPath()
      ctx.moveTo(points[0][0], points[0][1])
      for (var i = 1; i < points.length; i++) ctx.lineTo(points[i][0], points[i][1])
      if (stroke) ctx.stroke()
      else { ctx.closePath(); ctx.fill() }
    }
    function cut(draw) {
      ctx.save()
      ctx.globalCompositeOperation = "destination-out"
      draw()
      ctx.restore()
    }
    function star(cx, cy, r) {
      var points = []
      for (var i = 0; i < 10; i++) {
        var a = -Math.PI / 2 + i * Math.PI / 5
        var d = i % 2 === 0 ? r : r * 0.45
        points.push([cx + d * Math.cos(a), cy + d * Math.sin(a)])
      }
      poly(points)
    }
    function heart(cx, cy, r) {
      ctx.beginPath()
      ctx.moveTo(cx, cy + r)
      ctx.bezierCurveTo(cx - 2.2 * r, cy - 0.2 * r, cx - 0.9 * r, cy - 1.9 * r, cx, cy - 0.6 * r)
      ctx.bezierCurveTo(cx + 0.9 * r, cy - 1.9 * r, cx + 2.2 * r, cy - 0.2 * r, cx, cy + r)
      ctx.fill()
    }

    switch (root.tag) {
    case "stars100":
      star(8, 8.5, 7.5)
      break
    case "stars10k":
      star(5, 9, 4.8)
      star(11.5, 7, 4.8)
      break
    case "opacity0":
      // A circle you can hardly see.
      for (var i = 0; i < 8; i++) {
        ctx.beginPath()
        ctx.arc(8, 8, 6, i * Math.PI / 4, i * Math.PI / 4 + Math.PI / 8)
        ctx.stroke()
      }
      break
    case "blur_on":
      // A drop.
      ctx.beginPath()
      ctx.moveTo(8, 1.5)
      ctx.bezierCurveTo(8, 1.5, 3, 7.5, 3, 10)
      ctx.arc(8, 10, 5, Math.PI, 0, true)
      ctx.bezierCurveTo(13, 7.5, 8, 1.5, 8, 1.5)
      ctx.fill()
      break
    case "shadow_on":
      // A square and its shadow, offset.
      ctx.globalAlpha = 0.45
      ctx.fillRect(5, 5, 9, 9)
      ctx.globalAlpha = 1
      ctx.fillRect(2, 2, 9, 9)
      break
    case "animations_on":
      poly([[4, 2], [14, 8], [4, 14]])
      break
    case "series_a":
      // A stack of coins, a gap between each.
      for (var c = 0; c < 3; c++) {
        ctx.beginPath(); ctx.ellipse(2, 10.5 - c * 4, 12, 4.5); ctx.fill()
        if (c < 2) cut(function() { ctx.fillRect(0, 10.6 - c * 4, 16, 0.9) })
      }
      break
    case "prs_welcome":
      // A pull request: a branch joining the main line.
      circle(4, 3.5, 2)
      circle(4, 12.5, 2)
      circle(12, 12.5, 2)
      poly([[4, 4], [4, 12]], true)
      poly([[12, 11], [12, 7], [10, 5], [7, 5]], true)
      poly([[8, 3], [6, 5], [8, 7]], true)
      break
    case "wontfix":
      ctx.lineWidth = 1.8
      ring(8, 8, 6)
      poly([[3.8, 12.2], [12.2, 3.8]], true)
      break
    case "build_passing":
      ctx.lineWidth = 2.4
      poly([[2.5, 8.5], [6.5, 12.5], [13.5, 3.5]], true)
      break
    case "docs_latest":
      // An open book.
      poly([[1, 3.5], [7.3, 5], [7.3, 14], [1, 12.5]])
      poly([[15, 3.5], [8.7, 5], [8.7, 14], [15, 12.5]])
      break
    case "theme_dark":
      circle(8, 8, 7)
      cut(function() { circle(11.5, 5.5, 6) })
      break
    case "fun_yes":
      circle(8, 8, 7)
      cut(function() {
        circle(5.5, 6, 1.2)
        circle(10.5, 6, 1.2)
        ctx.lineWidth = 1.4
        ctx.beginPath(); ctx.arc(8, 8.5, 3.5, 0.2 * Math.PI, 0.8 * Math.PI); ctx.stroke()
      })
      break
    case "funding_oligarchy":
      // A crown.
      poly([[1.5, 13], [1.5, 4.5], [5, 8], [8, 3], [11, 8], [14.5, 4.5], [14.5, 13]])
      break
    case "commits1k":
      // A commit on its line.
      poly([[0.5, 8], [15.5, 8]], true)
      circle(8, 8, 3.8)
      cut(function() { circle(8, 8, 1.8) })
      break
    case "gc_done":
      // A bin.
      ctx.fillRect(2, 3, 12, 2)
      ctx.fillRect(6, 1, 4, 2)
      poly([[3, 6], [13, 6], [12, 15], [4, 15]])
      break
    case "price_free":
      // A price tag.
      poly([[1.5, 8], [7, 2.5], [14.5, 2.5], [14.5, 13.5], [7, 13.5]])
      cut(function() { circle(11, 8, 1.6) })
      break
    case "badges2x":
      // Two pills, one over the other.
      ctx.globalAlpha = 0.5
      ctx.fillRect(4, 3, 11, 6)
      ctx.globalAlpha = 1
      ctx.fillRect(1, 7, 11, 6)
      break
    case "resolution4k":
      ctx.fillRect(1.5, 1.5, 6, 6)
      ctx.fillRect(8.5, 1.5, 6, 6)
      ctx.fillRect(1.5, 8.5, 6, 6)
      ctx.fillRect(8.5, 8.5, 6, 6)
      break
    case "cache_hit":
      // A target.
      ctx.lineWidth = 1.5
      ring(8, 8, 6.5)
      ring(8, 8, 3.5)
      circle(8, 8, 1.4)
      break
    case "contributors2":
      // Two people.
      circle(5, 5, 2.6)
      ctx.beginPath(); ctx.arc(5, 14, 4.5, Math.PI, 0); ctx.fill()
      circle(11.5, 4.5, 2.6)
      ctx.beginPath(); ctx.arc(11.5, 13.5, 4.5, Math.PI, 0); ctx.fill()
      break
    case "speed_blazing":
      // A flame.
      ctx.beginPath()
      ctx.moveTo(8, 1)
      ctx.bezierCurveTo(11, 5, 14, 7, 13, 11)
      ctx.bezierCurveTo(12, 15, 4, 15, 3, 11)
      ctx.bezierCurveTo(2.5, 8, 5, 7, 5.5, 4.5)
      ctx.bezierCurveTo(7, 6.5, 7.5, 7, 8, 1)
      ctx.fill()
      break
    case "version3":
      // Three releases, up.
      ctx.lineWidth = 1.9
      poly([[3, 6], [8, 2], [13, 6]], true)
      poly([[3, 10], [8, 6], [13, 10]], true)
      poly([[3, 14], [8, 10], [13, 14]], true)
      break
    case "sponsors2x":
      heart(8, 8.5, 5.5)
      break
    }
  }
}
