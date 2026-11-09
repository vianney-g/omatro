import QtQuick
import omatro
import "Labels.js" as Labels

// At start, when a game was left in progress: resume it (Enter), or pick a
// deck for a new one (N), which replaces it once started. The game shows as
// it was left, a job stopped in the shell: its deck, jokers and clipboard.
Rectangle {
  id: root

  // The saved game's state.
  property var saved: null
  signal resume()
  signal newGame()
  // A joker is dealt during the entrance (Main sounds it).
  signal dealt()

  color: Util.alpha(Colors.background, 0.96)

  MouseArea { anchors.fill: parent }

  // Where the game stands: blind, shop or pack.
  readonly property string where: {
    var s = saved
    if (!s) return ""
    var blind = s.blinds.find(function(b) { return b.kind === s.blind })
    return s.phase === "shop" ? I18n.tr("in the shop")
      : s.phase === "pack" ? I18n.tr("opening a package")
      : blind ? blind.name : ""
  }

  // ---- the entrance: everything comes in turn as `t` goes from 0 to 1
  property real t: 1
  // Progress (0 to 1) of a part that starts at `start` and lasts `length`.
  function step(start, length) { return Math.max(0, Math.min(1, (t - start) / length)) }
  function outCubic(x) { return 1 - Math.pow(1 - x, 3) }
  function outBack(x) { var c = 1.70158; return 1 + (c + 1) * Math.pow(x - 1, 3) + c * Math.pow(x - 1, 2) }
  NumberAnimation on t { id: entrance; running: false; from: 0; to: 1; duration: 1600 }
  function enter() { if (visible && saved) entrance.restart() }
  onVisibleChanged: enter()
  onSavedChanged: enter()

  // ---- the antes: the ones beaten, this one, those left (the last ones
  // played in endless mode).
  readonly property int lastAnte: saved ? Math.max(saved.winning_ante, saved.ante) : 0
  readonly property int firstAnte: Math.max(1, lastAnte - 11)
  readonly property var antes: {
    var a = []
    for (var n = firstAnte; n <= lastAnte; n++) a.push(n)
    return a
  }

  Column {
    anchors.centerIn: parent
    spacing: Style.spacing.huge + Style.spacing.lg

    // ---- the title, as the shell reports a stopped job
    Column {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.md
      opacity: root.step(0, 0.2)
      transform: Translate { y: -Style.space(20) * (1 - root.outCubic(root.step(0, 0.25))) }
      Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Style.space(40)
        Text {
          text: "[1]+  Stopped"
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.displayLarge
          font.bold: true
        }
        Text {
          text: "omatro"
          color: Colors.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.displayLarge
          font.bold: true
        }
      }
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: I18n.tr("Your game waits where you left it.")
        color: Colors.muted
        font.family: Style.font.family
        font.pixelSize: Style.font.title
      }
    }

    // ---- the track of antes, then the boss waiting at the end of this one
    Column {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: Style.spacing.lg
      Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Style.spacing.md
        Repeater {
          model: root.antes
          Rectangle {
            id: ante
            readonly property bool beaten: root.saved && modelData < root.saved.ante
            readonly property bool current: root.saved && modelData === root.saved.ante
            readonly property bool final: root.saved && modelData === root.saved.winning_ante
            readonly property real shown: root.outBack(root.step(0.15 + index * 0.04, 0.2))
            anchors.verticalCenter: parent.verticalCenter
            width: current ? pips.implicitWidth + 2 * Style.space(10) : Style.space(34)
            height: Style.space(34)
            scale: shown
            opacity: Math.min(1, shown)
            radius: Style.cornerRadius
            color: beaten ? Util.alpha(Colors.accent, 0.35)
              : current ? Qt.tint(Colors.background, Util.alpha(Colors.accent, 0.15)) : "transparent"
            border.width: current || final ? Style.space(2) : Style.normalBorderWidth
            border.color: current ? Colors.accent : final ? Colors.warm : Style.normalBorderColor
            // This ante breathes.
            SequentialAnimation on border.color {
              running: ante.current && root.visible
              loops: Animation.Infinite
              ColorAnimation { to: Util.alpha(Colors.accent, 0.35); duration: 700; easing.type: Easing.InOutSine }
              ColorAnimation { to: Colors.accent; duration: 700; easing.type: Easing.InOutSine }
            }
            Text {
              anchors.centerIn: parent
              visible: !ante.current
              text: ante.beaten ? "✓" : modelData
              color: ante.beaten ? Colors.foreground : ante.final ? Colors.warm : Colors.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              font.bold: ante.beaten || ante.final
            }
            // This ante: its number, then its three blinds.
            Row {
              id: pips
              anchors.centerIn: parent
              visible: ante.current
              spacing: Style.spacing.md
              Text {
                anchors.verticalCenter: parent.verticalCenter
                text: modelData
                color: Colors.accent
                font.family: Style.font.family
                font.pixelSize: Style.font.subtitle
                font.bold: true
              }
              Repeater {
                model: ante.current && root.saved ? root.saved.blinds : []
                Rectangle {
                  anchors.verticalCenter: parent.verticalCenter
                  readonly property bool done: modelData.status === "defeated" || modelData.status === "skipped"
                  readonly property bool next: modelData.status === "select" || modelData.status === "current"
                  width: Style.space(modelData.kind === "boss" ? 14 : 10)
                  height: width
                  radius: width / 2
                  color: modelData.status === "defeated" ? Colors.accent
                    : modelData.status === "skipped" ? Colors.muted
                    : "transparent"
                  border.width: Style.space(2)
                  border.color: modelData.kind === "boss" ? Colors.urgent
                    : next ? Colors.accent : Style.normalBorderColor
                }
              }
            }
          }
        }
      }
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        opacity: root.step(0.5, 0.2)
        text: root.saved ? I18n.tr("Boss: {name}", { name: root.saved.boss.name }) : ""
        color: Colors.urgent
        font.family: Style.font.family
        font.pixelSize: Style.font.subtitle
        font.bold: true
      }
    }

    // ---- the game as it was left
    Rectangle {
      id: board
      anchors.horizontalCenter: parent.horizontalCenter
      opacity: root.step(0.1, 0.25)
      scale: 0.94 + 0.06 * root.outCubic(root.step(0.1, 0.3))
      width: boardRow.implicitWidth + 2 * Style.space(28)
      height: boardRow.implicitHeight + 2 * Style.space(24)
      radius: Style.cornerRadius
      color: Style.normalFill
      border.width: Style.normalBorderWidth
      border.color: Style.normalBorderColor

      Row {
        id: boardRow
        anchors.centerIn: parent
        spacing: Style.space(40)

        // The deck: a stack of backs, its name and difficulty level.
        Column {
          anchors.verticalCenter: parent.verticalCenter
          spacing: Style.spacing.lg
          Item {
            anchors.horizontalCenter: parent.horizontalCenter
            width: Style.space(96) + 2 * Style.space(4)
            height: Style.space(136) + 2 * Style.space(4)
            Repeater {
              model: 3
              CardBack {
                x: (2 - index) * Style.space(4)
                y: index * Style.space(4)
                width: Style.space(96)
                height: Style.space(136)
              }
            }
          }
          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.saved && root.saved.run_deck ? root.saved.run_deck.name : ""
            color: Colors.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.subtitle
            font.bold: true
          }
          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.saved ? root.saved.stake.name : ""
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
        }

        // The jokers, fanned out, and the clipboard below them.
        Column {
          anchors.verticalCenter: parent.verticalCenter
          spacing: Style.spacing.xl
          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.saved ? I18n.tr("Jokers {a} / {b}", { a: root.saved.jokers.length, b: root.saved.max_jokers }) : ""
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
          Fan {
            id: jokers
            anchors.horizontalCenter: parent.horizontalCenter
            width: Math.max(implicitWidth, Style.space(120))
            height: Style.space(230)
            count: root.saved ? root.saved.jokers.length : 0
            itemWidth: Style.space(120)
            spacing: Style.spacing.xl
            available: Style.space(560)
            Repeater {
              model: root.saved ? root.saved.jokers : []
              // Dealt one after the other, from the deck on the left.
              Item {
                readonly property real dealt: root.outBack(root.step(0.3 + index * 0.07, 0.3))
                readonly property bool leaving: root.t > 0.3 + index * 0.07
                onLeavingChanged: if (leaving && entrance.running) root.dealt()
                x: jokers.xOf(index)
                y: jokers.liftOf(index)
                z: jokers.zOf(index)
                Behavior on y { NumberAnimation { duration: 100 } }
                width: jokers.itemWidth
                height: card.height
                opacity: Math.min(1, root.step(0.3 + index * 0.07, 0.12))
                rotation: -12 * (1 - dealt)
                transform: Translate { x: -Style.space(160) * (1 - Math.min(1, dealt)) }
                JokerView {
                  id: card
                  width: parent.width
                  joker: modelData
                }
                HoverHandler { onHoveredChanged: jokers.hover(index, hovered) }
              }
            }
            // No joker yet: an empty slot.
            Rectangle {
              visible: jokers.count === 0
              width: Style.space(120)
              height: Style.space(230)
              radius: Style.cornerRadius
              color: "transparent"
              border.width: Style.normalBorderWidth
              border.color: Style.normalBorderColor
              Text {
                anchors.centerIn: parent
                text: I18n.tr("no joker")
                color: Colors.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.body
              }
            }
          }
        }

        // The clipboard, when it holds anything.
        Column {
          anchors.verticalCenter: parent.verticalCenter
          visible: root.saved !== null && root.saved.consumables.length > 0
          spacing: Style.spacing.xl
          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.saved ? I18n.tr("Clipboard {a} / {b}", { a: root.saved.consumables.length, b: root.saved.consumable_slots }) : ""
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
          Row {
            spacing: Style.spacing.lg
            Repeater {
              model: root.saved ? root.saved.consumables : []
              ConsumableView {
                width: Style.space(100)
                item: modelData
              }
            }
          }
        }

        // Where the game stands, and the money.
        Column {
          anchors.verticalCenter: parent.verticalCenter
          spacing: Style.spacing.md
          Text {
            text: I18n.tr("Ante")
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
          Text {
            text: root.saved ? root.saved.ante + " / " + root.saved.winning_ante : ""
            color: Colors.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.display
            font.bold: true
          }
          Text {
            text: root.where
            color: Colors.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.subtitle
          }
          Item { width: 1; height: Style.spacing.xl }
          Text {
            text: I18n.tr("Money")
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
          Text {
            text: root.saved ? Labels.money(root.saved.money) : ""
            color: Colors.warm
            font.family: Style.font.family
            font.pixelSize: Style.font.display
            font.bold: true
          }
        }
      }
    }

    // ---- the two choices, as cards that lift when hovered
    Row {
      anchors.horizontalCenter: parent.horizontalCenter
      opacity: root.step(0.6, 0.2)
      transform: Translate { y: Style.space(30) * (1 - root.outBack(root.step(0.6, 0.35))) }
      spacing: Style.space(40)
      Choice {
        command: "fg"
        label: I18n.tr("Resume the game")
        key: I18n.tr("Enter")
        main: true
        onChosen: root.resume()
      }
      Choice {
        command: "kill %1"
        label: I18n.tr("New game")
        key: "N"
        hint: I18n.tr("A new game replaces it once started.")
        onChosen: root.newGame()
      }
    }
  }

  component Choice: Item {
    id: choice
    property string command: ""
    property string label: ""
    property string key: ""
    property string hint: ""
    property bool main: false
    signal chosen()

    width: Style.space(260)
    height: Style.space(150)

    Rectangle {
      id: face
      width: parent.width
      height: parent.height
      y: hover.hovered ? -Style.space(10) : 0
      scale: tap.pressed ? 0.97 : 1
      Behavior on y { NumberAnimation { duration: 140; easing.type: Easing.OutBack } }
      Behavior on scale { NumberAnimation { duration: 80 } }
      radius: Style.cornerRadius
      color: choice.main ? Qt.tint(Colors.background, Util.alpha(Colors.accent, hover.hovered ? 0.3 : 0.18))
        : hover.hovered ? Style.hoverFill : Style.normalFill
      border.width: Style.space(2)
      border.color: choice.main ? Colors.accent : Style.normalBorderColor

      Column {
        anchors.centerIn: parent
        spacing: Style.spacing.lg
        Text {
          anchors.horizontalCenter: parent.horizontalCenter
          text: "$ " + choice.command
          color: choice.main ? Colors.accent : Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.display
          font.bold: true
        }
        Text {
          anchors.horizontalCenter: parent.horizontalCenter
          text: choice.hint !== "" && hover.hovered ? choice.hint : choice.label
          width: choice.width - 2 * Style.space(16)
          horizontalAlignment: Text.AlignHCenter
          wrapMode: Text.WordWrap
          color: choice.hint !== "" && hover.hovered ? Colors.urgent : Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.subtitle
        }
        Rectangle {
          anchors.horizontalCenter: parent.horizontalCenter
          width: keyText.implicitWidth + 2 * Style.space(8)
          height: keyText.implicitHeight + 2 * Style.space(3)
          radius: Style.cornerRadius
          color: "transparent"
          border.width: Style.normalBorderWidth
          border.color: Style.normalBorderColor
          Text {
            id: keyText
            anchors.centerIn: parent
            text: choice.key
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.caption
          }
        }
      }
    }
    HoverHandler { id: hover; cursorShape: Qt.PointingHandCursor }
    TapHandler { id: tap; onTapped: choice.chosen() }
  }
}
