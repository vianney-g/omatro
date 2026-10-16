pragma Singleton
import QtQuick
import "Labels.js" as Labels

// The card the pointer is on, and the terms its text mentions: the glossary
// panel shows them beside it, as Balatro does. Card views call `show` and
// `hide` as they are hovered.
QtObject {
  // From the engine's `glossary`, fetched once.
  property var terms: []
  property Item target: null
  property var shown: []

  function show(item, text, own) {
    var found = Labels.mentions(text, terms, own)
    shown = found
    target = found.length > 0 ? item : null
  }
  function hide(item) {
    if (target === item) {
      target = null
      shown = []
    }
  }
}
