pragma Singleton
import QtQuick

// Asks for a card at full size while its small form is hovered: CardPeek,
// over everything in Main, shows it. A drawn joker anywhere (shop, packs,
// the resume screen, the collection) calls `showJoker` from its own hover.
QtObject {
  signal jokerShown(Item item, var joker, Item under, bool hidden)
  signal consumableShown(Item item, var consumable, string note)
  signal hidden(Item item)

  // `under`: shown under that item (the joker row); null, beside `item`.
  function showJoker(item, joker, under, hidden) { jokerShown(item, joker, under, hidden) }
  function showConsumable(item, consumable, note) { consumableShown(item, consumable, note) }
  function hide(item) { hidden(item) }
}
