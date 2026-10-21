import QtQuick
import omatro

// Settings (O, from anywhere): language, music and sound volumes; then every
// key of the game.
Rectangle {
  id: root

  property bool music: true
  property real musicVolume: 0.5
  property bool sounds: true
  property real soundVolume: 0.5
  signal close()
  signal language(string lang)
  signal toggleMusic()
  signal musicVolumeSet(real volume)
  signal toggleSounds()
  signal soundVolumeSet(real volume)

  color: Util.alpha(Colors.background, 0.92)

  // The keys, by moment of the game: [keys, what they do].
  readonly property var keys: [
    { title: "In a blind", keys: [
      ["1-9, 0", "choose cards"], ["Enter", "play the hand"], ["D", "discard"],
      ["Tab", "sort by rank or suit"], ["Shift+←/→", "move the selected card"] ] },
    { title: "Before a blind", keys: [
      ["Enter", "play it"], ["S", "skip it, for its badge"], ["B", "reroll the boss"] ] },
    { title: "In the shop", keys: [
      ["1-9", "buy: offers, packs, then pull requests"], ["Shift+1-9", "buy and use at once"],
      ["R", "reroll"], ["Enter", "next blind"] ] },
    { title: "In a pack", keys: [
      ["1-5", "install"], ["Shift+1-5", "install, with cards selected"], ["Ctrl+C, Esc", "skip the pack"] ] },
    { title: "Anywhere", keys: [
      ["J, Shift+←/→", "bring a joker forward, move it"], ["Ctrl+1, Ctrl+2", "apply a consumable"],
      ["I", "fastfetch"], ["C", "collection"], ["Ctrl+N", "new game"], ["O", "settings"], ["Ctrl+Q", "quit"] ] },
  ]

  // Key names as the keyboard reads in the language (Maj, Entrée…).
  function keyName(keys) {
    return keys.replace("Shift", I18n.tr("Shift")).replace("Enter", I18n.tr("Enter")).replace("Esc", I18n.tr("Esc"))
  }

  MouseArea { anchors.fill: parent; onClicked: root.close() }

  component Label: Text {
    color: Colors.foreground
    font.family: Style.font.family
    font.pixelSize: Style.font.body
  }

  component Heading: Text {
    color: Colors.accent
    font.family: Style.font.family
    font.pixelSize: Style.font.heading
    font.bold: true
  }

  component KeySection: Column {
    property var section
    spacing: Style.spacing.sm
    Label { text: I18n.tr(section.title); color: Colors.muted }
    Repeater {
      model: section.keys
      Row {
        Label { width: Style.space(150); text: root.keyName(modelData[0]); font.bold: true }
        Label { text: I18n.tr(modelData[1]) }
      }
    }
  }

  Rectangle {
    anchors.centerIn: parent
    width: content.implicitWidth + 2 * Style.spacing.huge
    height: content.implicitHeight + 2 * Style.spacing.huge
    radius: Style.cornerRadius
    color: Colors.background
    border.width: Style.normalBorderWidth
    border.color: Colors.accent

    // Clicks inside the panel do not close it.
    MouseArea { anchors.fill: parent }

    Column {
      id: content
      anchors { left: parent.left; top: parent.top; margins: Style.spacing.huge }
      spacing: Style.spacing.xxl

      Row {
        spacing: Style.spacing.xxl
        Text {
          text: I18n.tr("Settings")
          color: Colors.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.displayLarge
          font.bold: true
        }
        Text {
          anchors.baseline: parent.children[0].baseline
          text: I18n.tr("O or Esc: close")
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
          font.italic: true
        }
      }

      Grid {
        columns: 2
        columnSpacing: Style.spacing.xxl
        rowSpacing: Style.spacing.lg
        verticalItemAlignment: Grid.AlignVCenter

        Label { text: I18n.tr("Language (L)"); color: Colors.muted }
        Row {
          spacing: Style.spacing.md
          Repeater {
            model: I18n.languages
            Button {
              compact: true
              text: modelData.name
              bordered: modelData.id === I18n.lang
              onClicked: root.language(modelData.id)
            }
          }
        }
        Label { text: I18n.tr("Music (M, ←/→)"); color: Colors.muted }
        VolumeControl {
          active: root.music
          volume: root.musicVolume
          onToggled: root.toggleMusic()
          onVolumeChosen: function(v) { root.musicVolumeSet(v) }
        }
        Label { text: I18n.tr("Sounds (Shift+M, Shift+←/→)"); color: Colors.muted }
        VolumeControl {
          active: root.sounds
          volume: root.soundVolume
          onToggled: root.toggleSounds()
          onVolumeChosen: function(v) { root.soundVolumeSet(v) }
        }
      }

      Heading { text: I18n.tr("Keys") }
      Row {
        spacing: Style.space(40)
        Column {
          spacing: Style.spacing.xl
          KeySection { section: root.keys[0] }
          KeySection { section: root.keys[1] }
          KeySection { section: root.keys[3] }
        }
        Column {
          spacing: Style.spacing.xl
          KeySection { section: root.keys[2] }
          KeySection { section: root.keys[4] }
        }
      }
    }
  }
}
