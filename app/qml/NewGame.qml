import QtQuick
import omatro
import "Labels.js" as Labels

// The new game screen: pick the machine to play on (arrows, Enter), or go back
// to the game in progress (Escape).
Rectangle {
  id: root

  property var decks: []
  property int current: 0
  property bool canCancel: false
  // A problem to report (a saved game that could not be resumed).
  property string message: ""
  signal start(string deck)
  signal cancel()
  signal language(string lang)

  color: Util.alpha(Color.background, 0.96)

  function move(step) {
    if (decks.length > 0) current = (current + step + decks.length) % decks.length
  }

  MouseArea { anchors.fill: parent }

  Column {
    anchors.centerIn: parent
    spacing: Style.spacing.huge

    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: I18n.tr("New game")
      color: Color.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.displayLarge
      font.bold: true
    }
    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: I18n.tr("Pick your machine · arrows to choose · Enter to start") + (root.canCancel ? I18n.tr(" · Escape to go back") : "")
      color: Color.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }
    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      visible: root.message !== ""
      text: root.message
      color: Color.urgent
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }

    Grid {
      anchors.horizontalCenter: parent.horizontalCenter
      columns: 3
      spacing: Style.spacing.lg
      Repeater {
        model: root.decks
        Rectangle {
          readonly property bool chosen: index === root.current
          width: Style.space(285)
          height: Style.space(88)
          radius: Style.cornerRadius
          color: chosen ? Qt.tint(Color.background, Util.alpha(Color.accent, 0.15)) : Style.normalFill
          border.width: chosen ? Style.space(2) : Style.normalBorderWidth
          border.color: chosen ? Color.accent : Style.normalBorderColor
          Column {
            anchors { fill: parent; margins: Style.space(8) }
            spacing: Style.space(3)
            Text {
              text: modelData.name
              color: chosen ? Color.accent : Color.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.subtitle
              font.bold: true
            }
            Text {
              width: parent.width
              wrapMode: Text.WordWrap
              maximumLineCount: 2
              elide: Text.ElideRight
              textFormat: Text.StyledText
              text: Labels.highlight(modelData.description, Hints.terms, modelData.name)
              color: Color.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
            }
            Text {
              width: parent.width
              elide: Text.ElideRight
              text: modelData.quip
              color: Color.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
              font.italic: true
            }
          }
          MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.current === index ? root.start(modelData.id) : root.current = index
            // Its terms explained beside it (Glossary).
            onContainsMouseChanged: containsMouse
              ? Hints.show(parent, modelData.description, modelData.name) : Hints.hide(parent)
          }
        }
      }
    }

    // The language of the game (L switches).
    Row {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.md
      Text {
        anchors.verticalCenter: parent.verticalCenter
        text: I18n.tr("Language (L)")
        color: Color.muted
        font.family: Style.font.family
        font.pixelSize: Style.font.body
      }
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

    Row {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.xxl
      Button {
        text: I18n.tr("Start (Enter)")
        enabled: root.decks.length > 0
        onClicked: root.start(root.decks[root.current].id)
      }
      Button {
        visible: root.canCancel
        text: I18n.tr("Back (Escape)")
        onClicked: root.cancel()
      }
    }
  }
}
