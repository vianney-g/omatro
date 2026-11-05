import QtQuick
import omatro
import "Labels.js" as Labels

// The end of a lost game, as a kernel panic on the console: the lines come
// out one by one (the blind that wasn't beaten, the panic, the jokers held
// as a call trace), a cursor blinks, then the way out: a new game, the seed
// to copy. The kernel's own words stay in English, as on a real console.
Rectangle {
  id: root

  property var game: null
  // The blind that ended the game (its name).
  property string blindName: ""
  signal newGame()

  function copySeed() { seed.copy() }

  color: Qt.rgba(0, 0, 0, 0.94)

  MouseArea { anchors.fill: parent }

  readonly property color text: "#c8c8c8"
  readonly property color panic: Qt.lighter(Colors.urgent, 1.3)

  // "[ 1234.567890] ", the kernel's clock.
  property real clock: 0
  function stamp(i) {
    var t = (root.clock + i * 0.000173).toFixed(6)
    while (t.length < 12) t = " " + t
    return "[" + t + "] "
  }
  function hex(n) {
    var s = ""
    for (var i = 0; i < n; i++) s += "0123456789abcdef"[Math.floor(Math.random() * 16)]
    return s
  }

  // The lines: [text, is it the panic itself].
  property var lines: []
  function build() {
    var g = root.game
    if (!g) return
    root.clock = g.ante * 1000 + Math.random() * 999
    var over = I18n.tr("Game over")
    var out = [
      ["omatro: " + root.blindName + " (ante " + g.ante + "): "
        + Labels.number(g.score) + " / " + Labels.number(g.target), false],
      ["Kernel panic - not syncing: " + over, true],
      ["CPU: 0 PID: 1 Comm: omatro", false],
      ["Hardware name: " + (g.run_deck ? g.run_deck.name : "omatro") + " / " + g.stake.name, false],
      ["Call Trace:", false],
      [" <TASK>", false],
    ]
    g.jokers.forEach(function(j) {
      out.push(["  " + j.id + "+0x" + hex(2) + "/0x" + hex(3), false])
    })
    out.push(["  play_hand+0x" + hex(2) + "/0x" + hex(3), false])
    out.push([" </TASK>", false])
    out.push(["---[ end Kernel panic - not syncing: " + over + " ]---", true])
    root.lines = out
    root.shown = 0
    typing.start()
  }

  // How many lines are out.
  property int shown: 0
  Timer {
    id: typing
    interval: 70
    repeat: true
    onTriggered: if (++root.shown >= root.lines.length) stop()
  }
  onVisibleChanged: if (visible) build(); else typing.stop()

  Column {
    anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.spacing.huge }
    spacing: Style.spacing.xs

    Repeater {
      model: root.lines.slice(0, root.shown)
      Text {
        width: parent.width
        wrapMode: Text.WrapAnywhere
        text: root.stamp(index) + modelData[0]
        color: modelData[1] ? root.panic : root.text
        font.family: Style.font.family
        font.pixelSize: Style.font.body
        font.bold: modelData[1]
      }
    }
    // The cursor, blinking.
    Rectangle {
      width: Style.space(9); height: Style.font.body
      color: root.text
      SequentialAnimation on opacity {
        running: root.visible
        loops: Animation.Infinite
        PropertyAction { value: 1 }
        PauseAnimation { duration: 500 }
        PropertyAction { value: 0 }
        PauseAnimation { duration: 500 }
      }
    }

    // The way out, once the panic is written: in the theme's colours, as
    // its buttons are.
    Item { width: 1; height: Style.spacing.huge }
    Rectangle {
      width: wayOut.implicitWidth + 2 * Style.spacing.huge
      height: wayOut.implicitHeight + 2 * Style.spacing.xxl
      radius: Style.cornerRadius
      color: Colors.background
      border.width: Style.space(2)
      border.color: Colors.urgent
      opacity: root.shown >= root.lines.length ? 1 : 0
      Behavior on opacity { NumberAnimation { duration: 300 } }
      Column {
        id: wayOut
        anchors.centerIn: parent
        spacing: Style.spacing.xxl
        Text {
          text: I18n.tr("Have you tried rebooting?")
          color: Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.title
          font.italic: true
        }
        Row {
          spacing: Style.spacing.xxl
          Button {
            text: I18n.tr("New game (Enter)")
            bordered: true
            onClicked: root.newGame()
          }
          SeedLabel {
            id: seed
            anchors.verticalCenter: parent.verticalCenter
            code: root.game ? root.game.seed_code : ""
            shortcut: "Ctrl+C"
            pixelSize: Style.font.caption
          }
        }
      }
    }
  }
}
