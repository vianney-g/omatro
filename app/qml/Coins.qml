import QtQuick
import omatro

// Coins flying from what pays to the money, in an arc, one after the
// other: `fly(from, to, amount)`, one coin per dollar (10 at most); `landed`
// fires as each one lands. Where they start is read at once, so the payer
// may go right after (a joker sold). Fill the window with it.
Item {
  id: root

  enabled: false  // never in the way of the pointer
  signal landed()

  function fly(from, to, amount) {
    if (!from || !to || amount <= 0) return
    var a = from.mapToItem(root, from.width / 2, from.height / 2)
    var b = to.mapToItem(root, to.width / 2, to.height / 2)
    var n = Math.max(1, Math.min(10, Math.round(amount)))
    for (var i = 0; i < n; i++)
      coin.createObject(root, { fromX: a.x, fromY: a.y, toX: b.x, toY: b.y, delay: i * 70 })
  }

  Component {
    id: coin
    Rectangle {
      id: c
      property real fromX
      property real fromY
      property real toX
      property real toY
      property int delay
      // 0 → 1 along the arc, which rises above the straight line.
      property real t: 0
      readonly property real lift: Math.min(Style.space(160), Math.abs(toX - fromX) * 0.4 + Style.space(40))
      readonly property real spread: (Math.random() - 0.5) * Style.space(30)
      width: Style.space(20); height: width
      radius: width / 2
      x: fromX + (toX - fromX) * t + spread * (1 - t) - width / 2
      y: fromY + (toY - fromY) * t - lift * 4 * t * (1 - t) - height / 2
      opacity: 0
      gradient: Gradient {
        GradientStop { position: 0; color: Qt.lighter(Colors.warm, 1.35) }
        GradientStop { position: 1; color: Qt.darker(Colors.warm, 1.25) }
      }
      border.width: Style.normalBorderWidth
      border.color: Qt.darker(Colors.warm, 1.6)
      Text {
        anchors.centerIn: parent
        text: "$"
        color: Qt.darker(Colors.warm, 2.2)
        font.family: Style.font.family
        font.pixelSize: Math.round(parent.width * 0.7)
        font.bold: true
      }
      // Spinning as it flies, like a coin seen edgewise.
      transform: Scale { origin.x: c.width / 2; xScale: Math.abs(Math.cos(c.t * Math.PI * 3)) * 0.8 + 0.2 }
      SequentialAnimation {
        running: true
        PauseAnimation { duration: c.delay }
        PropertyAction { target: c; property: "opacity"; value: 1 }
        NumberAnimation { target: c; property: "t"; to: 1; duration: 620; easing.type: Easing.InOutQuad }
        ScriptAction { script: root.landed() }
        NumberAnimation { target: c; property: "opacity"; to: 0; duration: 80 }
        ScriptAction { script: c.destroy() }
      }
    }
  }
}
