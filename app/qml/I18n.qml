pragma Singleton
import QtQuick

// The interface's own text, in the language the engine was set to: `tr`
// takes the English text (the catalog's key) and fills its `{name}` slots.
// Bindings that call it follow a language change.
QtObject {
  // From the engine's `language` response: English text → translation.
  property var table: ({})
  property string lang: "en"
  // The languages to choose from: [{id, name}].
  property var languages: []

  function tr(text, args) {
    var out = table[text] || text
    if (args)
      for (var key in args)
        out = out.split("{" + key + "}").join(String(args[key]))
    return out
  }
}
