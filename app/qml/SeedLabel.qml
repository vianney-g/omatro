import QtQuick
import omatro

// A game's seed, with a button that copies it to the clipboard (to share
// the game or replay it from the new game screen).
Row {
  id: root

  property string code: ""
  // The key that copies it too, shown on the button.
  property string shortcut: ""
  property int pixelSize: Style.font.body

  spacing: Style.spacing.md

  // Copies the code: through a hidden text field, as QML has no clipboard
  // of its own.
  function copy() {
    clip.text = root.code
    clip.selectAll()
    clip.copy()
    copied.restart()
  }

  TextEdit { id: clip; visible: false }
  Timer { id: copied; interval: 1500 }

  Text {
    anchors.verticalCenter: parent.verticalCenter
    text: I18n.tr("Seed")
    color: Colors.muted
    font.family: Style.font.family
    font.pixelSize: root.pixelSize
  }
  // Selectable as well, for a mouse copy.
  TextEdit {
    anchors.verticalCenter: parent.verticalCenter
    readOnly: true
    selectByMouse: true
    text: root.code
    color: Colors.foreground
    selectionColor: Colors.accent
    selectedTextColor: Colors.background
    font.family: Style.font.family
    font.pixelSize: root.pixelSize
    font.bold: true
  }
  Button {
    anchors.verticalCenter: parent.verticalCenter
    compact: true
    text: copied.running ? I18n.tr("Copied") : root.shortcut ? I18n.tr("Copy ({key})", { key: root.shortcut }) : I18n.tr("Copy")
    onClicked: root.copy()
  }
}
