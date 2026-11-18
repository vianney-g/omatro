pragma Singleton
import QtQuick

// What every view may need of the game in progress without being handed it:
// the deck it is played on, whose sign the card backs show (CardBack). Main
// keeps it up to date.
QtObject {
  property string deck: ""
}
