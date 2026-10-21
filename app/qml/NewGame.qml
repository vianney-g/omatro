import QtQuick
import omatro
import "Labels.js" as Labels

// The new game screen: pick the machine to play on in a carousel (←/→,
// Enter), or go back to the game in progress (Escape).
Rectangle {
  id: root

  property var decks: []
  property int current: 0
  // The difficulty levels, and the one chosen (↑/↓), within what the
  // chosen deck has unlocked.
  property var stakes: []
  property int stake: 0
  readonly property var deck: decks.length ? decks[current] : null
  readonly property int topStake: deck ? stakeIndex(deck.top_stake) : 0
  readonly property int chosenStake: Math.min(stake, topStake)
  property bool canCancel: false
  // A problem to report (a saved game that could not be resumed).
  property string message: ""
  // The seed typed in (S), empty for a random one.
  property alias seed: seedInput.text
  signal start(string deck, string stake, string seed)
  // The seed field gives the keys back.
  signal seedDone()
  signal cancel()
  signal options()
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
  // A game in progress, not over: starting another abandons it, so the
  // first Start asks again (its ante, for the warning; 0 for none).
  property int ongoingAnte: 0
  property bool confirming: false
  onVisibleChanged: confirming = false
  onCurrentChanged: confirming = false
  function startChosen() {
    if (!deck || deck.achievement || !stakes.length) return
    if (ongoingAnte > 0 && !confirming) { confirming = true; return }
    confirming = false
    start(deck.id, stakes[chosenStake].id, seed)
  }
  function editSeed() { seedInput.forceActiveFocus() }

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
      text: I18n.tr("Pick your machine · ←/→ to choose · Enter to start") + (root.canCancel ? I18n.tr(" · Escape to go back") : "")
        + I18n.tr(" · C: collection") + I18n.tr(" · O: settings")
      color: Colors.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }
    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      visible: root.message !== "" || root.confirming
      text: root.confirming
        ? I18n.tr("Your game in progress (ante {n}) will count as abandoned. Enter again to start · Escape to go back to it", { n: root.ongoingAnte })
        : root.message
      color: Colors.urgent
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }

    // The decks in a carousel: the chosen one in front, its neighbours
    // around it, smaller (click one to bring it forward).
    Item {
      id: carousel
      anchors.horizontalCenter: parent.horizontalCenter
      width: Style.space(980)
      height: Style.space(140)
      clip: true
      // Distance from the centre of the first and second neighbours.
      readonly property real near: Style.space(300)
      readonly property real far: Style.space(490)
      Repeater {
        model: root.decks
        Rectangle {
          readonly property bool chosen: index === root.current
          // Its place from the chosen one, the shortest way round.
          readonly property int offset: {
            var n = root.decks.length
            var d = ((index - root.current) % n + n) % n
            return d > n / 2 ? d - n : d
          }
          readonly property int distance: Math.abs(offset)
          visible: distance <= 2
          x: carousel.width / 2 + (offset === 0 ? 0 : (offset > 0 ? 1 : -1) * (distance === 1 ? carousel.near : carousel.far)) - width / 2
          y: (carousel.height - height) / 2
          z: 3 - distance
          scale: distance === 0 ? 1 : distance === 1 ? 0.8 : 0.62
          opacity: distance === 0 ? 1 : distance === 1 ? 0.7 : 0.35
          Behavior on x { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
          Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
          Behavior on opacity { NumberAnimation { duration: 180 } }
          // A hidden deck, until its achievement.
          readonly property bool locked: !!modelData.achievement
          width: Style.space(320)
          height: Style.space(120)
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
              maximumLineCount: 3
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
              maximumLineCount: 3
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
    // ‹ n / total ›, also clickable.
    Row {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.lg
      Button { compact: true; text: "‹"; onClicked: root.move(-1) }
      Text {
        anchors.verticalCenter: parent.verticalCenter
        text: root.decks.length ? (root.current + 1) + " / " + root.decks.length : ""
        color: Colors.muted
        font.family: Style.font.family
        font.pixelSize: Style.font.body
      }
      Button { compact: true; text: "›"; onClicked: root.move(1) }
    }

    // The difficulty level: those the deck unlocked, then locked ones.
    Column {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.md
      visible: root.stakes.length > 0
      Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Style.spacing.sm
        Text {
          anchors.verticalCenter: parent.verticalCenter
          rightPadding: Style.spacing.md
          text: I18n.tr("Level (↑/↓)")
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.body
        }
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
      // The level's own rule, then those it keeps from the levels below.
      Column {
        id: stakeRules
        anchors.horizontalCenter: parent.horizontalCenter
        width: 3 * Style.space(285) + 2 * Style.spacing.lg
        spacing: Style.spacing.xs
        readonly property var s: root.stakes.length ? root.stakes[root.chosenStake] : null
        Text {
          width: parent.width
          horizontalAlignment: Text.AlignHCenter
          wrapMode: Text.WordWrap
          text: stakeRules.s ? I18n.tr(stakeRules.s.description) + "  —  " + stakeRules.s.quip : ""
          color: Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.body
        }
        Text {
          width: parent.width
          visible: stakeRules.s !== null && stakeRules.s.rules.length > 1
          horizontalAlignment: Text.AlignHCenter
          wrapMode: Text.WordWrap
          text: visible ? I18n.tr("Plus the levels below: {rules}", {
            rules: stakeRules.s.rules.slice(0, -1).map(function(r) { return I18n.tr(r) }).join(" · ")
          }) : ""
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
        }
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

    // The seed: random unless one is typed in, to replay or share a game.
    Column {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.sm
      Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Style.spacing.md
        Text {
          anchors.verticalCenter: parent.verticalCenter
          text: I18n.tr("Seed (S)")
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.body
        }
        Rectangle {
          width: Style.space(180)
          height: seedInput.implicitHeight + 2 * Style.spacing.sm
          radius: Style.cornerRadius
          color: Style.normalFill
          border.width: seedInput.activeFocus ? Style.space(2) : Style.normalBorderWidth
          border.color: seedInput.activeFocus ? Colors.accent : Style.normalBorderColor
          TextInput {
            id: seedInput
            anchors { fill: parent; leftMargin: Style.spacing.md; rightMargin: Style.spacing.md }
            verticalAlignment: TextInput.AlignVCenter
            clip: true
            validator: RegularExpressionValidator { regularExpression: /[0-9A-Za-z]{0,12}/ }
            color: Colors.foreground
            selectionColor: Colors.accent
            selectedTextColor: Colors.background
            font.family: Style.font.family
            font.pixelSize: Style.font.body
            font.bold: true
            font.capitalization: Font.AllUppercase
            Keys.onPressed: function(event) {
              if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) root.startChosen()
              else if (event.key === Qt.Key_Escape || event.key === Qt.Key_Tab) root.seedDone()
              else return
              event.accepted = true
            }
          }
          Text {
            anchors { fill: parent; leftMargin: Style.spacing.md }
            verticalAlignment: Text.AlignVCenter
            visible: seedInput.text === "" && !seedInput.activeFocus
            text: I18n.tr("random")
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.body
            font.italic: true
          }
        }
      }
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: seedInput.text !== ""
          ? I18n.tr("A chosen seed counts for no stat nor achievement.")
          : I18n.tr("Replay or share a game by its seed (I during the game).")
        color: seedInput.text !== "" ? Colors.warm : Colors.muted
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.italic: true
      }
    }

    Row {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.xxl
      Button {
        text: I18n.tr(root.confirming ? "Abandon and start (Enter)" : "Start (Enter)")
        enabled: root.decks.length > 0 && !root.decks[root.current].achievement
        onClicked: root.startChosen()
      }
      Button {
        text: I18n.tr("Collection (C)")
        onClicked: root.collection()
      }
      Button {
        text: I18n.tr("Settings (O)")
        onClicked: root.options()
      }
      Button {
        visible: root.canCancel
        text: I18n.tr("Back (Escape)")
        onClicked: root.cancel()
      }
    }
  }
}
