import QtQuick
import omatro

// A row of cards that overlap when they no longer fit side by side, like a
// hand of cards: each keeps its full width, the left edge of each stays in
// view, and the card in front (hovered, flashing, or pinned with a click or
// a key) shows whole, lifted. Delegates place themselves with `xOf(index)`,
// `zOf(index)` and `liftOf(index)`, and report hovering with `hover`.
Item {
  id: fan

  property int count: 0
  // Width of a card, and the gap between cards when they fit.
  property real itemWidth: Style.space(120)
  property real spacing: Style.spacing.lg
  // The width the row may take.
  property real available: width

  // The card hovered, the one a scoring step flashes, the one pinned (-1: none).
  property int hovered: -1
  property int flashing: -1
  property int pinned: -1
  // Without any of them, the last card, which lies on top anyway: its
  // controls show.
  readonly property int chosen: hovered >= 0 ? hovered : flashing >= 0 ? flashing : pinned
  readonly property int front: chosen >= 0 ? chosen : count - 1

  readonly property bool stacked: count > 1 && count * itemWidth + (count - 1) * spacing > available
  readonly property real step: stacked ? Math.max(Style.space(12), (available - itemWidth) / (count - 1))
    : itemWidth + spacing

  implicitWidth: count > 0 ? itemWidth + (count - 1) * step : 0
  implicitHeight: {
    var h = 0
    for (var i = 0; i < children.length; i++)
      h = Math.max(h, children[i].implicitHeight)
    return h
  }

  function xOf(i) { return i * step }
  // Later cards lie on earlier ones; the front card on all of them.
  function zOf(i) { return i === front ? count + 1 : i }
  // Only a card brought forward lifts, not the last one lying on top; a
  // pinned card always does, to show which one keys move.
  function liftOf(i) { return (stacked && i === chosen) || i === pinned ? -Style.space(8) : 0 }
  // Whether the card's own buttons show: always side by side, only the front
  // card's when stacked (they would overlap).
  function showsControls(i) { return !stacked || i === front }

  function hover(i, on) {
    if (on) hovered = i
    else if (hovered === i) hovered = -1
  }
  function pin(i) { pinned = pinned === i ? -1 : i }
  // Next card in front (keyboard), then none after the last.
  function cycle() { pinned = pinned + 1 < count ? pinned + 1 : -1 }
  function flash(i) {
    flashing = i
    flashTimer.restart()
  }
  onCountChanged: if (pinned >= count) pinned = -1

  Timer { id: flashTimer; interval: 450; onTriggered: fan.flashing = -1 }
}
