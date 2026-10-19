import QtQuick
import omatro
import "Labels.js" as Labels

// The new game screen: pick the machine to play on (arrows, Enter), or go back
// to the game in progress (Escape).
Rectangle {
  id: root

  property var decks: []
  property int current: 0
  // The difficulty levels, and the one chosen (Shift+←/→), within what the
  // chosen deck has unlocked.
  property var stakes: []
  property int stake: 0
  readonly property var deck: decks.length ? decks[current] : null
  readonly property int topStake: deck ? stakeIndex(deck.top_stake) : 0
  readonly property int chosenStake: Math.min(stake, topStake)
  property bool canCancel: false
  // A problem to report (a saved game that could not be resumed).
  property string message: ""
  signal start(string deck, string stake)
  signal cancel()
  signal language(string lang)
  signal collection()

  color: Util.alpha(Colors.background, 0.96)

  function move(step) {
    if (decks.length > 0) current = (current + step + decks.length) % decks.length
  }
  function stakeIndex(id) {
    var i = stakes.findIndex(function(s) { return s.id === id })
    return Math.max(0, i)
  }
  function moveStake(step) {
    stake = Math.max(0, Math.min(topStake, chosenStake + step))
  }
  function startChosen() {
    if (deck && !deck.achievement && stakes.length) start(deck.id, stakes[chosenStake].id)
  }

  MouseArea { anchors.fill: parent }

  Column {
    anchors.centerIn: parent
    spacing: Style.spacing.huge

    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: I18n.tr("New game")
      color: Colors.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.displayLarge
      font.bold: true
    }
    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: I18n.tr("Pick your machine · arrows to choose · Enter to start") + (root.canCancel ? I18n.tr(" · Escape to go back") : "")
        + I18n.tr(" · Shift+←/→: level · C: collection")
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }
    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      visible: root.message !== ""
      text: root.message
      color: Colors.urgent
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
          // A hidden deck, until its achievement.
          readonly property bool locked: !!modelData.achievement
          width: Style.space(285)
          height: Style.space(88)
          radius: Style.cornerRadius
          color: chosen ? Qt.tint(Colors.background, Util.alpha(Colors.accent, 0.15)) : Style.normalFill
          border.width: chosen ? Style.space(2) : Style.normalBorderWidth
          border.color: chosen ? Colors.accent : Style.normalBorderColor
          Column {
            visible: locked
            anchors { fill: parent; margins: Style.space(8) }
            spacing: Style.space(3)
            Text {
              text: "?????"
              color: Colors.urgent
              font.family: Style.font.family
              font.pixelSize: Style.font.subtitle
              font.bold: true
            }
            Text {
              width: parent.width
              wrapMode: Text.WordWrap
              maximumLineCount: 2
              elide: Text.ElideRight
              text: locked ? I18n.tr("Hidden: {achievement}", { achievement: modelData.achievement })
                + " · " + I18n.tr(modelData.condition) : ""
              color: Colors.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
            }
          }
          Column {
            visible: !locked
            anchors { fill: parent; margins: Style.space(8) }
            spacing: Style.space(3)
            Row {
              width: parent.width
              Text {
                width: parent.width - bestText.width
                elide: Text.ElideRight
                text: modelData.name
                color: chosen ? Colors.accent : Colors.foreground
                font.family: Style.font.family
                font.pixelSize: Style.font.subtitle
                font.bold: true
              }
              // The highest level won with it.
              Text {
                id: bestText
                anchors.baseline: parent.children[0].baseline
                text: modelData.best_stake ? root.stakes[root.stakeIndex(modelData.best_stake)].name : ""
                color: Colors.warm
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                font.bold: true
              }
            }
            Text {
              width: parent.width
              wrapMode: Text.WordWrap
              maximumLineCount: 2
              elide: Text.ElideRight
              textFormat: Text.StyledText
              text: Labels.highlight(modelData.description, Hints.terms, modelData.name)
              color: Colors.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
            }
            Text {
              width: parent.width
              elide: Text.ElideRight
              text: modelData.quip
              color: Colors.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
              font.italic: true
            }
          }
          MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.current === index && !parent.locked ? root.startChosen() : root.current = index
            // Its terms explained beside it (Glossary).
            onContainsMouseChanged: containsMouse && !parent.locked
              ? Hints.show(parent, modelData.description, modelData.name) : Hints.hide(parent)
          }
        }
      }
    }

    // The difficulty level: those the deck unlocked, then locked ones.
    Column {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.md
      visible: root.stakes.length > 0
      Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Style.spacing.sm
        Repeater {
          model: root.stakes
          Rectangle {
            readonly property bool chosen: index === root.chosenStake
            readonly property bool open: index <= root.topStake
            width: stakeLabel.implicitWidth + 2 * Style.spacing.lg
            height: stakeLabel.implicitHeight + 2 * Style.spacing.sm
            radius: Style.cornerRadius
            color: chosen ? Qt.tint(Colors.background, Util.alpha(Colors.warm, 0.18)) : Style.normalFill
            border.width: chosen ? Style.space(2) : Style.normalBorderWidth
            border.color: chosen ? Colors.warm : Style.normalBorderColor
            opacity: open ? 1 : 0.45
            Text {
              id: stakeLabel
              anchors.centerIn: parent
              text: open ? modelData.name : (index + 1) + " · ?"
              color: chosen ? Colors.warm : Colors.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
              font.bold: chosen
            }
            MouseArea {
              anchors.fill: parent
              enabled: parent.open
              cursorShape: Qt.PointingHandCursor
              onClicked: root.stake = index
            }
          }
        }
      }
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        width: 3 * Style.space(285) + 2 * Style.spacing.lg
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        readonly property var s: root.stakes.length ? root.stakes[root.chosenStake] : null
        text: s ? (s.rules.length ? s.rules.map(function(r) { return I18n.tr(r) }).join(" · ") : I18n.tr(s.description))
          + "  —  " + s.quip : ""
        color: Colors.muted
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
      }
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        visible: root.topStake < root.stakes.length - 1
        text: I18n.tr("Win with this deck to unlock the next level.")
        color: Colors.muted
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.italic: true
      }
    }

    // The language of the game (L switches).
    Row {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.md
      Text {
        anchors.verticalCenter: parent.verticalCenter
        text: I18n.tr("Language (L)")
        color: Colors.muted
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
        enabled: root.decks.length > 0 && !root.decks[root.current].achievement
        onClicked: root.startChosen()
      }
      Button {
        text: I18n.tr("Collection (C)")
        onClicked: root.collection()
      }
      Button {
        visible: root.canCancel
        text: I18n.tr("Back (Escape)")
        onClicked: root.cancel()
      }
    }
  }
}
