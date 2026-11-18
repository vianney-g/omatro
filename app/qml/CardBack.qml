import QtQuick
import omatro

// The back of a card: its deck's sign (DeckMark; Omarchy's logo for a deck
// without one) in a double frame. The deck is the game's (Run) unless given.
Rectangle {
  property string deck: Run.deck

  radius: Style.cornerRadius
  color: Qt.tint(Colors.background, Util.alpha(Colors.accent, 0.25))
  border.width: Style.space(2)
  border.color: Util.alpha(Colors.accent, 0.6)
  Rectangle {
    anchors { fill: parent; margins: Style.space(6) }
    radius: Style.cornerRadius
    color: "transparent"
    border.width: Style.normalBorderWidth
    border.color: Util.alpha(Colors.accent, 0.5)
  }
  DeckMark {
    anchors.centerIn: parent
    width: Math.round(parent.width * 0.62)
    height: width
    deck: parent.deck
  }
}
