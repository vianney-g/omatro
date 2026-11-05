import QtQuick
import omatro

// A number shown as a mechanical counter: each digit is a drum of 0 to 9
// that rolls to its value, so a growing number makes its digits spin (the
// last ones fastest). Separators (",", narrow spaces) stand still. Drums are
// kept by place from the right, so that a digit keeps rolling as the number
// grows longer.
Item {
  id: root

  property string text: ""
  property color color: Colors.foreground
  property int pixelSize: Style.font.display
  property bool bold: true

  implicitWidth: row.implicitWidth
  implicitHeight: probe.implicitHeight

  // The height of a line, for the drums' windows.
  Text {
    id: probe
    visible: false
    text: "0"
    font.family: Style.font.family
    font.pixelSize: root.pixelSize
    font.bold: root.bold
  }

  // The characters from the right.
  readonly property var places: text.split("").reverse()

  Row {
    id: row
    anchors.right: parent.right
    layoutDirection: Qt.RightToLeft
    Repeater {
      model: root.places.length
      Item {
        id: place
        required property int index
        readonly property string glyph: root.places[index] || ""
        readonly property bool digit: glyph >= "0" && glyph <= "9"
        width: digit ? probe.implicitWidth : still.implicitWidth
        height: probe.implicitHeight
        clip: true

        // A separator.
        Text {
          id: still
          visible: !place.digit
          text: place.digit ? "" : place.glyph
          color: root.color
          font.family: Style.font.family
          font.pixelSize: root.pixelSize
          font.bold: root.bold
        }

        // A digit: the drum, 0 to 9, rolled to it.
        Column {
          visible: place.digit
          y: place.digit ? -Number(place.glyph) * probe.implicitHeight : 0
          Behavior on y { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
          Repeater {
            model: 10
            Text {
              required property int index
              text: index
              height: probe.implicitHeight
              color: root.color
              font.family: Style.font.family
              font.pixelSize: root.pixelSize
              font.bold: root.bold
            }
          }
        }
      }
    }
  }
}
