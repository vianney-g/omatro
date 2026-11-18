import QtQuick
import omatro

// A deck's sign, in the middle of its card backs: a shape as plain as
// Omarchy's logo, drawn on a 32 × 32 grid in the theme's colours (the
// accent, the urgent red where the colour is the subject, the theme's hues
// for Theme Hopper and Dual Boot). An unknown deck gets Omarchy's logo.
Item {
  id: root

  property string deck: ""
  property color accent: Colors.accent
  property color urgent: Colors.urgent
  property color green: Colors.green
  property color cyan: Colors.cyan
  property color magenta: Colors.magenta

  readonly property var known: ["think_pad", "framework", "basecamp", "raspberry_pi",
    "gaming_rig", "omakase_install", "theme_hopper", "board_member", "mac_refugee",
    "dual_boot", "power_user", "studio_display", "github_profile", "hyprland_dwindle",
    "live_usb", "raid1"]
  readonly property bool drawn: known.indexOf(deck) >= 0

  implicitWidth: 100
  implicitHeight: 100

  OmarchyLogo {
    anchors.centerIn: parent
    visible: !root.drawn
    width: parent.width
    height: implicitHeight
    color: root.accent
  }

  Canvas {
    id: canvas
    anchors.fill: parent
    visible: root.drawn
    // Redrawn when the deck, the size or a colour changes.
    readonly property var key: [root.deck, width, height, root.accent, root.urgent,
      root.green, root.cyan, root.magenta]
    onKeyChanged: requestPaint()

    onPaint: {
      var ctx = getContext("2d")
      ctx.reset()
      var s = Math.min(width, height) / 32
      ctx.translate((width - 32 * s) / 2, (height - 32 * s) / 2)
      ctx.scale(s, s)
      ctx.fillStyle = root.accent
      ctx.strokeStyle = root.accent
      ctx.lineWidth = 2.6
      ctx.lineCap = "square"
      ctx.lineJoin = "miter"

      function circle(x, y, r) { ctx.beginPath(); ctx.arc(x, y, r, 0, 2 * Math.PI); ctx.fill() }
      function poly(points) {
        ctx.beginPath()
        ctx.moveTo(points[0][0], points[0][1])
        for (var i = 1; i < points.length; i++) ctx.lineTo(points[i][0], points[i][1])
        ctx.closePath()
        ctx.fill()
      }
      function line(points) {
        ctx.beginPath()
        ctx.moveTo(points[0][0], points[0][1])
        for (var i = 1; i < points.length; i++) ctx.lineTo(points[i][0], points[i][1])
        ctx.stroke()
      }
      // Cut out of what is drawn.
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
          var d = i % 2 === 0 ? r : r * 0.42
          points.push([cx + d * Math.cos(a), cy + d * Math.sin(a)])
        }
        poly(points)
      }

      switch (root.deck) {
      case "think_pad":
        // The red nub.
        ctx.fillStyle = root.urgent
        circle(16, 16, 9)
        break
      case "framework":
        // A puzzle piece: knobs on top and right, a socket on the left.
        ctx.fillRect(7, 9, 17, 17)
        circle(15.5, 8, 4.2)
        circle(25, 17.5, 4.2)
        cut(function() { circle(7, 17.5, 4.2) })
        break
      case "basecamp":
        // A tent, its door open.
        poly([[16, 5], [29, 27], [3, 27]])
        cut(function() { poly([[16, 16], [21, 27], [11, 27]]) })
        break
      case "raspberry_pi":
        // A raspberry: drupelets in a cluster, two leaves.
        ctx.fillStyle = root.green
        ctx.beginPath(); ctx.ellipse(8.5, 6.5, 7.5, 4); ctx.fill()
        ctx.beginPath(); ctx.ellipse(16, 6.5, 7.5, 4); ctx.fill()
        ctx.fillStyle = root.urgent
        var drupes = [[12.5, 13], [19.5, 13], [9, 18.5], [16, 18.5], [23, 18.5], [12.5, 24], [19.5, 24], [16, 28.5]]
        for (var i = 0; i < drupes.length; i++) circle(drupes[i][0], drupes[i][1], 3.9)
        break
      case "gaming_rig":
        // A fan: its frame, three blades, the hub.
        ctx.beginPath(); ctx.arc(16, 16, 12.5, 0, 2 * Math.PI); ctx.stroke()
        for (var b = 0; b < 3; b++) {
          ctx.save()
          ctx.translate(16, 16)
          ctx.rotate(b * 2 * Math.PI / 3)
          ctx.beginPath()
          ctx.moveTo(0, -2)
          ctx.quadraticCurveTo(7, -9, 2, -10.5)
          ctx.quadraticCurveTo(-3, -6, 0, -2)
          ctx.fill()
          ctx.restore()
        }
        circle(16, 16, 3)
        break
      case "omakase_install":
        // Chopsticks.
        ctx.lineWidth = 2.8
        line([[7, 27], [21, 4]])
        line([[12, 28], [26, 5]])
        break
      case "theme_hopper":
        // Three swatches overlapping.
        ctx.globalAlpha = 0.85
        ctx.fillStyle = root.green; circle(11.5, 12.5, 7.5)
        ctx.fillStyle = root.cyan; circle(20.5, 12.5, 7.5)
        ctx.fillStyle = root.magenta; circle(16, 20.5, 7.5)
        ctx.globalAlpha = 1
        break
      case "board_member":
        // An empty chair, side on.
        ctx.lineWidth = 3
        line([[10, 5], [10, 27]])
        line([[10, 17], [23, 17], [23, 27]])
        break
      case "mac_refugee":
        // An apple core: its caps, the bitten sides, the stem.
        ctx.beginPath(); ctx.ellipse(8, 6, 16, 5); ctx.fill()
        ctx.beginPath(); ctx.ellipse(8, 23, 16, 5); ctx.fill()
        ctx.beginPath()
        ctx.moveTo(9.5, 8.5)
        ctx.quadraticCurveTo(15, 16.5, 9.5, 25.5)
        ctx.lineTo(22.5, 25.5)
        ctx.quadraticCurveTo(17, 16.5, 22.5, 8.5)
        ctx.closePath()
        ctx.fill()
        ctx.lineWidth = 2
        line([[16, 6], [18, 2.5]])
        break
      case "dual_boot":
        // Two halves of one disc.
        ctx.beginPath(); ctx.moveTo(15, 5); ctx.arc(15, 16, 11, -Math.PI / 2, Math.PI / 2, true); ctx.closePath(); ctx.fill()
        ctx.fillStyle = root.urgent
        ctx.beginPath(); ctx.moveTo(17, 5); ctx.arc(17, 16, 11, -Math.PI / 2, Math.PI / 2, false); ctx.closePath(); ctx.fill()
        break
      case "power_user":
        // A bolt.
        poly([[19, 2], [7, 18], [14.5, 18], [12, 30], [25, 13], [17.5, 13]])
        break
      case "studio_display":
        // A big screen, its stand, its one cable.
        ctx.lineWidth = 2.6
        ctx.strokeRect(4, 5, 24, 16)
        ctx.fillRect(14.5, 21, 3, 5)
        ctx.fillRect(10, 25.5, 12, 2.5)
        ctx.lineCap = "round"
        ctx.beginPath(); ctx.moveTo(19, 24); ctx.quadraticCurveTo(26, 25, 27, 30.5); ctx.stroke()
        break
      case "github_profile":
        // Twice the stars.
        star(9, 17, 8.8)
        star(23, 17, 8.8)
        break
      case "hyprland_dwindle":
        // Each window split in two, the second again: a spiral.
        ctx.lineWidth = 2.2
        ctx.strokeRect(3.5, 6, 25, 20)
        line([[16, 6], [16, 26]])
        line([[16, 16], [28.5, 16]])
        line([[22, 16], [22, 26]])
        break
      case "live_usb":
        // A die: five.
        ctx.lineWidth = 2.6
        ctx.strokeRect(5.5, 5.5, 21, 21)
        circle(11, 11, 2.2); circle(21, 11, 2.2); circle(16, 16, 2.2)
        circle(11, 21, 2.2); circle(21, 21, 2.2)
        break
      case "raid1":
        // Two identical disks either side of the mirror.
        ctx.fillRect(6, 6, 7, 20)
        ctx.fillRect(19, 6, 7, 20)
        for (var y = 3; y < 29; y += 3) ctx.fillRect(15.4, y, 1.2, 1.5)
        break
      }
    }
  }
}
