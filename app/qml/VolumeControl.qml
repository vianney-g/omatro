import QtQuick
import omatro

// On or off (the middle button), and the volume by tenths (− and +).
Row {
  id: control

  property bool active
  property real volume
  signal toggled()
  signal volumeChosen(real volume)

  spacing: Style.spacing.md

  Button {
    compact: true
    text: "−"
    enabled: control.active && control.volume > 0
    onClicked: control.volumeChosen(control.volume - 0.1)
  }
  Button {
    compact: true
    width: Style.space(70)
    text: control.active ? Math.round(control.volume * 100) + (I18n.lang === "fr" ? " %" : "%") : I18n.tr("Off")
    bordered: control.active
    onClicked: control.toggled()
  }
  Button {
    compact: true
    text: "+"
    enabled: !control.active || control.volume < 1
    onClicked: control.volumeChosen(control.volume + 0.1)
  }
}
