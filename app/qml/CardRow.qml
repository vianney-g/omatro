import QtQuick
import omatro

// A row of cards that never overlap: past the width it may take, every card
// narrows alike (delegates scale their view by `cardScale`), each keeping
// its own buttons under it, so nothing moves under the pointer. One card can
// be pinned with a click or a key, for keys that act on it; it shows an
// outline. Delegates place themselves with `xOf(index)`.
Item {
  id: row

  property int count: 0
  // Width of a card at full size, and the gap between cards.
  property real itemWidth: Style.space(120)
  property real spacing: Style.spacing.lg
  // The width the row may take.
  property real available: width

  readonly property real cardWidth: count > 0
    ? Math.min(itemWidth, (available - (count - 1) * spacing) / count) : itemWidth
  readonly property real cardScale: cardWidth / itemWidth
  // Narrowed cards are hard to read: hovered, they show at full size.
  readonly property bool narrow: cardScale < 0.99
  readonly property real step: cardWidth + spacing

  // The card pinned (-1: none).
  property int pinned: -1

  implicitWidth: count > 0 ? count * cardWidth + (count - 1) * spacing : 0

  function xOf(i) { return i * step }
  function pin(i) { pinned = pinned === i ? -1 : i }
  // Next card pinned (keyboard), then none after the last.
  function cycle() { pinned = pinned + 1 < count ? pinned + 1 : -1 }

  // Card i is about to leave the row (sold): once it has, the pinned card
  // stays the same one, or none if it was that one. The state may come later
  // (after cards change), hence the wait for the count to drop.
  property int leaving: -1
  property int lastCount: 0
  function removing(i) { leaving = i }
  onCountChanged: {
    if (leaving >= 0 && count < lastCount) {
      if (pinned === leaving) pinned = -1
      else if (pinned > leaving) pinned--
    }
    leaving = -1
    lastCount = count
    if (pinned >= count) pinned = -1
  }
}
