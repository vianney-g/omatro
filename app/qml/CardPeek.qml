import QtQuick
import omatro

// A card shown at full size over everything while its small form is hovered:
// a consumable of the clipboard list beside its line, a narrowed or drawn
// joker of the joker row under the row (below its buttons, which stay
// clickable), a drawn joker elsewhere beside it. Views ask through `Peek`.
// Its terms are explained beside it (Glossary). It takes no pointer, and no
// room.
Item {
  id: root

  Connections {
    target: Peek
    function onJokerShown(item, joker, under, hidden) { root.showJoker(item, joker, under, hidden) }
    function onConsumableShown(item, consumable, note) { root.showConsumable(item, consumable, note) }
    function onHidden(item) { root.hide(item) }
  }

  // The small form hovered, and what it shows.
  property Item source: null
  property var consumable: null
  property var joker: null
  property bool hidden: false
  // Under this item, for a joker of the row (with its buttons); null, beside
  // the joker.
  property Item under: null
  // What the consumable needs, said under it ("select 1-2 cards").
  property string note: ""

  readonly property real gap: Style.spacing.lg

  function showConsumable(item, c, note) {
    joker = null
    consumable = c
    root.note = note
    place(item)
    Hints.show(consumableView, c.description, c.name)
  }
  function showJoker(item, j, under, hidden) {
    consumable = null
    joker = j
    root.hidden = hidden
    root.under = under
    note = ""
    place(item)
    if (!hidden) Hints.show(jokerView, j.description + (j.stickers || []).map(function(s) { return " " + s.name }).join(""), j.name)
  }
  function hide(item) {
    if (source !== item) return
    Hints.hide(consumable ? consumableView : jokerView)
    source = null
    consumable = null
    joker = null
  }
  // Placed now, before the glossary reads where it is (it reads it when its
  // target changes: hence the hide first, the view staying the same).
  function place(item) {
    Hints.hide(consumableView)
    Hints.hide(jokerView)
    source = item
    var p = item.mapToItem(root, 0, 0)
    if (consumable || !under) {
      // Beside it: on its right, or on its left when there is no room.
      var w = consumable ? consumableView.width : jokerView.width
      var h = consumable ? card.height : jokerView.height
      card.x = p.x + item.width + gap + w <= root.width || consumable ? p.x + item.width + gap : Math.max(0, p.x - gap - w)
      card.y = Math.max(0, Math.min(p.y + item.height / 2 - h / 2, root.height - h))
    } else {
      var u = under.mapToItem(root, 0, under.height)
      card.x = Math.max(0, Math.min(p.x + item.width / 2 - card.width / 2, root.width - card.width))
      card.y = Math.min(u.y + gap, root.height - card.height)
    }
  }

  Column {
    id: card
    visible: root.source !== null
    enabled: false
    spacing: Style.spacing.sm
    ConsumableView {
      id: consumableView
      full: true
      visible: root.consumable !== null
      item: root.consumable
    }
    Text {
      visible: root.consumable !== null && root.note !== ""
      width: consumableView.width
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.note
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
    JokerView {
      id: jokerView
      visible: root.joker !== null
      joker: root.joker
      hidden: root.hidden
    }
  }
}
