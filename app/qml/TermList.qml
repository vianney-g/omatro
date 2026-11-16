import QtQuick
import omatro

// The glossary entries of a bubble (Labels.mentions): each term's name, its
// category and what it means.
Column {
  id: root

  property var terms: []

  spacing: Style.space(8)
  Repeater {
    model: root.terms
    Column {
      width: root.width
      spacing: Style.space(2)
      Text {
        width: parent.width
        wrapMode: Text.WordWrap
        text: modelData.name
        color: Colors.accent
        font.family: Style.font.family
        font.pixelSize: Style.font.body
        font.bold: true
      }
      Text {
        width: parent.width
        text: modelData.category
        color: Colors.muted
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.italic: true
      }
      Text {
        width: parent.width
        wrapMode: Text.WordWrap
        text: modelData.description
        color: Colors.foreground
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
      }
    }
  }
}
