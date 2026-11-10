import QtQuick
import omatro

// A row of cards that overlap when they no longer fit side by side, like a
// hand of cards: each keeps its full width, the left edge of each stays in
// view, and the card in front (hovered, flashing, or pinned with a click or
// a key) shows whole. Delegates place themselves with `xOf(index)`,
// `zOf(index)` and `liftOf(index)`, and report hovering with `hover`.
// Side by side, each card shows its own buttons (`showsControls`); stacked,
// they would overlap, so one action bar under the row acts on `target`: the
// pinned card, or the last one. Hovering never changes it, so the pointer
// can cross other cards on its way to the bar.
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
  // The card the action bar acts on, lifted, when stacked.
  readonly property int target: pinned >= 0 ? pinned : count - 1

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
  // A pinned card lifts, to show which one keys move; stacked, the action
  // bar's target does. A hovered card only comes forward, without moving
  // under the pointer.
  function liftOf(i) { return (stacked ? i === target : i === pinned) ? -Style.space(8) : 0 }
  // Whether the card's own buttons show: only side by side; stacked, the
  // action bar has them.
  function showsControls(i) { return !stacked }

  function hover(i, on) {
    if (on) hovered = i
    else if (hovered === i) hovered = -1
  }
  function pin(i) { pinned = pinned === i ? -1 : i }
  // Next card in front (keyboard), then none after the last.
  function cycle() { pinned = pinned + 1 < count ? pinned + 1 : -1 }
  // Card i left the row (sold, used): the pinned card stays the same one,
  // and is no longer pinned if it was that one.
  function removed(i) {
    if (pinned === i) pinned = -1
    else if (pinned > i) pinned--
  }
  function flash(i) {
    flashing = i
    flashTimer.restart()
  }
  onCountChanged: if (pinned >= count) pinned = -1

  Timer { id: flashTimer; interval: 450; onTriggered: fan.flashing = -1 }
}
