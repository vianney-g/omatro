import QtQuick
import omatro

// Beside the hovered card, what the terms of its text mean (Hints.shown), in
// a bubble pointing at it (Bubble): on its right, or its left when there is
// no room; always in the window. Fill the window with it, over everything.
// A card shown in CardPeek has the terms in its own bubble instead.
Item {
  id: root

  readonly property Item target: Hints.target
  onTargetChanged: if (target) {
    var p = target.mapToItem(root, 0, 0)
    bubble.place = Qt.rect(p.x, p.y, target.width, target.height)
  }

  Bubble {
    id: bubble
    anchors.fill: parent
    shown: root.target !== null && Hints.shown.length > 0
    panelWidth: Style.space(220)
    TermList {
      width: parent.width
      terms: Hints.shown
    }
  }
}
